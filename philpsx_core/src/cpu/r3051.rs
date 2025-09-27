// SPDX-License-Identifier: GPL-3.0
// r3051.rs - Copyright Phillip Potter, 2025, under GPLv3 only.

use super::{Cpu, CpuBridge};
use philpsx_utility::{CustomInteger, SystemBusHolder};
use mips_exception::{MIPSException, MIPSExceptionReason};
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

    /// This is a utility function to swap the endianness of a word. It can be
    /// used to allow the processor to operate in both endian modes by transparently
    /// swapping the byte order before writing or after writing.
    fn swap_word_endianness(&self, word: i32) -> i32 {
        (word << 24) |
        ((word << 8) & 0xFF0000) |
        (word.logical_rshift(8) & 0xFF00) |
        (word.logical_rshift(24) & 0xFF)
    }

    /// This function reads a data value of the specified width, and abstracts
    /// this functionality from the MEM stage.
    fn read_data_value(&mut self, bridge: &mut dyn CpuBridge, width: R3051Width, address: i32) -> i32 {

        // Get physical address.
        let physical_address = self.sccp.virtual_to_physical(address);
        let mut temp_physical_address = (physical_address as i64) & 0xFFFFFFFF;

        // Is cache isolated? Although we don't have a data cache (it is used as
        // scratchpad instead) we should read from instruction cache if so.
        if self.sccp.is_data_cache_isolated() { // Yes.

            // Read from instruction cache no matter what.
            match width {

                R3051Width::BYTE => {
                    0xFF & (self.read_instruction_cache_byte(physical_address) as i32)
                },

                R3051Width::HALFWORD => {
                    let mut value = 0xFF & self.read_instruction_cache_byte(temp_physical_address as i32) as i32;
                    temp_physical_address += 1;
                    value = (value << 8) |
                            (0xFF & self.read_instruction_cache_byte(temp_physical_address as i32) as i32);
                    value
                },

                R3051Width::WORD => {
                    self.read_instruction_cache_word(physical_address)
                },
            }
        } else { // No.

            // Check if we are reading from scratchpad.
            if (0x1F800000..0x1F800400).contains(&temp_physical_address) &&
               bridge.scratchpad_enabled(self) {

                // Although we are sending reads to system, this actually
                // accesses scratchpad.
                match width {

                    R3051Width::BYTE => {
                        0xFF & bridge.read_byte(self, physical_address) as i32
                    },

                    R3051Width::HALFWORD => {
                        let mut value = 0xFF & bridge.read_byte(self, temp_physical_address as i32) as i32;
                        temp_physical_address += 1;
                        value = (value << 8) |
                                (0xFF & bridge.read_byte(self, temp_physical_address as i32) as i32);

                        value
                    },

                    R3051Width::WORD => {
                        bridge.read_word(self, physical_address)
                    },
                }
            } else {

                // Calculate delay cycles before reading from system.
                let delay_cycles = bridge.how_how_many_stall_cycles(self, physical_address);

                // Begin transaction.

                // Read value straight away.
                let value = match width {

                    R3051Width::BYTE => {
                        0xFF & bridge.read_byte(self, physical_address) as i32
                    },

                    R3051Width::HALFWORD => {
                        let mut value = 0xFF & bridge.read_byte(self, temp_physical_address as i32) as i32;
                        temp_physical_address = if bridge.ok_to_increment(self, temp_physical_address) {
                            temp_physical_address + 1
                        } else {
                            temp_physical_address
                        };
                        value = (value << 8) |
                                (0xFF & bridge.read_byte(self, temp_physical_address as i32) as i32);
                        value
                    },

                    R3051Width::WORD => {
                        bridge.read_word(self, physical_address)
                    },
                };

                self.cycles += delay_cycles;
                self.total_cycles += delay_cycles as i64;

                // End transaction.
                value
            }
        }
    }

    /// This function writes a data value of the specified width, and abstracts
    /// this functionality from the MEM stage.
    fn write_data_value(
        &mut self,
        bridge: &mut dyn CpuBridge,
        width: R3051Width,
        address: i32,
        value: i32
    ) {

        // Get physical address.
        let physical_address = self.sccp.virtual_to_physical(address);
        let mut temp_physical_address = (physical_address as i64) & 0xFFFFFFFF;

        // Is cache isolated?
        if self.sccp.is_data_cache_isolated() { // Yes.

            // Write to instruction cache no matter what.
            match width {

                R3051Width::BYTE => {
                    self.write_instruction_cache_byte(physical_address, value as i8);
                },

                R3051Width::HALFWORD => {
                    self.write_instruction_cache_byte(temp_physical_address as i32, value.logical_rshift(8) as i8);
                    temp_physical_address += 1;
                    self.write_instruction_cache_byte(temp_physical_address as i32, value as i8);
                },

                R3051Width::WORD => {
                    self.write_instruction_cache_word(physical_address, value);
                },
            }
        } else { // No.

            // Check if we are writing to scratchpad.
            if (0x1F800000..0x1F800400).contains(&temp_physical_address) &&
               bridge.scratchpad_enabled(self) {

                // Although we are sending writes to the system, they actually
                // go to the scratchpad.
                match width {
                    R3051Width::BYTE => {
                        bridge.write_byte(self, physical_address, value as i8);
                    },

                    R3051Width::HALFWORD => {
                        bridge.write_byte(self, temp_physical_address as i32, value.logical_rshift(8) as i8);
                        temp_physical_address += 1;
                        bridge.write_byte(self, temp_physical_address as i32, value as i8);
                    },

                    R3051Width::WORD => {
                        bridge.write_word(self, physical_address, value);
                    },
                }
            } else {

                // Calculate delay cycles before writing to system.
                let delay_cycles = bridge.how_how_many_stall_cycles(self, physical_address);

                // Begin transaction.

                // Write value.
                match width {

                    R3051Width::BYTE => {
                        bridge.write_byte(self, physical_address, value as i8);
                    },

                    R3051Width::HALFWORD => {
                        bridge.write_byte(self, temp_physical_address as i32, value.logical_rshift(8) as i8);
                        temp_physical_address = if bridge.ok_to_increment(self, temp_physical_address) {
                            temp_physical_address + 1
                        } else {
                            temp_physical_address
                        };
                        bridge.write_byte(self, temp_physical_address as i32, value as i8);
                    },

                    R3051Width::WORD => {
                        bridge.write_word(self, physical_address, value);
                    },
                }

                self.cycles += delay_cycles;
                self.total_cycles += delay_cycles as i64;

                // End transaction.
            }
        }
    }

    /// This function reads an instruction word. It allows abstraction of this
    /// functionality from the IF pipeline stage.
    fn read_instruction_word(
        &mut self,
        bridge: &mut dyn CpuBridge,
        address: i32,
        temp_branch_address: i32
    ) -> i64 {

        // Check for dodgy address.
        if self.sccp.is_address_allowed(address) || self.program_counter % 4 != 0 {

            // Trigger exception.
            self.exception.bad_address = address;
            self.exception.exception_reason = MIPSExceptionReason::ADEL;
            self.exception.is_in_branch_delay_slot = self.prev_was_branch;
            self.exception.program_counter_origin = if self.exception.is_in_branch_delay_slot {
                temp_branch_address
            } else {
                address
            };

            //R3051_handleException(cpu);
            return -1;
        }

        // Check if instruction cache enabled.
        let instruction_cache_enabled = bridge.instruction_cache_enabled(self);

        // Get physical address.
        let physical_address = self.sccp.virtual_to_physical(address);

        // Check if address is cacheable or not.
        let word_val = if self.sccp.is_cacheable(address) && instruction_cache_enabled {

            // Check cache for hit.
            if self.check_for_instruction_cache_hit(physical_address) {
                self.read_instruction_cache_word(physical_address)
            } else {

                // Refill cache then set wordVal.
                if self.get_system_bus_holder(bridge) != SystemBusHolder::CPU {

                    // Stall for one cycle as BIU is being used by
                    // another component.
                    return -1;
                } else {
                    // Begin transaction.

                    let stall_cycles = bridge.how_how_many_stall_cycles(self, physical_address);
                    self.cycles += stall_cycles;
                    self.total_cycles += stall_cycles as i64;
                    self.refill_instruction_cache_line(bridge, physical_address);
                    self.read_instruction_cache_word(physical_address)

                    // End transaction.
                }
            }
        } else {

            // Read word straight from system, stalling if being used.
            if self.get_system_bus_holder(bridge) != SystemBusHolder::CPU {

                // Stall for one cycle as BIU is being used by another component.
                return -1;
            } else {
                // Begin transaction.

                let stall_cycles = bridge.how_how_many_stall_cycles(self, physical_address);
                self.cycles += stall_cycles;
                self.total_cycles += stall_cycles as i64;
                bridge.read_word(self, physical_address)

                // End transaction.
            }
        };

        // Return word variable.
        0xFFFFFFFF & word_val as i64
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

/// This enum is used to specify the width we want to use (byte/half word/word).
enum R3051Width {
    BYTE,
    HALFWORD,
    WORD,
}