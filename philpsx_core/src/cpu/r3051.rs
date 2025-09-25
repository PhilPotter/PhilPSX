// SPDX-License-Identifier: GPL-3.0
// r3051.rs - Copyright Phillip Potter, 2025, under GPLv3 only.

use super::{Cpu, CpuBridge};
use philpsx_utility::{CustomInteger, SystemBusHolder};
use mips_exception::MIPSException;
use cp0::CP0;
use cp2::CP2;

/// This module contains an implementation of the MIPS exceptions
/// modelled from inside the R3051 processor.
mod mips_exception;

/// This module contains an implementation of the CP0 co-processor, also
/// referred to as the System Control Co-processor.
mod cp0;

/// This module contains an implementation of the CP2 co-processor, also
/// referred to as the Geometry Transformation Engine.
mod cp2;

/// The number of general CPU registers.
const REGISTER_COUNT: usize = 32;

/// The maximum size of the instruction cache data array in entries.
const INSTRUCTION_CACHE_DATA_MAX_ENTRIES: usize = 4096;

/// The maximum size of the instruction cache tag array in entries.
const INSTRUCTION_CACHE_TAG_MAX_ENTRIES: usize = 256;

/// The maximum size of the instruction cache validity array in entries.
const INSTRUCTION_CACHE_VALID_MAX_ENTRIES: usize = 256;

/// This structure represents the internal state of the R3051 processor.
/// It contains registers, and internal subcomponents.
pub struct R3051 {

    // Register definitions.
    general_registers: [i32; REGISTER_COUNT],
    program_counter: i32,
    hi_reg: i32,
    lo_reg: i32,

    // Jump address holder and boolean.
    jump_address: i32,
    jump_pending: bool,

    // Co-processors.
    sccp: CP0,
    gte: CP2,

    // System bus holder definition.
    system_bus_holder: SystemBusHolder,

    // This stores the current exception.
    exception: MIPSException,

    // This tells us if the last instruction was a branch/jump instruction.
    prev_was_branch: bool,
    is_branch: bool,

    // This counts the cycles of the current instruction.
    cycles: i32,
    gte_cycles: i32,
    total_cycles: i64,

    // Instruction cache variables - previously these were modelled
    // separately in the C version. All are heap allocated.
    instruction_cache_data: Vec<i8>,
    instruction_cache_tag: Vec<i32>,
    instruction_cache_valid: Vec<bool>,
}

/// Implementation functions for the R3051 component itself.
impl R3051 {

    /// Creates a new R3051 object with the correct initial state.
    pub fn new() -> Self {

        let mut r3051 = R3051 {

            // Setup registers (remember, r1 should always be 0).
            general_registers: [0; REGISTER_COUNT],
            program_counter: 0,
            hi_reg: 0,
            lo_reg: 0,

            // Setup jump variables.
            jump_address: 0,
            jump_pending: false,

            // Setup co-processors.
            sccp: CP0::new(),
            gte: CP2::new(),

            // Setup the bus holder.
            system_bus_holder: SystemBusHolder::CPU,

            // Create exception object.
            exception: MIPSException::new(),

            // Setup the branch marker.
            prev_was_branch: false,
            is_branch: false,

            // Setup instruction cycle count.
            cycles: 0,
            gte_cycles: 0,
            total_cycles: 0,

            // Setup instruction cache variables.
            instruction_cache_data: vec![0; INSTRUCTION_CACHE_DATA_MAX_ENTRIES],
            instruction_cache_tag: vec![0; INSTRUCTION_CACHE_TAG_MAX_ENTRIES],
            instruction_cache_valid: vec![false; INSTRUCTION_CACHE_VALID_MAX_ENTRIES],
        };

        r3051.reset();

        r3051
    }

    /// Set the R3051 object to its correct initial state.
    fn reset(&mut self) {

        // Patch in later with proper reset exception vector.
        self.program_counter = self.sccp.get_reset_exception_vector();
    }

