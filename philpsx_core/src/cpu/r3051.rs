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

            self.handle_exception();
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

    /// This function can deal with an exception, making sure the right things
    /// are done.
    fn handle_exception(&mut self) -> bool {

        // Exit if exception is NULL.
        if self.exception.exception_reason == MIPSExceptionReason::NULL {
            return false;
        }

        // Wipe out pending jump and branch delay info.
        self.jump_pending = false;
        self.prev_was_branch = false;

        // Fetch cause register and status register from Cop0.
        let mut temp_cause = self.sccp.read_reg(13);
        let mut temp_status = self.sccp.read_reg(12);

        // Bail out early for reset exceptions.
        if self.exception.exception_reason == MIPSExceptionReason::RESET {

            // Reset and exit method early.
            self.reset();
            self.sccp.reset();
            return true;
        }

        // Check the exception code and handle common behaviour accordingly.
        match self.exception.exception_reason {

            MIPSExceptionReason::ADEL |
            MIPSExceptionReason::ADES |
            MIPSExceptionReason::BP |
            MIPSExceptionReason::DBE |
            MIPSExceptionReason::IBE |
            MIPSExceptionReason::CPU |
            MIPSExceptionReason::INT |
            MIPSExceptionReason::OVF |
            MIPSExceptionReason::RI |
            MIPSExceptionReason::SYS => {

                // Mask out ExcCode and replace.
                temp_cause = (temp_cause & (0xFFFFFF83_u32 as i32)) |
                             ((self.exception.exception_reason as i32) << 2);

                // Set BD bit of cause register if necessary.
                if self.exception.is_in_branch_delay_slot {
                    temp_cause |= 0x80000000_u32 as i32;
                } else {
                    temp_cause &= 0x7FFFFFFF;
                }

                // Set EPC register.
                self.sccp.write_reg(14, self.exception.program_counter_origin, true);

                // Save KUp and IEp to KUo and IEo, KUc and IEc to KUp and IEp,
                // and reset KUc and IEc to 0.
                let temp = (temp_status & 0x0000000F) << 2;
                temp_status &= 0xFFFFFFC0_u32 as i32;
                temp_status |= temp;

                // Set PC to general exception vector and finish processing.
                self.program_counter = self.sccp.get_general_exception_vector();
            },

            _ => (),
        }

        // Specifically for ADEL/ADES/CPU exception types, handle the custom behaviour.
        match self.exception.exception_reason {

            // Handle address error exception (load or store operation).
            MIPSExceptionReason::ADEL | MIPSExceptionReason::ADES => {

                // Set bad virtual address register.
                self.sccp.write_reg(8, self.exception.bad_address, true);
            },

            // Handle co-processor unusable exception.
            MIPSExceptionReason::CPU => {

                // Set relevant bit of CE field in cause register.
                temp_cause = (temp_cause & (0xCFFFFFFF_u32 as i32)) |
                             (self.exception.co_processor_num << 28);
            },

            _ => (),
        }

        // Write cause and status registers back to Cop0.
        self.sccp.write_reg(13, temp_cause, true);
        self.sccp.write_reg(12, temp_status, true);

        // Reset exception object.
        self.exception.reset();

        // Signal that an exception has been processed.
        true
    }


    /// This function is for handling interrupts from within the execution loop.
    fn handle_interrupts(&mut self, bridge: &mut dyn CpuBridge) -> bool {

        // Increment all system interrupt counters.
        bridge.increment_interrupt_counters(self);

        // Get interrupt status and mask registers.
        let mut interrupt_status = {
            let word = bridge.read_word(self, 0x1F801070) & 0x7FF;
            self.swap_word_endianness(word)
        };
        let interrupt_mask = {
            let word = bridge.read_word(self, 0x1F801074) & 0x7FF;
            self.swap_word_endianness(word)
        };

        // Mask the interrupt status register.
        interrupt_status &= interrupt_mask;

        // Set bit 10 of COP0 cause register if needed.
        {
            let mut cause_register = self.sccp.read_reg(13);
            if interrupt_status != 0 {
                cause_register |= 0x400;
            } else {
                cause_register &= 0xFFFFFBFF_u32 as i32;
            }
            self.sccp.write_reg(13, cause_register, true);
        }

        // Get status reg and cause reg from COP0.
        let mut status_register = self.sccp.read_reg(12);
        let mut cause_register = self.sccp.read_reg(13);

        // Check if interrupts are enabled, if not then do nothing.
        if (status_register & 0x1) == 0x1 {

            // Use IM mask from status register to mask interrupt bits
            // in cause register.
            status_register &= 0x0000FF00;
            cause_register &= 0x0000FF00;
            cause_register &= status_register;

            // If resulting value is non-zero, trigger interrupt.
            if cause_register != 0 {
                self.exception.exception_reason = MIPSExceptionReason::INT;
                self.exception.program_counter_origin = self.program_counter;
                self.exception.is_in_branch_delay_slot = self.prev_was_branch;
                self.handle_exception();

                // Signal that interrupt occurred and was handled
                // by exception routine.
                return true;
            }
        }

        // Signal that no interrupt occurred.
        false
    }

    /// This function handles the ADD R3051 instruction.
    fn add_instruction(&mut self, instruction: i32) {

        // Get rs, rt and rd.
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;
        let rd = (instruction.logical_rshift(11) & 0x1F) as usize;

        // Add rs_val to rt_val.
        let rs_val = self.general_registers[rs];
        let rt_val = self.general_registers[rt];
        let result = rs_val.wrapping_add(rt_val);

        // Check for two's complement overflow.
        let sign_bit = 0x80000000_u32 as i32;
        if (rs_val & sign_bit) == (rt_val & sign_bit) &&
           (rs_val & sign_bit) != (result & sign_bit) {

            // Trigger exception.
            let mut temp_address = (self.program_counter as i64) & 0xFFFFFFFF;
            temp_address -= 4;
            self.exception.exception_reason = MIPSExceptionReason::OVF;
            self.exception.is_in_branch_delay_slot = self.prev_was_branch;
            self.exception.program_counter_origin = if self.exception.is_in_branch_delay_slot {
                temp_address as i32
            } else {
                self.program_counter
            };

            return;
        }

        // Store result.
        self.general_registers[rd] = result;
        self.general_registers[0] = 0;
    }

    /// This function handles the ADDI R3051 instruction.
    fn addi_instruction(&mut self, instruction: i32) {

        // Get rs, rt and immediate. Sign extend immediate if needed.
        let immediate = instruction.sign_extend(15);
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;

        // Add to rs_val.
        let rs_val = self.general_registers[rs];
        let result = rs_val.wrapping_add(immediate);

        // Check for two's complement overflow.
        let sign_bit = 0x80000000_u32 as i32;
        if (rs_val & sign_bit) == (immediate & sign_bit) &&
           (rs_val & sign_bit) != (result & sign_bit) {

            // Trigger exception.
            let mut temp_address = (self.program_counter as i64) & 0xFFFFFFFF;
            temp_address -= 4;
            self.exception.exception_reason = MIPSExceptionReason::OVF;
            self.exception.is_in_branch_delay_slot = self.prev_was_branch;
            self.exception.program_counter_origin = if self.exception.is_in_branch_delay_slot {
                temp_address as i32
            } else {
                self.program_counter
            };

            return;
        }

        // Store result.
        self.general_registers[rt] = result;
        self.general_registers[0] = 0;
    }

    /// This function handles the ADDIU R3051 instruction.
    fn addiu_instruction(&mut self, instruction: i32) {

        // Get rs, rt and immediate. Sign extend immediate if needed.
        let immediate = instruction.sign_extend(15);
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;

        // Add to rs_val.
        let result = ((self.general_registers[rs] as i64) & 0xFFFFFFFF) + (immediate as i64);

        // Store result.
        self.general_registers[rt] = result as i32;
        self.general_registers[0] = 0;
    }

    /// This function handles the ADDU R3051 instruction.
    fn addu_instruction(&mut self, instruction: i32) {

        // Get rs, rt and rd.
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;
        let rd = (instruction.logical_rshift(11) & 0x1F) as usize;

        // Add rs_val to rt_val.
        let result = ((self.general_registers[rs] as i64) & 0xFFFFFFFF) +
                     ((self.general_registers[rt] as i64) & 0xFFFFFFFF);

        // Store result.
        self.general_registers[rd] = result as i32;
        self.general_registers[0] = 0;
    }

    /// This function handles the AND R3051 instruction.
    fn and_instruction(&mut self, instruction: i32) {

        // Get rs, rt and rd.
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;
        let rd = (instruction.logical_rshift(11) & 0x1F) as usize;

        // Bitwise AND rs_val and rt_val, storing result.
        self.general_registers[rd] = self.general_registers[rs] & self.general_registers[rt];
        self.general_registers[0] = 0;
    }

    /// This function handles the ANDI R3051 instruction.
    fn andi_instruction(&mut self, instruction: i32) {

        // Get rs, rt and immediate.
        let immediate = instruction & 0xFFFF;
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;

        // Zero extending immediate is already done for us
        // so just AND with rsVal and store result.
        self.general_registers[rt] = immediate & self.general_registers[rs];
        self.general_registers[0] = 0;
    }

    /// This function handles the BC2F R3051 instruction.
    fn bc2f_instruction(&mut self, instruction: i32) {

        // Get immediate.
        let immediate = instruction & 0xFFFF;

        // Create target address.
        let mut target_address = ((self.program_counter as i64) & 0xFFFFFFFF) + 4;
        let mut offset = immediate << 2;
        if (offset & 0x20000) == 0x20000 {
            offset |= 0xFFFC0000_u32 as i32;
        }
        target_address += offset as i64;

        // Jump if COP2 condition line is false.
        if !self.gte.get_condition_line_status() {
            self.jump_address = target_address as i32;
            self.jump_pending = true;
        }
    }

   /// This function handles the BC2T R3051 instruction.
   fn bc2t_instruction(&mut self, instruction: i32) {

        // Get immediate.
        let immediate = instruction & 0xFFFF;

        // Create target address.
        let mut target_address = ((self.program_counter as i64) & 0xFFFFFFFF) + 4;
        let mut offset = immediate << 2;
        if (offset & 0x20000) == 0x20000 {
            offset |= 0xFFFC0000_u32 as i32;
        }
        target_address += offset as i64;

        // Jump if COP2 condition line is true.
        if self.gte.get_condition_line_status() {
            self.jump_address = target_address as i32;
            self.jump_pending = true;
        }
    }

    /// This function handles the BEQ R3051 instruction.
    fn beq_instruction(&mut self, instruction: i32) {

        // Get rs, rt and immediate.
        let immediate = instruction & 0xFFFF;
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;

        // Create target address.
        let mut target_address = ((self.program_counter as i64) & 0xFFFFFFFF) + 4;
        let mut offset = immediate << 2;
        if (offset & 0x20000) == 0x20000 {
            offset |= 0xFFFC0000_u32 as i32;
        }
        target_address += offset as i64;

        // Tells us this is a branch-type instruction.
        self.is_branch = true;

        // Jump if condition holds true.
        if self.general_registers[rs] == self.general_registers[rt] {
            self.jump_address = target_address as i32;
            self.jump_pending = true;
        }
    }

    /// This function handles the BGEZ R3051 instruction.
    fn bgez_instruction(&mut self, instruction: i32) {

        // Get rs and immediate.
        let immediate = instruction & 0xFFFF;
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;

        // Create target address.
        let mut target_address = ((self.program_counter as i64) & 0xFFFFFFFF) + 4;
        let mut offset = immediate << 2;
        if (offset & 0x20000) == 0x20000 {
            offset |= 0xFFFC0000_u32 as i32;
        }
        target_address += offset as i64;

        // This tells us this is a branch-type instruction.
        self.is_branch = true;

        // Jump if rs greater than or equal to 0.
        if self.general_registers[rs] >= 0 {
            self.jump_address = target_address as i32;
            self.jump_pending = true;
        }
    }

    /// This function handles the BGEZAL R3051 instruction.
    fn bgezal_instruction(&mut self, instruction: i32) {

        // Get rs and immediate.
        let immediate = instruction & 0xFFFF;
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;

        // Define target address and return address.
        let mut target_address = ((self.program_counter as i64) & 0xFFFFFFFF) + 8;
        let return_address = target_address;

        // Create target address.
        target_address -= 4;
        let mut offset = immediate << 2;
        if (offset & 0x20000) == 0x20000 {
            offset |= 0xFFFC0000_u32 as i32;
        }
        target_address += offset as i64;

        // This tells us this is a branch-type instruction.
        self.is_branch = true;

        // Jump if rs greater than or equal to 0, and save return address in r31.
        if self.general_registers[rs] >= 0 {
            self.jump_address = target_address as i32;
            self.jump_pending = true;
            self.general_registers[31] = return_address as i32;
            self.general_registers[0] = 0;
        }
    }

    /// This function handles the BGTZ R3051 instruction.
    fn bgtz_instruction(&mut self, instruction: i32) {

        // Get rs and immediate.
        let immediate = instruction & 0xFFFF;
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;

        // Create target address.
        let mut target_address = ((self.program_counter as i64) & 0xFFFFFFFF) + 4;
        let mut offset = immediate << 2;
        if (offset & 0x20000) == 0x20000 {
            offset |= 0xFFFC0000_u32 as i32;
        }
        target_address += offset as i64;

        // This tells us this is a branch-type instruction.
        self.is_branch = true;

        // Jump if rs greater than 0.
        if self.general_registers[rs] > 0 {
            self.jump_address = target_address as i32;
            self.jump_pending = true;
        }
    }

    /// This function handles the BLEZ R3051 instruction.
    fn blez_instruction(&mut self, instruction: i32) {

        // Get rs and immediate.
        let immediate = instruction & 0xFFFF;
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;

        // Create target address.
        let mut target_address = ((self.program_counter as i64) & 0xFFFFFFFF) + 4;
        let mut offset = immediate << 2;
        if (offset & 0x20000) == 0x20000 {
            offset |= 0xFFFC0000_u32 as i32;
        }
        target_address += offset as i64;

        // This tells us this is a branch-type instruction.
        self.is_branch = true;

        // Jump if rs less than or equal to 0.
        if self.general_registers[rs] <= 0 {
            self.jump_address = target_address as i32;
            self.jump_pending = true;
        }
    }

    /// This function handles the BLTZ R3051 instruction.
    fn bltz_instruction(&mut self, instruction: i32) {

        // Get rs and immediate.
        let immediate = instruction & 0xFFFF;
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;

        // Create target address.
        let mut target_address = ((self.program_counter as i64) & 0xFFFFFFFF) + 4;
        let mut offset = immediate << 2;
        if (offset & 0x20000) == 0x20000 {
            offset |= 0xFFFC0000u32 as i32;
        }
        target_address += offset as i64;

        // This tells us this is a branch-type instruction.
        self.is_branch = true;

        // Jump if rs less than 0.
        if self.general_registers[rs] < 0 {
            self.jump_address = target_address as i32;
            self.jump_pending = true;
        }
    }

    /// This function handles the BLTZAL R3051 instruction.
    fn bltzal_instruction(&mut self, instruction: i32) {

        // Get rs and immediate.
        let immediate = instruction & 0xFFFF;
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;

        // Define target address and return address.
        let mut target_address = ((self.program_counter as i64) & 0xFFFFFFFF) + 8;
        let return_address = target_address;

        // Create target address.
        target_address -= 4;
        let mut offset = immediate << 2;
        if (offset & 0x20000) == 0x20000 {
            offset |= 0xFFFC0000_u32 as i32;
        }
        target_address += offset as i64;

        // This tells us this is a branch-type instruction.
        self.is_branch = true;

        // Jump if rs less than 0, and save return address in r31.
        if self.general_registers[rs] < 0 {
            self.jump_address = target_address as i32;
            self.jump_pending = true;
            self.general_registers[31] = return_address as i32;
            self.general_registers[0] = 0;
        }
    }

    /// This function handles the BNE R3051 instruction.
    fn bne_instruction(&mut self, instruction: i32) {

        // Get rs, rt and immediate.
        let immediate = instruction & 0xFFFF;
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;

        // Create target address.
        let mut target_address = ((self.program_counter as i64) & 0xFFFFFFFF) + 4;
        let mut offset = immediate << 2;
        if (offset & 0x20000) == 0x20000 {
            offset |= 0xFFFC0000_u32 as i32;
        }
        target_address += offset as i64;

        // Tells us this is a branch-type instruction.
        self.is_branch = true;

        // Jump if condition holds false.
        if self.general_registers[rs] != self.general_registers[rt] {
            self.jump_address = target_address as i32;
            self.jump_pending = true;
        }
    }

    /// This function handles the BREAK R3051 instruction.
    fn break_instruction(&mut self) {

        // Trigger Breakpoint Exception.
        let mut temp_address = (self.program_counter as i64) & 0xFFFFFFFF;
        temp_address -= 4;
        self.exception.exception_reason = MIPSExceptionReason::BP;
        self.exception.is_in_branch_delay_slot = self.prev_was_branch;
        self.exception.program_counter_origin = if self.exception.is_in_branch_delay_slot {
            temp_address as i32
        } else {
            self.program_counter
        };
    }

    /// This function handles the CF2 R3051 instruction.
    fn cf2_instruction(&mut self, instruction: i32) {

        // Get rt and rd.
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;
        let rd = instruction.logical_rshift(11) & 0x1F;

        // Move from COP2 control reg rd to CPU reg rt.
        self.general_registers[rt] = self.gte.read_control_reg(rd);
        self.general_registers[0] = 0;
    }

    /// This function handles the CT2 R3051 instruction.
    fn ct2_instruction(&mut self, instruction: i32) {

        // Get rt and rd.
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;
        let rd = instruction.logical_rshift(11) & 0x1F;

        // Move from CPU reg rt to COP2 control reg rd.
        self.gte.write_control_reg(rd, self.general_registers[rt], false);
    }

    /// This function handles the DIV R3051 instruction.
    fn div_instruction(&mut self, instruction: i32) {

        // Get rs and rt.
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;

        // Divide rs by rt as signed values.
        let rs_val = self.general_registers[rs] as i64;
        let rt_val = self.general_registers[rt] as i64;
        let mut quotient = 0;
        let mut remainder = 0;

        if rt_val != 0 {
            quotient = rs_val / rt_val;
            remainder = rs_val % rt_val;
        } else {
            quotient = 0xFFFFFFFF;
            remainder = rs_val;
        }

        // Store result.
        self.hi_reg = remainder as i32;
        self.lo_reg = quotient as i32;
    }

    /// This function handles the DIVU R3051 instruction.
    fn divu_instruction(&mut self, instruction: i32) {

        // Get rs and rt
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;

        // Divide rs by rt as unsigned values.
        let rs_val = (self.general_registers[rs] as i64) & 0xFFFFFFFF;
        let rt_val = (self.general_registers[rt] as i64) & 0xFFFFFFFF;
        let mut quotient = 0;
        let mut remainder = 0;

        if rt_val != 0 {
            quotient = rs_val / rt_val;
            remainder = rs_val % rt_val;
        } else {
            quotient = 0xFFFFFFFF;
            remainder = rs_val;
        }

        // Store result.
        self.hi_reg = remainder as i32;
        self.lo_reg = quotient as i32;
    }

    /// This function handles the J R3051 instruction.
    fn j_instruction(&mut self, instruction: i32) {

        // Get target.
        let target = instruction & 0x3FFFFFF;

        // Create address to jump to.
        self.jump_address = (target << 2) | (self.program_counter & 0xF0000000_u32 as i32);
        self.jump_pending = true;
        self.is_branch = true;
    }

    /// This function handles the JAL R3051 instruction.
    fn jal_instruction(&mut self, instruction: i32) {

        // Get target.
        let target = instruction & 0x3FFFFFF;

        // Create address to jump to, and place address of instruction
        // after delay slot in r31.
        self.jump_address = (target << 2) | (self.program_counter & 0xF0000000_u32 as i32);
        self.jump_pending = true;
        self.is_branch = true;
        let new_address = ((self.program_counter as i64) & 0xFFFFFFFF) + 8;
        self.general_registers[31] = new_address as i32;
        self.general_registers[0] = 0;
    }

    /// This function handles the JALR R3051 instruction.
    fn jalr_instruction(&mut self, instruction: i32) {

        // Get rs and rd.
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;
        let rd = (instruction.logical_rshift(11) & 0x1F) as usize;

        // Jump to rs value and place address of instruction
        // after delay slot into rd.
        self.jump_address = self.general_registers[rs];
        self.jump_pending = true;
        self.is_branch = true;
        let new_address = ((self.program_counter as i64) & 0xFFFFFFFF) + 8;
        self.general_registers[rd] = new_address as i32;
        self.general_registers[0] = 0;
    }

    /// This function handles the JR R3051 instruction.
    fn jr_instruction(&mut self, instruction: i32) {

        // Get rs.
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;

        // Jump to rs value.
        self.jump_address = self.general_registers[rs];
        self.jump_pending = true;
        self.is_branch = true;
    }

    /// This function handles the LB R3051 instruction.
    fn lb_instruction(&mut self, bridge: &mut dyn CpuBridge, instruction: i32) {

        // Get rs, rt and immediate.
        let immediate = instruction.sign_extend(15);
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;

        // Calculate address.
        let address = ((self.general_registers[rs] as i64) & 0xFFFFFFFF) + (immediate as i64);

        // Check if address is allowed, trigger exception if not.
        if !self.sccp.is_address_allowed(address as i32) {
            let mut temp_address = (self.program_counter as i64) & 0xFFFFFFFF;
            temp_address -= 4;
            self.exception.bad_address = address as i32;
            self.exception.exception_reason = MIPSExceptionReason::ADEL;
            self.exception.is_in_branch_delay_slot = self.prev_was_branch;
            self.exception.program_counter_origin = if self.exception.is_in_branch_delay_slot {
                temp_address as i32
            } else {
                self.program_counter
            };

            return;
        }

        // Load byte and sign extend.
        let temp_byte = self.read_data_value(
            bridge, R3051Width::BYTE, address as i32
        ).sign_extend(7);

        // Write byte to correct register
        self.general_registers[rt] = temp_byte;
        self.general_registers[0] = 0;
    }

    /// This function handles the LBU R3051 instruction.
    fn lbu_instruction(&mut self, bridge: &mut dyn CpuBridge, instruction: i32) {

        // Get rs, rt and immediate.
        let immediate = instruction.sign_extend(15);
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;

        // Calculate address.
        let address = ((self.general_registers[rs] as i64) & 0xFFFFFFFF) + (immediate as i64);

        // Check if address is allowed, trigger exception if not.
        if !self.sccp.is_address_allowed(address as i32) {
            let mut temp_address = (self.program_counter as i64) & 0xFFFFFFFF;
            temp_address -= 4;
            self.exception.bad_address = address as i32;
            self.exception.exception_reason = MIPSExceptionReason::ADEL;
            self.exception.is_in_branch_delay_slot = self.prev_was_branch;
            self.exception.program_counter_origin = if self.exception.is_in_branch_delay_slot {
                temp_address as i32
            } else {
                self.program_counter
            };

            return;
        }

        // Load byte and zero extend.
        let temp_byte = 0xFF & self.read_data_value(bridge, R3051Width::BYTE, address as i32);

        // Write byte to correct register.
        self.general_registers[rt] = temp_byte;
        self.general_registers[0] = 0;
    }

    /// This function handles the LH R3051 instruction.
    fn lh_instruction(&mut self, bridge: &mut dyn CpuBridge, instruction: i32) {

        // Get rs, rt and immediate.
        let immediate = instruction.sign_extend(15);
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;

        // Calculate address.
        let address = ((self.general_registers[rs] as i64) & 0xFFFFFFFF) + (immediate as i64);

        // Check if address is allowed and half-word aligned, trigger
        // exception if not.
        if !self.sccp.is_address_allowed(address as i32) || address % 2 != 0 {
            let mut temp_address = (self.program_counter as i64) & 0xFFFFFFFF;
            temp_address -= 4;
            self.exception.bad_address = address as i32;
            self.exception.exception_reason = MIPSExceptionReason::ADEL;
            self.exception.is_in_branch_delay_slot = self.prev_was_branch;
            self.exception.program_counter_origin = if self.exception.is_in_branch_delay_slot {
                temp_address as i32
            } else {
                self.program_counter
            };

            return;
        }

        // Load half-word and swap endianness, sign extend.
        let mut temp_half_word = 0xFFFF & self.read_data_value(bridge, R3051Width::HALFWORD, address as i32);
        temp_half_word = ((temp_half_word << 8) & 0xFF00) | temp_half_word.logical_rshift(8);
        temp_half_word = temp_half_word.sign_extend(15);

        // Write half-word to correct register.
        self.general_registers[rt] = temp_half_word;
        self.general_registers[0] = 0;
    }

    /// This function handles the LHU R3051 instruction.
    fn lhu_instruction(&mut self, bridge: &mut dyn CpuBridge, instruction: i32) {

        // Get rs, rt and immediate.
        let immediate = instruction.sign_extend(15);
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;

        // Calculate address.
        let address = ((self.general_registers[rs] as i64) & 0xFFFFFFFF) + (immediate as i64);

        // Check if address is allowed and half-word aligned, trigger
        // exception if not.
        if !self.sccp.is_address_allowed(address as i32) || address % 2 != 0 {
            let mut temp_address = (self.program_counter as i64) & 0xFFFFFFFF;
            temp_address -= 4;
            self.exception.bad_address = address as i32;
            self.exception.exception_reason = MIPSExceptionReason::ADEL;
            self.exception.is_in_branch_delay_slot = self.prev_was_branch;
            self.exception.program_counter_origin = if self.exception.is_in_branch_delay_slot {
                temp_address as i32
            } else {
                self.program_counter
            };

            return;
        }

        // Load half-word and swap endianness, zero extend.
        let mut temp_half_word = 0xFFFF & self.read_data_value(bridge, R3051Width::HALFWORD, address as i32);

        // Swap byte order.
        temp_half_word = ((temp_half_word << 8) & 0xFF00) | temp_half_word.logical_rshift(8);

        // Write half-word to correct register.
        self.general_registers[rt] = temp_half_word;
        self.general_registers[0] = 0;
    }

    /// This function handles the LUI R3051 instruction.
    fn lui_instruction(&mut self, instruction: i32) {

       // Get rt and immediate.
       let immediate = instruction & 0xFFFF;
       let rt = (instruction.logical_rshift(16) & 0x1F) as usize;

       // Shift immediate left by 16 bits (leaving least significant
       // 16 bits as zeroes) and store result.
       self.general_registers[rt] = immediate << 16;
       self.general_registers[0] = 0;
    }

    /// This function handles the LW R3051 instruction.
    fn lw_instruction(&mut self, bridge: &mut dyn CpuBridge, instruction: i32) {

        // Get rs, rt and immediate.
        let immediate = instruction.sign_extend(15);
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;

        // Calculate address.
        let address = ((self.general_registers[rs] as i64) & 0xFFFFFFFF) + (immediate as i64);

        // Check if address is allowed and word aligned, trigger exception if not.
        if !self.sccp.is_address_allowed(address as i32) || address % 4 != 0 {
            let mut temp_address = (self.program_counter as i64) & 0xFFFFFFFF;
            temp_address -= 4;
            self.exception.bad_address = address as i32;
            self.exception.exception_reason = MIPSExceptionReason::ADEL;
            self.exception.is_in_branch_delay_slot = self.prev_was_branch;
            self.exception.program_counter_origin = if self.exception.is_in_branch_delay_slot {
                temp_address as i32
            } else {
                self.program_counter
            };

            return;
        }

        // Load word.
        let mut temp_word = self.read_data_value(bridge, R3051Width::WORD, address as i32);

        // Swap byte order.
        temp_word = self.swap_word_endianness(temp_word);

        // Write word to correct register.
        self.general_registers[rt] = temp_word;
        self.general_registers[0] = 0;
    }

    /// This function handles the LWC2 R3051 instruction.
    fn lwc2_instruction(&mut self, bridge: &mut dyn CpuBridge, instruction: i32) {

        // Get rs, rt and immediate.
        let immediate = instruction.sign_extend(15);
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;
        let rt = instruction.logical_rshift(16) & 0x1F;

        // Calculate address.
        let address = ((self.general_registers[rs] as i64) & 0xFFFFFFFF) + (immediate as i64);

        // Check if address is allowed and word aligned, trigger exception if not.
        if !self.sccp.is_address_allowed(address as i32) || address % 4 != 0 {
            let mut temp_address = (self.program_counter as i64) & 0xFFFFFFFF;
            temp_address -= 4;
            self.exception.bad_address = address as i32;
            self.exception.exception_reason = MIPSExceptionReason::ADEL;
            self.exception.is_in_branch_delay_slot = self.prev_was_branch;
            self.exception.program_counter_origin = if self.exception.is_in_branch_delay_slot {
                temp_address as i32
            } else {
                self.program_counter
            };

            return;
        }

        // Load word.
        let mut temp_word = self.read_data_value(bridge, R3051Width::WORD, address as i32);

        // Swap byte order.
        temp_word = self.swap_word_endianness(temp_word);

        // Write word to correct COP2 data register.
        self.gte.write_data_reg(rt, temp_word, false);
    }

    /// This function handles the LWL R3051 instruction.
    fn lwl_instruction(&mut self, bridge: &mut dyn CpuBridge, instruction: i32) {

        // Get rs, rt and immediate.
        let immediate = instruction.sign_extend(15);
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;

        // Calculate address.
        let address = ((self.general_registers[rs] as i64) & 0xFFFFFFFF) + (immediate as i64);

        // Check if address is allowed, trigger exception if not.
        if !self.sccp.is_address_allowed(address as i32) {
            let mut temp_address = (self.program_counter as i64) & 0xFFFFFFFF;
            temp_address -= 4;
            self.exception.bad_address = address as i32;
            self.exception.exception_reason = MIPSExceptionReason::ADEL;
            self.exception.is_in_branch_delay_slot = self.prev_was_branch;
            self.exception.program_counter_origin = if self.exception.is_in_branch_delay_slot {
                temp_address as i32
            } else {
                self.program_counter
            };

            return;
        }

        // Align address, fetch word, and store shift index.
        let temp_address = (address & 0xFFFFFFFC) as i32;
        let byte_shift_index = (!address & 0x3) as i32;
        let mut temp_word = self.read_data_value(bridge, R3051Width::WORD, temp_address);

        // Swap byte order.
        temp_word = self.swap_word_endianness(temp_word);

        // Shift word value left by required amount.
        temp_word <<= byte_shift_index * 8;

        // Fetch rt contents, and calculate mask.
        let mut temp_rt_val = self.general_registers[rt];
        let mask = !((0xFFFFFFFF_u32 as i32) << (byte_shift_index * 8));
        temp_rt_val &= mask;

        // Merge contents.
        temp_word |= temp_rt_val;

        // Write word to correct register.
        self.general_registers[rt] = temp_word;
        self.general_registers[0] = 0;
    }

    /// This function handles the LWR R3051 instruction.
    fn lwr_instruction(&mut self, bridge: &mut dyn CpuBridge, instruction: i32) {

        // Get rs, rt and immediate.
        let immediate = instruction.sign_extend(15);
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;

        // Calculate address.
        let address = ((self.general_registers[rs] as i64) & 0xFFFFFFFF) + (immediate as i64);

        // Check if address is allowed, trigger exception if not.
        if !self.sccp.is_address_allowed(address as i32) {
            let mut temp_address = (self.program_counter as i64) & 0xFFFFFFFF;
            temp_address -= 4;
            self.exception.bad_address = address as i32;
            self.exception.exception_reason = MIPSExceptionReason::ADEL;
            self.exception.is_in_branch_delay_slot = self.prev_was_branch;
            self.exception.program_counter_origin = if self.exception.is_in_branch_delay_slot {
                temp_address as i32
            } else {
                self.program_counter
            };

            return;
        }

        // Align address, fetch word, and store shift index.
        let temp_address = (address & 0xFFFFFFFC) as i32;
        let byte_shift_index = (address & 0x3) as i32;
        let mut temp_word = self.read_data_value(bridge, R3051Width::WORD, temp_address);

        // Swap byte order.
        temp_word = self.swap_word_endianness(temp_word);

        // Shift word value right by required amount.
        temp_word = temp_word.logical_rshift(byte_shift_index * 8);

        // Fetch rt contents, and calculate mask.
        let mut temp_rt_val = self.general_registers[rt];
        let mask = !((0xFFFFFFFF_u32 as i32).logical_rshift(byte_shift_index * 8));
        temp_rt_val &= mask;

        // Merge contents.
        temp_word |= temp_rt_val;

        // Write word to correct register.
        self.general_registers[rt] = temp_word;
        self.general_registers[0] = 0;
    }

    /// This function handles the MF0 R3051 instruction.
    fn mf0_instruction(&mut self, instruction: i32) {

        // Get rt and rd.
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;
        let rd = instruction.logical_rshift(11) & 0x1F;

        // Check if rd is any of the following and trigger exception if so.
        match rd {

            0 | 1 | 2 | 4 | 10 => {

                // Trigger exception.
                let mut temp_address = (self.program_counter as i64) & 0xFFFFFFFF;
                temp_address -= 4;
                self.exception.exception_reason = MIPSExceptionReason::RI;
                self.exception.is_in_branch_delay_slot = self.prev_was_branch;
                self.exception.program_counter_origin = if self.exception.is_in_branch_delay_slot {
                    temp_address as i32
                } else {
                    self.program_counter
                };

                return;
            },

            _ => (),
        }

        // Move COP0 reg rd to CPU reg rt.
        self.general_registers[rt] = self.sccp.read_reg(rd);
        self.general_registers[0] = 0;
    }

    /// This function handles the MF2 R3051 instruction.
    fn mf2_instruction(&mut self, instruction: i32) {

        // Get rt and rd.
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;
        let rd = instruction.logical_rshift(11) & 0x1F;

        // Move from COP2 data reg rd to CPU reg rt.
        self.general_registers[rt] = self.gte.read_data_reg(rd);
        self.general_registers[0] = 0;
    }

    /// This function handles the MFHI R3051 instruction.
    fn mfhi_instruction(&mut self, instruction: i32) {

        // Get rd.
        let rd = (instruction.logical_rshift(11) & 0x1F) as usize;

        // Move Hi to rd.
        self.general_registers[rd] = self.hi_reg;
        self.general_registers[0] = 0;
    }

    /// This function handles the MFLO R3051 instruction.
    fn mflo_instruction(&mut self, instruction: i32) {

        // Get rd.
        let rd = (instruction.logical_rshift(11) & 0x1F) as usize;

        // Move Lo to rd.
        self.general_registers[rd] = self.lo_reg;
        self.general_registers[0] = 0;
    }

    /// This function handles the MT0 R3051 instruction.
    fn mt0_instruction(&mut self, instruction: i32) {

        // Get rt and rd.
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;
        let rd = instruction.logical_rshift(11) & 0x1F;

        // Move CPU reg rt to COP0 reg rd.
        self.sccp.write_reg(rd, self.general_registers[rt], false);
    }

    /// This function handles the MT2 R3051 instruction.
    fn mt2_instruction(&mut self, instruction: i32) {

        // Get rt and rd.
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;
        let rd = instruction.logical_rshift(11) & 0x1F;

        // Move from CPU reg rt to COP2 data reg rd.
        self.gte.write_data_reg(rd, self.general_registers[rt], false);
    }

    /// This function handles the MTHI R3051 instruction.
    fn mthi_instruction(&mut self, instruction: i32) {

        // Get rs.
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;

        // Move rs to Hi.
        self.hi_reg = self.general_registers[rs];
    }

    /// This function handles the MTLO R3051 instruction.
    fn mtlo_instruction(&mut self, instruction: i32) {

        // Get rs.
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;

        // Move rs to Lo,
        self.lo_reg = self.general_registers[rs];
    }

    /// This function handles the MULT R3051 instruction.
    fn mult_instruction(&mut self, instruction: i32) {

        // Get rs and rt.
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;

        // Multiply rs and rt as signed values.
        let rs_val = self.general_registers[rs] as i64;
        let rt_val = self.general_registers[rt] as i64;
        let result = rs_val * rt_val;

        // Store result.
        self.hi_reg = result.logical_rshift(32) as i32;
        self.lo_reg = result as i32;
    }

    /// This function handles the MULTU R3051 instruction.
    fn multu_instruction(&mut self, instruction: i32) {

        // Get rs and rt.
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;

        // Multiply rs and rt as unsigned values.
        let rs_val = (self.general_registers[rs] as i64) & 0xFFFFFFFF;
        let rt_val = (self.general_registers[rt] as i64) & 0xFFFFFFFF;
        let result = rs_val * rt_val;

        // Store result.
        self.hi_reg = result.logical_rshift(32) as i32;
        self.lo_reg = result as i32;
    }

    /// This function handles the NOR R3051 instruction.
    fn nor_instruction(&mut self, instruction: i32) {

        // Get rs, rt and rd.
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;
        let rd = (instruction.logical_rshift(11) & 0x1F) as usize;

        // Bitwise NOR rs_val and rt_val, storing result.
        self.general_registers[rd] = !(self.general_registers[rs] | self.general_registers[rt]);
        self.general_registers[0] = 0;
    }

    /// This function handles the OR R3051 instruction.
    fn or_instruction(&mut self, instruction: i32) {

        // Get rs, rt and rd.
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;
        let rd = (instruction.logical_rshift(11) & 0x1F) as usize;

        // Bitwise OR rs_val and rt_val, storing result.
        self.general_registers[rd] = self.general_registers[rs] | self.general_registers[rt];
        self.general_registers[0] = 0;
    }

    /// This function handles the ORI R3051 instruction.
    fn ori_instruction(&mut self, instruction: i32) {

        // Get rs, rt and immediate.
        let immediate = instruction & 0xFFFF;
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;

        // Zero extending immediate is already done for us
        // so just OR with rs_val and store result.
        self.general_registers[rt] = immediate | self.general_registers[rs];
        self.general_registers[0] = 0;
    }

    /// This function handles the RFE R3051 instruction.
    fn rfe_instruction(&mut self) {
        self.sccp.rfe();
    }

    /// This function handles the SB R3051 instruction.
    fn sb_instruction(&mut self, bridge: &mut dyn CpuBridge, instruction: i32) {

        // Get rs, rt and immediate.
        let immediate = instruction.sign_extend(15);
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;

        // Calculate address.
        let address = ((self.general_registers[rs] as i64) & 0xFFFFFFFF) + (immediate as i64);

        // Check if address is allowed, trigger exception if not.
        if !self.sccp.is_address_allowed(address as i32) {
            let mut temp_address = (self.program_counter as i64) & 0xFFFFFFFF;
            temp_address -= 4;
            self.exception.bad_address = address as i32;
            self.exception.exception_reason = MIPSExceptionReason::ADES;
            self.exception.is_in_branch_delay_slot = self.prev_was_branch;
            self.exception.program_counter_origin = if self.exception.is_in_branch_delay_slot {
                temp_address as i32
            } else {
                self.program_counter
            };

            return;
        }

        // Load byte from register and write to memory.
        let temp_byte = 0xFF & self.general_registers[rt];
        self.write_data_value(bridge, R3051Width::BYTE, address as i32, temp_byte);
    }

    /// This function handles the SH R3051 instruction.
    fn sh_instruction(&mut self, bridge: &mut dyn CpuBridge, instruction: i32) {

        // Get rs, rt and immediate.
        let immediate = instruction.sign_extend(15);
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;

        // Calculate address.
        let address = ((self.general_registers[rs] as i64) & 0xFFFFFFFF) + (immediate as i64);

        // Check if address is allowed and half-word aligned, trigger
        // exception if not.
        if !self.sccp.is_address_allowed(address as i32) || address % 2 != 0 {
            let mut temp_address = (self.program_counter as i64) & 0xFFFFFFFF;
            temp_address -= 4;
            self.exception.bad_address = address as i32;
            self.exception.exception_reason = MIPSExceptionReason::ADES;
            self.exception.is_in_branch_delay_slot = self.prev_was_branch;
            self.exception.program_counter_origin = if self.exception.is_in_branch_delay_slot {
                temp_address as i32
            } else {
                self.program_counter
            };

            return;
        }

        // Load half-word from register and swap endianness, then write to memory
        // (checking for exceptions and stalls).
        let mut temp_half_word = 0xFFFF & self.general_registers[rt];

        // Swap byte order and write to memory.
        temp_half_word = ((temp_half_word << 8) & 0xFF00) | temp_half_word.logical_rshift(8);
        self.write_data_value(bridge, R3051Width::HALFWORD, address as i32, temp_half_word);
    }

    /// This function handles the SLL R3051 instruction.
    fn sll_instruction(&mut self, instruction: i32) {

        // Get rt, rd and shamt.
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;
        let rd = (instruction.logical_rshift(11) & 0x1F) as usize;
        let shamt = instruction.logical_rshift(6) & 0x1F;

        // Shift rt value left by shamt bits, inserting zeroes
        // into low order bits, then store result.
        self.general_registers[rd] = self.general_registers[rt] << shamt;
        self.general_registers[0] = 0;
    }

    /// This function handles the SLLV R3051 instruction.
    fn sllv_instruction(&mut self, instruction: i32) {

        // Get rs, rt and rd.
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;
        let rd = (instruction.logical_rshift(11) & 0x1F) as usize;

        // Shift rt value left by (lowest 5 bits of rs value),
        // inserting zeroes into low order bits, then
        // store result.
        self.general_registers[rd] = self.general_registers[rt] << (self.general_registers[rs] & 0x1F);
        self.general_registers[0] = 0;
    }

    /// This function handles the SLT R3051 instruction.
    fn slt_instruction(&mut self, instruction: i32) {

        // Get rs, rt and rd.
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;
        let rd = (instruction.logical_rshift(11) & 0x1F) as usize;

        // Compare rs_val and rt_val, storing result.
        self.general_registers[rd] = if self.general_registers[rs] < self.general_registers[rt] {
            1
        } else {
            0
        };
        self.general_registers[0] = 0;
    }

    /// This function handles the SLTI R3051 instruction.
    fn slti_instruction(&mut self, instruction: i32) {

        // Get rs, rt and sign-extended immediate.
        let immediate = instruction.sign_extend(15);
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;

        // Store result.
        self.general_registers[rt] = if self.general_registers[rs] < immediate {
            1
        } else {
            0
        };
        self.general_registers[0] = 0;
    }

    /// This function handles the SLTIU R3051 instruction.
    fn sltiu_instruction(&mut self, instruction: i32) {

        // Get rs, rt and sign-extended immediate.
        let immediate = (instruction.sign_extend(15) as i64) & 0xFFFFFFFF;
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;

        // Treat rs_val as unsigned.
        let temp_rs_val = (self.general_registers[rs] as i64) & 0xFFFFFFFF;

        // Store result.
        self.general_registers[rt] = if temp_rs_val < immediate {
            1
        } else {
            0
        };
        self.general_registers[0] = 0;
    }

    /// This function handles the SLTU R3051 instruction.
    fn sltu_instruction(&mut self, instruction: i32) {

        // Get rs, rt and rd.
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;
        let rd = (instruction.logical_rshift(11) & 0x1F) as usize;

        // Compare rs_val and rt_val as unsigned values, storing result.
        self.general_registers[rd] = if ((self.general_registers[rs] as i64) & 0xFFFFFFFF) <
                                        ((self.general_registers[rt] as i64) & 0xFFFFFFFF) {
            1
        } else {
            0
        };
        self.general_registers[0] = 0;
    }

    /// This function handles the SRA R3051 instruction.
    fn sra_instruction(&mut self, instruction: i32) {

        // Get rt, rd and shamt.
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;
        let rd = (instruction.logical_rshift(11) & 0x1F) as usize;
        let shamt = instruction.logical_rshift(6) & 0x1F;

        // Shift rt value right by shamt bits, sign extending
        // high order bits, then store result.
        self.general_registers[rd] = self.general_registers[rt] >> shamt;
        self.general_registers[0] = 0;
    }

    /// This function handles the SRAV R3051 instruction.
    fn srav_instruction(&mut self, instruction: i32) {

        // Get rs, rt and rd.
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;
        let rd = (instruction.logical_rshift(11) & 0x1F) as usize;

        // Shift rt value right by (lowest 5 bits of rs value),
        // sign extending high order bits, then store result.
        self.general_registers[rd] = self.general_registers[rt] >> (self.general_registers[rs] & 0x1F);
        self.general_registers[0] = 0;
    }

    /// This function handles the SRL R3051 instruction.
    fn srl_instruction(&mut self, instruction: i32) {

        // Get rt, rd and shamt.
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;
        let rd = (instruction.logical_rshift(11) & 0x1F) as usize;
        let shamt = instruction.logical_rshift(6) & 0x1F;

        // Shift rt value right by shamt bits, inserting zeroes
        // into high order bits, then store result.
        self.general_registers[rd] = self.general_registers[rt].logical_rshift(shamt);
        self.general_registers[0] = 0;
    }

    /// This function handles the SRLV R3051 instruction.
    fn srlv_instruction(&mut self, instruction: i32) {

        // Get rs, rt and rd.
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;
        let rd = (instruction.logical_rshift(11) & 0x1F) as usize;

        // Shift rt value right by (lowest 5 bits of rs value),
        // inserting zeroes into high order bits, then
        // store result.
        self.general_registers[rd] =
            self.general_registers[rt].logical_rshift(self.general_registers[rs] & 0x1F);
        self.general_registers[0] = 0;
    }

    /// This function handles the SUB R3051 instruction.
    fn sub_instruction(&mut self, instruction: i32) {

        // Get rs, rt and rd.
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;
        let rd = (instruction.logical_rshift(11) & 0x1F) as usize;

        // Subtract rt_val from rs_val.
        let rs_val = self.general_registers[rs];
        let rt_val = self.general_registers[rt];
        let result = rs_val.wrapping_sub(rt_val);

        // Check for two's complement overflow.
        let sign_bit = 0x80000000_u32 as i32;
        if (rs_val & sign_bit) != (rt_val & sign_bit) &&
           (rs_val & sign_bit) != (result & sign_bit) {

            // Trigger exception.
            let mut temp_address = (self.program_counter as i64) & 0xFFFFFFFF;
            temp_address -= 4;
            self.exception.exception_reason = MIPSExceptionReason::OVF;
            self.exception.is_in_branch_delay_slot = self.prev_was_branch;
            self.exception.program_counter_origin = if self.exception.is_in_branch_delay_slot {
                temp_address as i32
            } else {
                self.program_counter
            };

            return;
        }

        // Store result.
        self.general_registers[rd] = result;
        self.general_registers[0] = 0;
    }

    /// This function handles the SUBU R3051 instruction.
    fn subu_instruction(&mut self, instruction: i32) {

        // Get rs, rt and rd.
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;
        let rd = (instruction.logical_rshift(11) & 0x1F) as usize;

        // Subtract rt_val from rs_val.
        let result = ((self.general_registers[rs] as i64) & 0xFFFFFFFF) -
            ((self.general_registers[rt] as i64) & 0xFFFFFFFF);

        // Store result.
        self.general_registers[rd] = result as i32;
        self.general_registers[0] = 0;
    }

    /// This function handles the SW R3051 instruction.
    fn sw_instruction(&mut self, bridge: &mut dyn CpuBridge, instruction: i32) {

        // Get rs, rt and immediate.
        let immediate = instruction.sign_extend(15);
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;

        // Calculate address.
        let address = ((self.general_registers[rs] as i64) & 0xFFFFFFFF) + (immediate as i64);

        // Check if address is allowed and word aligned, trigger exception if not.
        if !self.sccp.is_address_allowed(address as i32) || address % 4 != 0 {
            let mut temp_address = (self.program_counter as i64) & 0xFFFFFFFF;
            temp_address -= 4;
            self.exception.bad_address = address as i32;
            self.exception.exception_reason = MIPSExceptionReason::ADES;
            self.exception.is_in_branch_delay_slot = self.prev_was_branch;
            self.exception.program_counter_origin = if self.exception.is_in_branch_delay_slot {
                temp_address as i32
            } else {
                self.program_counter
            };

            return;
        }

        // Load word from register, set byte order and write to memory
        // (checking for exceptions and stalls).
        let mut temp_word = self.general_registers[rt];

        // Swap byte order.
        temp_word = self.swap_word_endianness(temp_word);

        self.write_data_value(bridge, R3051Width::WORD, address as i32, temp_word);
    }

    /// This function handles the SWC2 R3051 instruction.
    fn swc2_instruction(&mut self, bridge: &mut dyn CpuBridge, instruction: i32) {

        // Get rs, rt and immediate.
        let immediate = instruction.sign_extend(15);
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;
        let rt = instruction.logical_rshift(16) & 0x1F;

        // Calculate address.
        let address = ((self.general_registers[rs] as i64) & 0xFFFFFFFF) + (immediate as i64);

        // Check if address is allowed and word aligned, trigger exception if not.
        if !self.sccp.is_address_allowed(address as i32) || address % 4 != 0 {
            let mut temp_address = (self.program_counter as i64) & 0xFFFFFFFF;
            temp_address -= 4;
            self.exception.bad_address = address as i32;
            self.exception.exception_reason = MIPSExceptionReason::ADES;
            self.exception.is_in_branch_delay_slot = self.prev_was_branch;
            self.exception.program_counter_origin = if self.exception.is_in_branch_delay_slot {
                temp_address as i32
            } else {
                self.program_counter
            };

            return;
        }

        // Load word from register, set byte order and write to memory.
        let mut temp_word = self.gte.read_data_reg(rt);

        // Swap byte order.
        temp_word = self.swap_word_endianness(temp_word);

        self.write_data_value(bridge, R3051Width::WORD, address as i32, temp_word);
    }

    /// This function handles the SWL R3051 instruction.
    fn swl_instruction(&mut self, bridge: &mut dyn CpuBridge, instruction: i32) {

        // Get rs, rt and immediate.
        let immediate = instruction.sign_extend(15);
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;

        // Calculate address.
        let address = ((self.general_registers[rs] as i64) & 0xFFFFFFFF) + (immediate as i64);

        // Check if address is allowed, trigger exception if not.
        if !self.sccp.is_address_allowed(address as i32) {
            let mut temp_address = (self.program_counter as i64) & 0xFFFFFFFF;
            temp_address -= 4;
            self.exception.bad_address = address as i32;
            self.exception.exception_reason = MIPSExceptionReason::ADES;
            self.exception.is_in_branch_delay_slot = self.prev_was_branch;
            self.exception.program_counter_origin = if self.exception.is_in_branch_delay_slot {
                temp_address as i32
            } else {
                self.program_counter
            };

            return;
        }

        // Align address, fetch word, and store shift index - sort byte order too.
        let temp_address = (address & 0xFFFFFFFC) as i32;
        let byte_shift_index = (!address & 0x3) as i32;
        let mut temp_word = self.general_registers[rt];

        // Swap byte order.
        temp_word = self.swap_word_endianness(temp_word);

        // Shift word value left by required amount.
        temp_word <<= byte_shift_index * 8;

        // Fetch memory contents, and calculate mask.
        let mut temp_val = bridge.read_word(self, temp_address);
        let mask = !((0xFFFFFFFF_u32 as i32) << (byte_shift_index * 8));
        temp_val &= mask;

        // Merge contents.
        temp_word |= temp_val;

        // Write word to memory.
        self.write_data_value(bridge, R3051Width::WORD, temp_address, temp_word);
    }

    /// This function handles the SWR R3051 instruction.
    fn swr_instruction(&mut self, bridge: &mut dyn CpuBridge, instruction: i32) {

        // Get rs, rt and immediate.
        let immediate = instruction.sign_extend(15);
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;

        // Calculate address.
        let address = ((self.general_registers[rs] as i64) & 0xFFFFFFFF) + (immediate as i64);

        // Check if address is allowed, trigger exception if not.
        if !self.sccp.is_address_allowed(address as i32) {
            let mut temp_address = (self.program_counter as i64) & 0xFFFFFFFF;
            temp_address -= 4;
            self.exception.bad_address = address as i32;
            self.exception.exception_reason = MIPSExceptionReason::ADES;
            self.exception.is_in_branch_delay_slot = self.prev_was_branch;
            self.exception.program_counter_origin = if self.exception.is_in_branch_delay_slot {
                temp_address as i32
            } else {
                self.program_counter
            };

            return;
        }

        // Align address, fetch word, and store shift index - sort byte order too.
        let temp_address = (address & 0xFFFFFFFC) as i32;
        let byte_shift_index = (address & 0x3) as i32;
        let mut temp_word = self.general_registers[rt];

        // Swap byte order.
        temp_word = self.swap_word_endianness(temp_word);

        // Shift word value right by required amount.
        temp_word = temp_word.logical_rshift(byte_shift_index * 8);

        // Fetch rt contents, and calculate mask.
        let mut temp_val = bridge.read_word(self, temp_address);
        let mask = !(0xFFFFFFFF_u32 as i32).logical_rshift(byte_shift_index * 8);
        temp_val &= mask;

        // Merge contents.
        temp_word |= temp_val;

        // Write word to main memory.
        self.write_data_value(bridge, R3051Width::WORD, temp_address, temp_word);
    }

    /// This function handles the SYSCALL R3051 instruction.
    fn syscall_instruction(&mut self) {

        // Trigger System Call Exception.
        let mut temp_address = (self.program_counter as i64) & 0xFFFFFFFF;
        temp_address -= 4;
        self.exception.exception_reason = MIPSExceptionReason::SYS;
        self.exception.is_in_branch_delay_slot = self.prev_was_branch;
        self.exception.program_counter_origin = if self.exception.is_in_branch_delay_slot {
            temp_address as i32
        } else {
            self.program_counter
        };
    }

    /// This function handles the XOR R3051 instruction.
    fn xor_instruction(&mut self, instruction: i32) {

        // Get rs, rt and rd.
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;
        let rd = (instruction.logical_rshift(11) & 0x1F) as usize;

        // Bitwise XOR rsVal and rtVal, storing result.
        self.general_registers[rd] = self.general_registers[rs] ^ self.general_registers[rt];
        self.general_registers[0] = 0;
    }

    /// This function handles the XORI R3051 instruction.
    fn xori_instruction(&mut self, instruction: i32) {

        // Get rs, rt and immediate.
        let immediate = instruction & 0xFFFF;
        let rs = (instruction.logical_rshift(21) & 0x1F) as usize;
        let rt = (instruction.logical_rshift(16) & 0x1F) as usize;

        // Zero extending immediate is already done for us
        // so just XOR with rsVal and store result.
        self.general_registers[rt] = immediate ^ self.general_registers[rs];
        self.general_registers[0] = 0;
    }

    /// This function executes an opcode in interpretive mode.
    fn execute_opcode(&mut self, bridge: &mut dyn CpuBridge, instruction: i32, temp_branch_address: i32) {

        // Deal with opcode.
        let opcode = instruction.logical_rshift(26);
        match opcode {

            0 => { // SPECIAL.
                let special_val = instruction & 0x3F;
                match special_val {

                    0 => {
                        // SLL.
                        self.sll_instruction(instruction);
                    },

                    2 => {
                        // SRL.
                        self.srl_instruction(instruction);
                    },

                    3 => {
                        // SRA.
                        self.sra_instruction(instruction);
                    },

                    4 => {
                        // SLLV.
                        self.sllv_instruction(instruction);
                    },

                    6 => {
                        // SRLV.
                        self.srlv_instruction(instruction);
                    },

                    7 => {
                        // SRAV.
                        self.srav_instruction(instruction);
                    },

                    8 => {
                        // JR.
                        self.jr_instruction(instruction);
                    },

                    9 => {
                        // JALR.
                        self.jalr_instruction(instruction);
                    },

                    12 => {
                        // SYSCALL.
                        self.syscall_instruction();
                    },

                    13 => {
                        // BREAK.
                        self.break_instruction();
                    },

                    16 => {
                        // MFHI.
                        self.mfhi_instruction(instruction);
                    },

                    17 => {
                        // MTHI.
                        self.mthi_instruction(instruction);
                    },

                    18 => {
                        // MFLO.
                        self.mflo_instruction(instruction);
                    },

                    19 => {
                        // MTLO.
                        self.mtlo_instruction(instruction);
                    },

                    24 => {
                        // MULT.
                        self.mult_instruction(instruction);
                    },

                    25 => {
                        // MULTU.
                        self.multu_instruction(instruction);
                    },

                    26 => {
                        // DIV.
                        self.div_instruction(instruction);
                    },

                    27 => {
                        // DIVU.
                        self.divu_instruction(instruction);
                    },

                    32 => {
                        // ADD.
                        self.add_instruction(instruction);
                    },

                    33 => {
                        // ADDU.
                        self.addu_instruction(instruction);
                    },

                    34 => {
                        // SUB.
                        self.sub_instruction(instruction);
                    },

                    35 => {
                        // SUBU.
                        self.subu_instruction(instruction);
                    },

                    36 => {
                        // AND.
                        self.and_instruction(instruction);
                    },

                    37 => {
                        // OR.
                        self.or_instruction(instruction);
                    },

                    38 => {
                        // XOR.
                        self.xor_instruction(instruction);
                    },

                    39 => {
                        // NOR.
                        self.nor_instruction(instruction);
                    },

                    42 => {
                        // SLT.
                        self.slt_instruction(instruction);
                    },

                    43 => {
                        // SLTU.
                        self.sltu_instruction(instruction);
                    },

                    _ => {
                        // Unrecognised - trigger Reserved Instruction Exception
                        self.exception.exception_reason = MIPSExceptionReason::RI;
                        self.exception.is_in_branch_delay_slot = self.prev_was_branch;
                        self.exception.program_counter_origin = if self.exception.is_in_branch_delay_slot {
                            temp_branch_address
                        } else {
                            self.program_counter
                        };
                    },
                }
            },

            1 => { // BCOND.
                let bcond_val = instruction.logical_rshift(16) & 0x1F;
                match bcond_val {

                    0 => {
                        // BLTZ.
                        self.bltz_instruction(instruction);
                    },

                    1 => {
                        // BGEZ.
                        self.bgez_instruction(instruction);
                    },

                    16 => {
                        // BLTZAL.
                        self.bltzal_instruction(instruction);
                    },

                    17 => {
                        // BGEZAL.
                        self.bgezal_instruction(instruction);
                    },

                    _ => (),
                }
            },

            2 => {
                // J.
                self.j_instruction(instruction);
            },

            3 => {
                // JAL.
                self.jal_instruction(instruction);
            },

            4 => {
                // BEQ.
                self.beq_instruction(instruction);
            },

            5 => {
                // BNE.
                self.bne_instruction(instruction);
            },

            6 => {
                // BLEZ.
                self.blez_instruction(instruction);
            },

            7 => {
                // BGTZ.
                self.bgtz_instruction(instruction);
            },

            8 => {
                // ADDI.
                self.addi_instruction(instruction);
            },

            9 => {
                // ADDIU.
                self.addiu_instruction(instruction);
            },

            10 => {
                // SLTI.
                self.slti_instruction(instruction);
            },

            11 => {
                // SLTIU.
                self.sltiu_instruction(instruction);
            },

            12 => {
                // ANDI.
                self.andi_instruction(instruction);
            },

            13 => {
                // ORI.
                self.ori_instruction(instruction);
            },

            14 => {
                // XORI.
                self.xori_instruction(instruction);
            },

            15 => {
                // LUI.
                self.lui_instruction(instruction);
            },

            16 => 'cop0: { // COP0.
                if !self.sccp.is_co_processor_usable(0) && !self.sccp.are_we_in_kernel_mode() {
                    // COP0 unusbale and we are not in kernel mode - throw exception.
                    self.exception.co_processor_num = 0;
                    self.exception.exception_reason = MIPSExceptionReason::CPU;
                    self.exception.is_in_branch_delay_slot = self.prev_was_branch;
                    self.exception.program_counter_origin = if self.exception.is_in_branch_delay_slot {
                        temp_branch_address
                    } else {
                        self.program_counter
                    };
                    break 'cop0;
                }

                let rfe = instruction & 0x1F;
                match rfe {

                    16 => {
                        // RFE.
                        self.rfe_instruction();
                    },

                    _ => {
                        let cop0_val = instruction.logical_rshift(21) & 0x1F;
                        match cop0_val {

                            0 => {
                                // MF.
                                self.mf0_instruction(instruction);
                            },

                            4 => {
                                // MT.
                                self.mt0_instruction(instruction);
                            },

                            _ => {
                                // Throw reserved instruction exception.
                                self.exception.exception_reason = MIPSExceptionReason::RI;
                                self.exception.is_in_branch_delay_slot = self.prev_was_branch;
                                self.exception.program_counter_origin = if self.exception.is_in_branch_delay_slot {
                                    temp_branch_address
                                } else {
                                    self.program_counter
                                };
                            },
                        }
                    },
                }
            },

            17 => { // COP1.
                // COP1 unusable - throw exception.
                self.exception.co_processor_num = 1;
                self.exception.exception_reason = MIPSExceptionReason::CPU;
                self.exception.is_in_branch_delay_slot = self.prev_was_branch;
                self.exception.program_counter_origin = if self.exception.is_in_branch_delay_slot {
                    temp_branch_address
                } else {
                    self.program_counter
                };
            },

            18 => 'cop2: { // COP2.
                if !self.sccp.is_co_processor_usable(2) {
                    // COP2 unusable - throw exception.
                    self.exception.co_processor_num = 2;
                    self.exception.exception_reason = MIPSExceptionReason::CPU;
                    self.exception.is_in_branch_delay_slot = self.prev_was_branch;
                    self.exception.program_counter_origin = if self.exception.is_in_branch_delay_slot {
                        temp_branch_address
                    } else {
                        self.program_counter
                    };
                    break 'cop2;
                }

                let cop2_val = instruction.logical_rshift(21) & 0x1F;
                match cop2_val {

                    0 => {
                        // MF.
                        self.mf2_instruction(instruction);
                    },

                    2 => {
                        // CF.
                        self.cf2_instruction(instruction);
                    },

                    4 => {
                        // MT.
                        self.mt2_instruction(instruction);
                    },

                    6 => {
                        // CT.
                        self.ct2_instruction(instruction);
                    },

                    8 => { // BC.
                        let cop2_val_extra = instruction.logical_rshift(16) & 0x1F;
                        match cop2_val_extra {

                            0 => {
                                // BC2F.
                                self.bc2f_instruction(instruction);
                            },

                            1 => {
                                // BC2T.
                                self.bc2t_instruction(instruction);
                            },

                            _ => (),
                        }
                    },

                    16..=31 => {
                        // Co-processor specific.
                        self.gte_cycles = self.gte.gte_function(instruction);
                    },

                    _ => (),
                }
            },

            19 => { // COP3.
                // COP3 unusable - throw exception.
                self.exception.co_processor_num = 3;
                self.exception.exception_reason = MIPSExceptionReason::CPU;
                self.exception.is_in_branch_delay_slot = self.prev_was_branch;
                self.exception.program_counter_origin = if self.exception.is_in_branch_delay_slot {
                    temp_branch_address
                } else {
                    self.program_counter
                };
            },

            32 => {
                // LB.
                self.lb_instruction(bridge, instruction);
            },

            33 => {
                // LH.
                self.lh_instruction(bridge, instruction);
            },

            34 => {
                // LWL.
                self.lwl_instruction(bridge, instruction);
            },

            35 => {
                // LW.
                self.lw_instruction(bridge, instruction);
            },

            36 => {
                // LBU.
                self.lbu_instruction(bridge, instruction);
            },

            37 => {
                // LHU.
                self.lhu_instruction(bridge, instruction);
            },

            38 => {
                // LWR.
                self.lwr_instruction(bridge, instruction);
            },

            40 => {
                // SB.
                self.sb_instruction(bridge, instruction);
            },

            41 => {
                // SH.
                self.sh_instruction(bridge, instruction);
            },

            42 => {
                // SWL.
                self.swl_instruction(bridge, instruction);
            },

            43 => {
                // SW.
                self.sw_instruction(bridge, instruction);
            },

            46 => {
                // SWR.
                self.swr_instruction(bridge, instruction);
            },

            49 => {
                // LWC1 (COP1 doesn't exist so throw exception).
                self.exception.co_processor_num = 1;
                self.exception.exception_reason = MIPSExceptionReason::CPU;
                self.exception.is_in_branch_delay_slot = self.prev_was_branch;
                self.exception.program_counter_origin = if self.exception.is_in_branch_delay_slot {
                    temp_branch_address
                } else {
                    self.program_counter
                };
            },

            50 => {
                // LWC2.
                self.lwc2_instruction(bridge, instruction);
            },

            51 => {
                // LWC3 (COP3 doesn't exist so throw exception).
                self.exception.co_processor_num = 3;
                self.exception.exception_reason = MIPSExceptionReason::CPU;
                self.exception.is_in_branch_delay_slot = self.prev_was_branch;
                self.exception.program_counter_origin = if self.exception.is_in_branch_delay_slot {
                    temp_branch_address
                } else {
                    self.program_counter
                };
            },

            48 | 56 => {
                // LWC0 and SWC0.
                self.exception.co_processor_num = 0;
                self.exception.exception_reason = MIPSExceptionReason::CPU;
                self.exception.is_in_branch_delay_slot = self.prev_was_branch;
                self.exception.program_counter_origin = if self.exception.is_in_branch_delay_slot {
                    temp_branch_address
                } else {
                    self.program_counter
                };
            },

            57 => {
                // SWC1 (COP1 doesn't exist so throw exception).
                self.exception.co_processor_num = 1;
                self.exception.exception_reason = MIPSExceptionReason::CPU;
                self.exception.is_in_branch_delay_slot = self.prev_was_branch;
                self.exception.program_counter_origin = if self.exception.is_in_branch_delay_slot {
                    temp_branch_address
                } else {
                    self.program_counter
                };
            },

            58 => {
                // SWC2.
                self.swc2_instruction(bridge, instruction);
            },

            59 => {
                // SWC3 (COP3 doesn't exist so throw exception).
                self.exception.co_processor_num = 3;
                self.exception.exception_reason = MIPSExceptionReason::CPU;
                self.exception.is_in_branch_delay_slot = self.prev_was_branch;
                self.exception.program_counter_origin = if self.exception.is_in_branch_delay_slot {
                    temp_branch_address
                } else {
                    self.program_counter
                };
            },

            _ => {
                // Unrecognised - trigger Reserved Instruction Exception
                self.exception.exception_reason = MIPSExceptionReason::RI;
                self.exception.is_in_branch_delay_slot = self.prev_was_branch;
                self.exception.program_counter_origin = if self.exception.is_in_branch_delay_slot {
                    temp_branch_address
                } else {
                    self.program_counter
                };
            },
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

    /// Get the system bus holder.
    fn get_system_bus_holder(
        &mut self,
        _bridge: &mut dyn CpuBridge
    ) -> SystemBusHolder {
        self.system_bus_holder
    }

    /// Move the whole processor on by one block of instructions.
    fn execute_instructions(
        &mut self,
        bridge: &mut dyn CpuBridge
    ) -> i64 {

        // Enter loop.
        loop {
            // Setup cycle count.
            self.cycles = 0;

            // Check address is OK, throwing exception if not.
            let mut temp_address = (((self.program_counter as i64) & 0xFFFFFFFF) - 4) as i32;

            // Perform read of instruction.
            let temp_instruction = self.read_instruction_word(bridge, self.program_counter, temp_address);
            if temp_instruction == -1 {
                self.cycles += 1;
                self.total_cycles += 1;
                let cycles = self.cycles;
                bridge.append_sync_cycles(self, cycles);
                continue;
            }

            // We now have instruction value. Swap the bytes.
            let instruction = self.swap_word_endianness(temp_instruction as i32);

            // Execute.
            self.execute_opcode(bridge, instruction, temp_address);

            // Handle exception if there was one.
            if self.handle_exception() {
                self.cycles += 1;
                self.total_cycles += 1;
                let cycles = self.cycles;
                bridge.append_sync_cycles(self, cycles);
                continue;
            }

            // Handle interrupt if there was one.
            if self.is_branch && self.handle_interrupts(bridge) {
                self.cycles += 1;
                self.total_cycles += 1;
                let cycles = self.cycles;
                bridge.append_sync_cycles(self, cycles);
                continue;
            }

            // Jump if pending, else add four to program counter.
            if self.jump_pending && self.prev_was_branch {
                self.program_counter = self.jump_address;
                self.jump_pending = false;
            } else {
                temp_address = (((self.program_counter as i64) & 0xFFFFFFFF) + 4) as i32;
                self.program_counter = temp_address;
            }

            // Increment cycle count.
            let cycles_to_add = if self.gte_cycles == 0 {
                1
            } else {
                self.gte_cycles
            };
            self.cycles += cycles_to_add;
            self.total_cycles += cycles_to_add as i64;
            self.gte_cycles = 0;

            // Setup whether the instruction just gone was a branch, and clear
            // current branch status.
            self.prev_was_branch = self.is_branch;
            self.is_branch = false;

            // Append number of cycles instruction took.
            {
                let cycles = self.cycles;
                bridge.append_sync_cycles(self, cycles);
            }

            if self.prev_was_branch {
                break;
            }
        }

        // Return cycle count for this block after resetting it in the CPU object.
        let ret_val = self.total_cycles;
        self.total_cycles = 0;

        ret_val
    }
}

/// This enum is used to specify the width we want to use (byte/half word/word).
enum R3051Width {
    BYTE,
    HALFWORD,
    WORD,
}

#[cfg(test)]
mod tests;