    /// This function checks for an instruction cache hit. The address provided
    /// must be physical and not virtual.
    fn check_for_instruction_cache_hit(&self, address: i32) -> bool {

        let tag_index = (address.logical_rshift(4) & 0xFF) as usize;
        let expected_tag = address.logical_rshift(12) & 0xFFFFF;

        self.instruction_cache_tag[tag_index] == expected_tag &&
        self.instruction_cache_valid[tag_index]
    }

    /// This function retrieves a word from the correct address.
    fn read_instruction_cache_word(&self, address: i32) -> i32 {

        let data_index = (address & 0xFFC) as usize;

        (((self.instruction_cache_data[data_index] as i32) & 0xFF) << 24) |
        (((self.instruction_cache_data[data_index + 1] as i32) & 0xFF) << 16) |
        (((self.instruction_cache_data[data_index + 2] as i32) & 0xFF) << 8) |
        ((self.instruction_cache_data[data_index + 3] as i32) & 0xFF)
    }

    /// This function retrieves a byte from the correct address.
    fn read_instruction_cache_byte(&self, address: i32) -> i8 {

        let data_index = (address & 0xFFF) as usize;

        self.instruction_cache_data[data_index]
    }

    /// This function writes a word to the correct address, and invalidates
    /// the correct cache line.
    fn write_instruction_cache_word(&mut self, address: i32, value: i32) {

        let data_index = (address & 0xFFC) as usize;

        // Update correct word.
        self.instruction_cache_data[data_index] = value.logical_rshift(24) as i8;
        self.instruction_cache_data[data_index + 1] = value.logical_rshift(16) as i8;
        self.instruction_cache_data[data_index + 2] = value.logical_rshift(8) as i8;
        self.instruction_cache_data[data_index + 3] = value as i8;

        // Invalidate line if cache is isolated.
        if self.sccp.is_data_cache_isolated() {
            let tag_index = (address.logical_rshift(4) & 0xFF) as usize;
            let tag = address.logical_rshift(12) & 0xFFFFF;
            self.instruction_cache_tag[tag_index] = tag;
            self.instruction_cache_valid[tag_index] = false;
        }
    }

    /// This function writes a byte to the correct address, and invalidates
    /// the correct cache line.
    fn write_instruction_cache_byte(&mut self, address: i32, value: i8) {

        let data_index = (address & 0xFFF) as usize;

        // Update correct byte.
        self.instruction_cache_data[data_index] = value;

        // Invalidate line if cache is isolated.
        if self.sccp.is_data_cache_isolated() {
            let tag_index = (address.logical_rshift(4) & 0xFF) as usize;
            let tag = address.logical_rshift(12) & 0xFFFFF;
            self.instruction_cache_tag[tag_index] = tag;
            self.instruction_cache_valid[tag_index] = false;
        }
    }

    /// This function refills a cache line using an address.
    fn refill_instruction_cache_line(&mut self, bridge: &mut dyn CpuBridge, address: i32) {
        // Refill a line - four words.
        // Check if cache is isolated first.
        if self.sccp.is_data_cache_isolated() {
            return;
        }

        // Calculate tag index and tag.
        let tag_index = (address.logical_rshift(4) & 0xFF) as usize;
        let tag = address.logical_rshift(12) & 0xFFFFF;

        // Write tag and valid flag.
        self.instruction_cache_tag[tag_index] = tag;
        self.instruction_cache_valid[tag_index] = true;

        // Refill cache line.
        let starting_address = (address & 0xFFFFFFF0_u32 as i32) as usize;
        for offset in 0..16 {
            self.instruction_cache_data[(starting_address + offset) & 0xFFF] =
                bridge.read_byte(self, starting_address as i32);
        }
    }
}

/// Implementation functions to be called from anything that understands what
/// a Cpu object is.
impl Cpu for R3051 {

    /// Set the system bus holder.
    fn set_system_bus_holder(
        &mut self,
        _bridge: &mut dyn CpuBridge,
        holder: SystemBusHolder
    ) {
        self.system_bus_holder = holder;
    }

    /// Implementations must use this to retrieve the system bus holder.
    fn get_system_bus_holder(
        &self,
        _bridge: &mut dyn CpuBridge
    ) -> SystemBusHolder {
        self.system_bus_holder
    }
}