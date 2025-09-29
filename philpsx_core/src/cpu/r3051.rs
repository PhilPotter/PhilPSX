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
        let result = rs_val + rt_val;

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
        let result = rs_val + immediate;

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
    fn break_instruction(&mut self, instruction: i32) {

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

/*
/*
 * This function handles the LH R3051 instruction.
 */
static void R3051_LH(R3051 *cpu, int32_t instruction)
{
    // Get rs, rt and immediate
    int32_t immediate = instruction & 0xFFFF;
    int32_t rs = logical_rshift(instruction, 21) & 0x1F;
    int32_t rt = logical_rshift(instruction, 16) & 0x1F;

    // Calculate address
    int64_t address = cpu->generalRegisters[rs] & 0xFFFFFFFFL;
    if ((immediate & 0x8000) == 0x8000) {
        immediate |= 0xFFFF0000;
    }
    address += immediate;

    // Check if address is allowed and half-word aligned, trigger
    // exception if not
    if (!Cop0_isAddressAllowed(&cpu->sccp, (int32_t)address) ||
            address % 2 != 0) {
        int64_t tempAddress = cpu->programCounter & 0xFFFFFFFFL;
        tempAddress -= 4;
        cpu->exception.badAddress = (int32_t)address;
        cpu->exception.exceptionReason = PHILPSX_EXCEPTION_ADEL;
        cpu->exception.isInBranchDelaySlot = cpu->prevWasBranch;
        cpu->exception.programCounterOrigin
                = cpu->exception.isInBranchDelaySlot ?
                    (int32_t)tempAddress : cpu->programCounter;
        return;
    }

    // Load half-word and swap endianness, sign extend
    int32_t tempHalfWord = 0xFFFF & R3051_readDataValue(
            cpu,
            PHILPSX_R3051_HALFWORD,
            (int32_t)address
            );
    tempHalfWord = ((tempHalfWord << 8) & 0xFF00) |
            logical_rshift(tempHalfWord, 8);
    if ((tempHalfWord & 0x8000) == 0x8000) {
        tempHalfWord |= 0xFFFF0000;
    }

    // Write half-word to correct register
    cpu->generalRegisters[rt] = tempHalfWord;
    cpu->generalRegisters[0] = 0;
}

/*
 * This function handles the LHU R3051 instruction.
 */
static void R3051_LHU(R3051 *cpu, int32_t instruction)
{
    // Get rs, rt and immediate
    int32_t immediate = instruction & 0xFFFF;
    int32_t rs = logical_rshift(instruction, 21) & 0x1F;
    int32_t rt = logical_rshift(instruction, 16) & 0x1F;

    // Calculate address
    int64_t address = cpu->generalRegisters[rs] & 0xFFFFFFFFL;
    if ((immediate & 0x8000) == 0x8000) {
        immediate |= 0xFFFF0000;
    }
    address += immediate;

    // Check if address is allowed and half-word aligned, trigger
    // exception if not
    if (!Cop0_isAddressAllowed(&cpu->sccp, (int32_t)address) ||
            address % 2 != 0) {
        int64_t tempAddress = cpu->programCounter & 0xFFFFFFFFL;
        tempAddress -= 4;
        cpu->exception.badAddress = (int32_t)address;
        cpu->exception.exceptionReason = PHILPSX_EXCEPTION_ADEL;
        cpu->exception.isInBranchDelaySlot = cpu->prevWasBranch;
        cpu->exception.programCounterOrigin
                = cpu->exception.isInBranchDelaySlot ?
                    (int32_t)tempAddress : cpu->programCounter;
        return;
    }

    // Load half-word and swap endianness, zero extend
    int32_t tempHalfWord = 0xFFFF & R3051_readDataValue(
            cpu,
            PHILPSX_R3051_HALFWORD,
            (int32_t)address
            );

    // Swap byte order
    tempHalfWord = ((tempHalfWord << 8) & 0xFF00) |
            logical_rshift(tempHalfWord, 8);

    // Write half-word to correct register
    cpu->generalRegisters[rt] = tempHalfWord;
    cpu->generalRegisters[0] = 0;
}

/*
 * This function handles the LUI R3051 instruction.
 */
static void R3051_LUI(R3051 *cpu, int32_t instruction)
{
    // Get rt and immediate
    int32_t immediate = instruction & 0xFFFF;
    int32_t rt = logical_rshift(instruction, 16) & 0x1F;

    // Shift immediate left by 16 bits (leaving least significant
    // 16 bits as zeroes) and store result
    cpu->generalRegisters[rt] = (immediate << 16);
    cpu->generalRegisters[0] = 0;
}

/*
 * This function handles the LW R3051 instruction.
 */
static void R3051_LW(R3051 *cpu, int32_t instruction)
{
    // Get rs, rt and immediate
    int32_t immediate = instruction & 0xFFFF;
    int32_t rs = logical_rshift(instruction, 21) & 0x1F;
    int32_t rt = logical_rshift(instruction, 16) & 0x1F;

    // Calculate address
    int64_t address = cpu->generalRegisters[rs] & 0xFFFFFFFFL;
    if ((immediate & 0x8000) == 0x8000) {
        immediate |= 0xFFFF0000;
    }
    address += immediate;

    // Check if address is allowed and word aligned, trigger exception if not
    if (!Cop0_isAddressAllowed(&cpu->sccp, (int32_t)address) ||
            address % 4 != 0) {
        int64_t tempAddress = cpu->programCounter & 0xFFFFFFFFL;
        tempAddress -= 4;
        cpu->exception.badAddress = (int32_t)address;
        cpu->exception.exceptionReason = PHILPSX_EXCEPTION_ADEL;
        cpu->exception.isInBranchDelaySlot = cpu->prevWasBranch;
        cpu->exception.programCounterOrigin
                = cpu->exception.isInBranchDelaySlot ?
                    (int32_t)tempAddress : cpu->programCounter;
        return;
    }

    // Load word
    int32_t tempWord = R3051_readDataValue(
            cpu,
            PHILPSX_R3051_WORD,
            (int32_t)address
            );

    // Swap byte order
    tempWord = R3051_swapWordEndianness(cpu, tempWord);

    // Write word to correct register
    cpu->generalRegisters[rt] = tempWord;
    cpu->generalRegisters[0] = 0;
}

/*
 * This function handles the LWC2 R3051 instruction.
 */
static void R3051_LWC2(R3051 *cpu, int32_t instruction)
{
    // Get rs, rt and immediate
    int32_t immediate = instruction & 0xFFFF;
    int32_t rs = logical_rshift(instruction, 21) & 0x1F;
    int32_t rt = logical_rshift(instruction, 16) & 0x1F;

    // Calculate address
    int64_t address = cpu->generalRegisters[rs] & 0xFFFFFFFFL;
    if ((immediate & 0x8000) == 0x8000) {
        immediate |= 0xFFFF0000;
    }
    address += immediate;

    // Check if address is allowed and word aligned, trigger exception if not
    if (!Cop0_isAddressAllowed(&cpu->sccp, (int32_t)address) ||
            address % 4 != 0) {
        int64_t tempAddress = cpu->programCounter & 0xFFFFFFFFL;
        tempAddress -= 4;
        cpu->exception.badAddress = (int32_t)address;
        cpu->exception.exceptionReason = PHILPSX_EXCEPTION_ADEL;
        cpu->exception.isInBranchDelaySlot = cpu->prevWasBranch;
        cpu->exception.programCounterOrigin
                = cpu->exception.isInBranchDelaySlot ?
                    (int32_t)tempAddress : cpu->programCounter;
        return;
    }

    // Load word
    int32_t tempWord = R3051_readDataValue(
            cpu,
            PHILPSX_R3051_WORD,
            (int32_t)address
            );

    // Swap byte order
    tempWord = R3051_swapWordEndianness(cpu, tempWord);

    // Write word to correct COP2 data register
    Cop2_writeDataReg(&cpu->gte, rt, tempWord, false);
}

/*
 * This function handles the LWL R3051 instruction.
 */
static void R3051_LWL(R3051 *cpu, int32_t instruction)
{
    // Get rs, rt and immediate
    int32_t immediate = instruction & 0xFFFF;
    int32_t rs = logical_rshift(instruction, 21) & 0x1F;
    int32_t rt = logical_rshift(instruction, 16) & 0x1F;

    // Calculate address
    int64_t address = cpu->generalRegisters[rs] & 0xFFFFFFFFL;
    if ((immediate & 0x8000) == 0x8000) {
        immediate |= 0xFFFF0000;
    }
    address += immediate;

    // Check if address is allowed, trigger exception if not
    if (!Cop0_isAddressAllowed(&cpu->sccp, (int32_t)address)) {
        int64_t tempAddress = cpu->programCounter & 0xFFFFFFFFL;
        tempAddress -= 4;
        cpu->exception.badAddress = (int32_t)address;
        cpu->exception.exceptionReason = PHILPSX_EXCEPTION_ADEL;
        cpu->exception.isInBranchDelaySlot = cpu->prevWasBranch;
        cpu->exception.programCounterOrigin
                = cpu->exception.isInBranchDelaySlot ?
                    (int32_t)tempAddress : cpu->programCounter;
        return;
    }

    // Align address, fetch word, and store shift index
    int32_t tempAddress = (int32_t)(address & 0xFFFFFFFC);
    int32_t byteShiftIndex = (int32_t)(~address & 0x3);
    int32_t tempWord = R3051_readDataValue(
            cpu,
            PHILPSX_R3051_WORD,
            tempAddress
            );

    // Swap byte order
    tempWord = R3051_swapWordEndianness(cpu, tempWord);

    // Shift word value left by required amount
    tempWord = tempWord << (byteShiftIndex * 8);

    // Fetch rt contents, and calculate mask
    int32_t tempRtVal = cpu->generalRegisters[rt];
    int32_t mask = ~(0xFFFFFFFF << (byteShiftIndex * 8));
    tempRtVal &= mask;

    // Merge contents
    tempWord |= tempRtVal;

    // Write word to correct register
    cpu->generalRegisters[rt] = tempWord;
    cpu->generalRegisters[0] = 0;
}

/*
 * This function handles the LWR R3051 instruction.
 */
static void R3051_LWR(R3051 *cpu, int32_t instruction)
{
    // Get rs, rt and immediate
    int32_t immediate = instruction & 0xFFFF;
    int32_t rs = logical_rshift(instruction, 21) & 0x1F;
    int32_t rt = logical_rshift(instruction, 16) & 0x1F;

    // Calculate address
    int64_t address = cpu->generalRegisters[rs] & 0xFFFFFFFFL;
    if ((immediate & 0x8000) == 0x8000) {
        immediate |= 0xFFFF0000;
    }
    address += immediate;

    // Check if address is allowed, trigger exception if not
    if (!Cop0_isAddressAllowed(&cpu->sccp, (int32_t)address)) {
        int64_t tempAddress = cpu->programCounter & 0xFFFFFFFFL;
        tempAddress -= 4;
        cpu->exception.badAddress = (int32_t)address;
        cpu->exception.exceptionReason = PHILPSX_EXCEPTION_ADEL;
        cpu->exception.isInBranchDelaySlot = cpu->prevWasBranch;
        cpu->exception.programCounterOrigin
                = cpu->exception.isInBranchDelaySlot ? (int32_t)tempAddress : cpu->programCounter;
        return;
    }

    // Align address, fetch word, and store shift index
    int32_t tempAddress = (int32_t)(address & 0xFFFFFFFC);
    int32_t byteShiftIndex = (int32_t)(address & 0x3);
    int32_t tempWord = R3051_readDataValue(cpu, PHILPSX_R3051_WORD, tempAddress);

    // Swap byte order
    tempWord = R3051_swapWordEndianness(cpu, tempWord);

    // Shift word value left by required amount
    tempWord = logical_rshift(tempWord, (byteShiftIndex * 8));

    // Fetch rt contents, and calculate mask
    int32_t tempRtVal = cpu->generalRegisters[rt];
    int32_t mask = ~(logical_rshift(0xFFFFFFFF, (byteShiftIndex * 8)));
    tempRtVal &= mask;

    // Merge contents
    tempWord |= tempRtVal;

    // Write word to correct register
    cpu->generalRegisters[rt] = tempWord;
    cpu->generalRegisters[0] = 0;
}

/*
 * This function handles the MF0 R3051 instruction.
 */
static void R3051_MF0(R3051 *cpu, int32_t instruction)
{
    // Get rt and rd
    int32_t rt = logical_rshift(instruction, 16) & 0x1F;
    int32_t rd = logical_rshift(instruction, 11) & 0x1F;

    // Check if rd is any of the following and trigger exception if so
    switch (rd) {
        case 0:
        case 1:
        case 2:
        case 4:
        case 10:
        {
            // Trigger exception
            int64_t tempAddress = cpu->programCounter & 0xFFFFFFFFL;
            tempAddress -= 4;
            cpu->exception.exceptionReason = PHILPSX_EXCEPTION_RI;
            cpu->exception.isInBranchDelaySlot = cpu->prevWasBranch;
            cpu->exception.programCounterOrigin
                    = cpu->exception.isInBranchDelaySlot ?
                        (int32_t)tempAddress : cpu->programCounter;
            return;
        }
    }

    // Move COP0 reg rd to CPU reg rt
    cpu->generalRegisters[rt] = Cop0_readReg(&cpu->sccp, rd);
    cpu->generalRegisters[0] = 0;
}

/*
 * This function handles the MF2 R3051 instruction.
 */
static void R3051_MF2(R3051 *cpu, int32_t instruction)
{
    // Get rt and rd
    int32_t rt = logical_rshift(instruction, 16) & 0x1F;
    int32_t rd = logical_rshift(instruction, 11) & 0x1F;

    // Move from COP2 data reg rd to CPU reg rt
    cpu->generalRegisters[rt] = Cop2_readDataReg(&cpu->gte, rd);
    cpu->generalRegisters[0] = 0;
}

/*
 * This function handles the MFHI R3051 instruction.
 */
static void R3051_MFHI(R3051 *cpu, int32_t instruction)
{
    // Get rd
    int32_t rd = logical_rshift(instruction, 11) & 0x1F;

    // Move Hi to rd
    cpu->generalRegisters[rd] = cpu->hiReg;
    cpu->generalRegisters[0] = 0;
}

/*
 * This function handles the MFLO R3051 instruction.
 */
static void R3051_MFLO(R3051 *cpu, int32_t instruction)
{
    // Get rd
    int32_t rd = logical_rshift(instruction, 11) & 0x1F;

    // Move Lo to rd
    cpu->generalRegisters[rd] = cpu->loReg;
    cpu->generalRegisters[0] = 0;
}

/*
 * This function handles the MT0 R3051 instruction.
 */
static void R3051_MT0(R3051 *cpu, int32_t instruction)
{
    // Get rt and rd
    int32_t rt = logical_rshift(instruction, 16) & 0x1F;
    int32_t rd = logical_rshift(instruction, 11) & 0x1F;

    // Move CPU reg rt to COP0 reg rd
    Cop0_writeReg(&cpu->sccp, rd, cpu->generalRegisters[rt], false);
}

/*
 * This function handles the MT2 R3051 instruction.
 */
static void R3051_MT2(R3051 *cpu, int32_t instruction)
{
    // Get rt and rd
    int32_t rt = logical_rshift(instruction, 16) & 0x1F;
    int32_t rd = logical_rshift(instruction, 11) & 0x1F;

    // Move from CPU reg rt to COP2 data reg rd
    Cop2_writeDataReg(&cpu->gte, rd, cpu->generalRegisters[rt], false);
}

/*
 * This function handles the MTHI R3051 instruction.
 */
static void R3051_MTHI(R3051 *cpu, int32_t instruction)
{
    // Get rs
    int32_t rs = logical_rshift(instruction, 21) & 0x1F;

    // Move rs to Hi
    cpu->hiReg = cpu->generalRegisters[rs];
}

/*
 * This function handles the MTLO R3051 instruction.
 */
static void R3051_MTLO(R3051 *cpu, int32_t instruction)
{
    // Get rs
    int32_t rs = logical_rshift(instruction, 21) & 0x1F;

    // Move rs to Lo
    cpu->loReg = cpu->generalRegisters[rs];
}

/*
 * This function handles the MULT R3051 instruction.
 */
static void R3051_MULT(R3051 *cpu, int32_t instruction)
{
    // Get rs and rt
    int32_t rs = logical_rshift(instruction, 21) & 0x1F;
    int32_t rt = logical_rshift(instruction, 16) & 0x1F;

    // Multiply rs and rt as signed values
    int64_t rsVal = cpu->generalRegisters[rs];
    int64_t rtVal = cpu->generalRegisters[rt];
    int64_t result = rsVal * rtVal;

    // Store result
    cpu->hiReg = (int32_t)logical_rshift(result, 32);
    cpu->loReg = (int32_t)result;
}

/*
 * This function handles the MULTU R3051 instruction.
 */
static void R3051_MULTU(R3051 *cpu, int32_t instruction)
{
    // Get rs and rt
    int32_t rs = logical_rshift(instruction, 21) & 0x1F;
    int32_t rt = logical_rshift(instruction, 16) & 0x1F;

    // Multiply rs and rt as unsigned values
    int64_t rsVal = cpu->generalRegisters[rs] & 0xFFFFFFFFL;
    int64_t rtVal = cpu->generalRegisters[rt] & 0xFFFFFFFFL;
    int64_t result = rsVal * rtVal;

    // Store result
    cpu->hiReg = (int32_t)logical_rshift(result, 32);
    cpu->loReg = (int32_t)result;
}

/*
 * This function handles the NOR R3051 instruction.
 */
static void R3051_NOR(R3051 *cpu, int32_t instruction)
{
    // Get rs, rt and rd
    int32_t rs = logical_rshift(instruction, 21) & 0x1F;
    int32_t rt = logical_rshift(instruction, 16) & 0x1F;
    int32_t rd = logical_rshift(instruction, 11) & 0x1F;

    // Bitwise NOR rsVal and rtVal, storing result
    cpu->generalRegisters[rd] =
            ~(cpu->generalRegisters[rs] | cpu->generalRegisters[rt]);
    cpu->generalRegisters[0] = 0;
}

/*
 * This function handles the OR R3051 instruction.
 */
static void R3051_OR(R3051 *cpu, int32_t instruction)
{
    // Get rs, rt and rd
    int32_t rs = logical_rshift(instruction, 21) & 0x1F;
    int32_t rt = logical_rshift(instruction, 16) & 0x1F;
    int32_t rd = logical_rshift(instruction, 11) & 0x1F;

    // Bitwise OR rsVal and rtVal, storing result
    cpu->generalRegisters[rd] =
            cpu->generalRegisters[rs] | cpu->generalRegisters[rt];
    cpu->generalRegisters[0] = 0;
}

/*
 * This function handles the ORI R3051 instruction.
 */
static void R3051_ORI(R3051 *cpu, int32_t instruction)
{
    // Get rs, rt and immediate
    int32_t immediate = instruction & 0xFFFF;
    int32_t rs = logical_rshift(instruction, 21) & 0x1F;
    int32_t rt = logical_rshift(instruction, 16) & 0x1F;

    // Zero extending immediate is already done for us
    // so just OR with rsVal and store result
    cpu->generalRegisters[rt] = immediate | cpu->generalRegisters[rs];
    cpu->generalRegisters[0] = 0;
}

/*
 * This function handles the RFE R3051 instruction.
 */
static void R3051_RFE(R3051 *cpu, int32_t instruction)
{
    Cop0_rfe(&cpu->sccp);
}

/*
 * This function handles the SB R3051 instruction.
 */
static void R3051_SB(R3051 *cpu, int32_t instruction)
{
    // Get rs, rt and immediate
    int32_t immediate = instruction & 0xFFFF;
    int32_t rs = logical_rshift(instruction, 21) & 0x1F;
    int32_t rt = logical_rshift(instruction, 16) & 0x1F;

    // Calculate address
    int64_t address = cpu->generalRegisters[rs] & 0xFFFFFFFFL;
    if ((immediate & 0x8000) == 0x8000) {
        immediate |= 0xFFFF0000;
    }
    address += immediate;

    // Check if address is allowed, trigger exception if not
    if (!Cop0_isAddressAllowed(&cpu->sccp, (int32_t)address)) {
        int64_t tempAddress = cpu->programCounter & 0xFFFFFFFFL;
        tempAddress -= 4;
        cpu->exception.badAddress = (int32_t)address;
        cpu->exception.exceptionReason = PHILPSX_EXCEPTION_ADES;
        cpu->exception.isInBranchDelaySlot = cpu->prevWasBranch;
        cpu->exception.programCounterOrigin
                = cpu->exception.isInBranchDelaySlot ?
                    (int32_t)tempAddress : cpu->programCounter;
        return;
    }

    // Load byte from register and write to memory
    int32_t tempByte = 0xFF & cpu->generalRegisters[rt];
    R3051_writeDataValue(cpu, PHILPSX_R3051_BYTE, (int32_t)address, tempByte);
}

/*
 * This function handles the SH R3051 instruction.
 */
static void R3051_SH(R3051 *cpu, int32_t instruction)
{
    // Get rs, rt and immediate
    int32_t immediate = instruction & 0xFFFF;
    int32_t rs = logical_rshift(instruction, 21) & 0x1F;
    int32_t rt = logical_rshift(instruction, 16) & 0x1F;

    // Calculate address
    int64_t address = cpu->generalRegisters[rs] & 0xFFFFFFFFL;
    if ((immediate & 0x8000) == 0x8000) {
        immediate |= 0xFFFF0000;
    }
    address += immediate;

    // Check if address is allowed and half-word aligned, trigger
    // exception if not
    if (!Cop0_isAddressAllowed(&cpu->sccp, (int32_t)address) ||
            address % 2 != 0) {
        int64_t tempAddress = cpu->programCounter & 0xFFFFFFFFL;
        tempAddress -= 4;
        cpu->exception.badAddress = (int32_t)address;
        cpu->exception.exceptionReason = PHILPSX_EXCEPTION_ADES;
        cpu->exception.isInBranchDelaySlot = cpu->prevWasBranch;
        cpu->exception.programCounterOrigin
                = cpu->exception.isInBranchDelaySlot ?
                    (int32_t)tempAddress : cpu->programCounter;
        return;
    }

    // Load half-word from register and swap endianness, then write to memory
    // (checking for exceptions and stalls)
    int32_t tempHalfWord = 0xFFFF & cpu->generalRegisters[rt];

    // Swap byte order
    tempHalfWord = ((tempHalfWord << 8) & 0xFF00) |
            logical_rshift(tempHalfWord, 8);

    R3051_writeDataValue(
            cpu,
            PHILPSX_R3051_HALFWORD,
            (int32_t)address,
            tempHalfWord
            );
}

/*
 * This function handles the SLL R3051 instruction.
 */
static void R3051_SLL(R3051 *cpu, int32_t instruction)
{
    // Get rt, rd and shamt
    int32_t rt = logical_rshift(instruction, 16) & 0x1F;
    int32_t rd = logical_rshift(instruction, 11) & 0x1F;
    int32_t shamt = logical_rshift(instruction, 6) & 0x1F;

    // Shift rt value left by shamt bits, inserting zeroes
    // into low order bits, then store result
    cpu->generalRegisters[rd] = cpu->generalRegisters[rt] << shamt;
    cpu->generalRegisters[0] = 0;
}

/*
 * This function handles the SLLV R3051 instruction.
 */
static void R3051_SLLV(R3051 *cpu, int32_t instruction)
{
    // Get rs, rt and rd
    int32_t rs = logical_rshift(instruction, 21) & 0x1F;
    int32_t rt = logical_rshift(instruction, 16) & 0x1F;
    int32_t rd = logical_rshift(instruction, 11) & 0x1F;

    // Shift rt value left by (lowest 5 bits of rs value), 
    // inserting zeroes into low order bits, then
    // store result
    cpu->generalRegisters[rd] = 
            cpu->generalRegisters[rt] << (cpu->generalRegisters[rs] & 0x1F);
    cpu->generalRegisters[0] = 0;
}

/*
 * This function handles the SLT R3051 instruction.
 */
static void R3051_SLT(R3051 *cpu, int32_t instruction)
{
    // Get rs, rt and rd
    int32_t rs = logical_rshift(instruction, 21) & 0x1F;
    int32_t rt = logical_rshift(instruction, 16) & 0x1F;
    int32_t rd = logical_rshift(instruction, 11) & 0x1F;

    // Compare rsVal and rtVal, storing result
    if (cpu->generalRegisters[rs] < cpu->generalRegisters[rt]) {
        cpu->generalRegisters[rd] = 1;
    } else {
        cpu->generalRegisters[rd] = 0;
    }
    cpu->generalRegisters[0] = 0;
}

/*
 * This function handles the SLTI R3051 instruction.
 */
static void R3051_SLTI(R3051 *cpu, int32_t instruction)
{
    // Get rs, rt and immediate
    int32_t immediate = instruction & 0xFFFF;
    int32_t rs = logical_rshift(instruction, 21) & 0x1F;
    int32_t rt = logical_rshift(instruction, 16) & 0x1F;

    // Sign extend immediate
    if ((immediate & 0x8000) == 0x8000) {
        immediate |= 0xFFFF0000;
    }

    // Store result
    if (cpu->generalRegisters[rs] < immediate) {
        cpu->generalRegisters[rt] = 1;
    } else {
        cpu->generalRegisters[rt] = 0;
    }
    cpu->generalRegisters[0] = 0;
}

/*
 * This function handles the SLTIU R3051 instruction.
 */
static void R3051_SLTIU(R3051 *cpu, int32_t instruction)
{
    // Get rs, rt and immediate
    int32_t immediate = instruction & 0xFFFF;
    int32_t rs = logical_rshift(instruction, 21) & 0x1F;
    int32_t rt = logical_rshift(instruction, 16) & 0x1F;

    // Sign extend immediate
    if ((immediate & 0x8000) == 0x8000) {
        immediate |= 0xFFFF0000L;
    }

    // Treat rsVal as unsigned
    int64_t tempRsVal = cpu->generalRegisters[rs] & 0xFFFFFFFFL;

    // Store result
    if (tempRsVal < immediate) {
        cpu->generalRegisters[rt] = 1;
    } else {
        cpu->generalRegisters[rt] = 0;
    }
    cpu->generalRegisters[0] = 0;
}

/*
 * This function handles the SLTU R3051 instruction.
 */
static void R3051_SLTU(R3051 *cpu, int32_t instruction)
{
    // Get rs, rt and rd
    int32_t rs = logical_rshift(instruction, 21) & 0x1F;
    int32_t rt = logical_rshift(instruction, 16) & 0x1F;
    int32_t rd = logical_rshift(instruction, 11) & 0x1F;

    // Compare rsVal and rtVal as unsigned values, storing result
    if ((cpu->generalRegisters[rs] & 0xFFFFFFFFL) <
            (cpu->generalRegisters[rt] & 0xFFFFFFFFL)) {
        cpu->generalRegisters[rd] = 1;
    } else {
        cpu->generalRegisters[rd] = 0;
    }
    cpu->generalRegisters[0] = 0;
}

/*
 * This function handles the SRA R3051 instruction.
 */
static void R3051_SRA(R3051 *cpu, int32_t instruction)
{
    // Get rt, rd and shamt
    int32_t rt = logical_rshift(instruction, 16) & 0x1F;
    int32_t rd = logical_rshift(instruction, 11) & 0x1F;
    int32_t shamt = logical_rshift(instruction, 6) & 0x1F;

    // Shift rt value right by shamt bits, sign extending
    // high order bits, then store result
    cpu->generalRegisters[rd] = cpu->generalRegisters[rt] >> shamt;
    cpu->generalRegisters[0] = 0;
}

/*
 * This function handles the SRAV R3051 instruction.
 */
static void R3051_SRAV(R3051 *cpu, int32_t instruction)
{
    // Get rs, rt and rd
    int32_t rs = logical_rshift(instruction, 21) & 0x1F;
    int32_t rt = logical_rshift(instruction, 16) & 0x1F;
    int32_t rd = logical_rshift(instruction, 11) & 0x1F;

    // Shift rt value right by (lowest 5 bits of rs value), 
    // sign extending high order bits, then
    // store result
    cpu->generalRegisters[rd] =
            cpu->generalRegisters[rt] >> (cpu->generalRegisters[rs] & 0x1F);
    cpu->generalRegisters[0] = 0;
}

/*
 * This function handles the SRL R3051 instruction.
 */
static void R3051_SRL(R3051 *cpu, int32_t instruction)
{
    // Get rt, rd and shamt
    int32_t rt = logical_rshift(instruction, 16) & 0x1F;
    int32_t rd = logical_rshift(instruction, 11) & 0x1F;
    int32_t shamt = logical_rshift(instruction, 6) & 0x1F;

    // Shift rt value right by shamt bits, inserting zeroes
    // into high order bits, then store result
    cpu->generalRegisters[rd] =
            logical_rshift(cpu->generalRegisters[rt], shamt);
    cpu->generalRegisters[0] = 0;
}

/*
 * This function handles the SRLV R3051 instruction.
 */
static void R3051_SRLV(R3051 *cpu, int32_t instruction)
{
    // Get rs, rt and rd
    int32_t rs = logical_rshift(instruction, 21) & 0x1F;
    int32_t rt = logical_rshift(instruction, 16) & 0x1F;
    int32_t rd = logical_rshift(instruction, 11) & 0x1F;

    // Shift rt value right by (lowest 5 bits of rs value), 
    // inserting zeroes into high order bits, then
    // store result
    cpu->generalRegisters[rd] =
            logical_rshift(
            cpu->generalRegisters[rt],
            (cpu->generalRegisters[rs] & 0x1F)
            );
    cpu->generalRegisters[0] = 0;
}

/*
 * This function handles the SUB R3051 instruction.
 */
static void R3051_SUB(R3051 *cpu, int32_t instruction)
{
    // Get rs, rt and rd
    int32_t rs = logical_rshift(instruction, 21) & 0x1F;
    int32_t rt = logical_rshift(instruction, 16) & 0x1F;
    int32_t rd = logical_rshift(instruction, 11) & 0x1F;

    // Subtract rtVal from rsVal
    int32_t rsVal = cpu->generalRegisters[rs];
    int32_t rtVal = cpu->generalRegisters[rt];
    int32_t result = rsVal - rtVal;

    // Check for two's complement overflow
    if ((rsVal & 0x80000000) != (rtVal & 0x80000000)) {
        if ((rsVal & 0x80000000) != (result & 0x80000000)) {

            // Trigger exception
            int64_t tempAddress = cpu->programCounter & 0xFFFFFFFFL;
            tempAddress -= 4;
            cpu->exception.exceptionReason = PHILPSX_EXCEPTION_OVF;
            cpu->exception.isInBranchDelaySlot = cpu->prevWasBranch;
            cpu->exception.programCounterOrigin
                    = cpu->exception.isInBranchDelaySlot ?
                        (int32_t)tempAddress : cpu->programCounter;
            return;
        }
    }

    // Store result
    cpu->generalRegisters[rd] = result;
    cpu->generalRegisters[0] = 0;
}

/*
 * This function handles the SUBU R3051 instruction.
 */
static void R3051_SUBU(R3051 *cpu, int32_t instruction)
{
    // Get rs, rt and rd
    int32_t rs = logical_rshift(instruction, 21) & 0x1F;
    int32_t rt = logical_rshift(instruction, 16) & 0x1F;
    int32_t rd = logical_rshift(instruction, 11) & 0x1F;

    // Subtract rtVal from rsVal
    int32_t result = (int32_t)((cpu->generalRegisters[rs] & 0xFFFFFFFFL) -
            (cpu->generalRegisters[rt] & 0xFFFFFFFFL));

    // Store result
    cpu->generalRegisters[rd] = result;
    cpu->generalRegisters[0] = 0;
}

/*
 * This function handles the SW R3051 instruction.
 */
static void R3051_SW(R3051 *cpu, int32_t instruction)
{
    // Get rs, rt and immediate
    int32_t immediate = instruction & 0xFFFF;
    int32_t rs = logical_rshift(instruction, 21) & 0x1F;
    int32_t rt = logical_rshift(instruction, 16) & 0x1F;

    // Calculate address
    int64_t address = cpu->generalRegisters[rs] & 0xFFFFFFFFL;
    if ((immediate & 0x8000) == 0x8000) {
        immediate |= 0xFFFF0000;
    }
    address += immediate;

    // Check if address is allowed and word aligned, trigger exception if not
    if (!Cop0_isAddressAllowed(&cpu->sccp, (int32_t)address) ||
            address % 4 != 0) {
        int64_t tempAddress = cpu->programCounter & 0xFFFFFFFFL;
        tempAddress -= 4;
        cpu->exception.badAddress = (int32_t)address;
        cpu->exception.exceptionReason = PHILPSX_EXCEPTION_ADES;
        cpu->exception.isInBranchDelaySlot = cpu->prevWasBranch;
        cpu->exception.programCounterOrigin
                = cpu->exception.isInBranchDelaySlot ?
                    (int32_t)tempAddress : cpu->programCounter;
        return;
    }

    // Load word from register, set byte order and write to memory
    // (checking for exceptions and stalls)
    int32_t tempWord = cpu->generalRegisters[rt];

    // Swap byte order
    tempWord = R3051_swapWordEndianness(cpu, tempWord);

    R3051_writeDataValue(cpu, PHILPSX_R3051_WORD, (int32_t)address, tempWord);
}

/*
 * This function handles the SWC2 R3051 instruction.
 */
static void R3051_SWC2(R3051 *cpu, int32_t instruction)
{
    // Get rs, rt and immediate
    int32_t immediate = instruction & 0xFFFF;
    int32_t rs = logical_rshift(instruction, 21) & 0x1F;
    int32_t rt = logical_rshift(instruction, 16) & 0x1F;

    // Calculate address
    int64_t address = cpu->generalRegisters[rs] & 0xFFFFFFFFL;
    if ((immediate & 0x8000) == 0x8000) {
        immediate |= 0xFFFF0000;
    }
    address += immediate;

    // Check if address is allowed and word aligned, trigger exception if not
    if (!Cop0_isAddressAllowed(&cpu->sccp, (int32_t)address) ||
            address % 4 != 0) {
        int64_t tempAddress = cpu->programCounter & 0xFFFFFFFFL;
        tempAddress -= 4;
        cpu->exception.badAddress = (int32_t)address;
        cpu->exception.exceptionReason = PHILPSX_EXCEPTION_ADES;
        cpu->exception.isInBranchDelaySlot = cpu->prevWasBranch;
        cpu->exception.programCounterOrigin
                = cpu->exception.isInBranchDelaySlot ?
                    (int32_t)tempAddress : cpu->programCounter;
        return;
    }

    // Load word from register, set byte order and write to memory
    int32_t tempWord = Cop2_readDataReg(&cpu->gte, rt);

    // Swap byte order
    tempWord = R3051_swapWordEndianness(cpu, tempWord);

    R3051_writeDataValue(cpu, PHILPSX_R3051_WORD, (int32_t)address, tempWord);
}

/*
 * This function handles the SWL R3051 instruction.
 */
static void R3051_SWL(R3051 *cpu, int32_t instruction)
{
    // Get rs, rt and immediate
    int32_t immediate = instruction & 0xFFFF;
    int32_t rs = logical_rshift(instruction, 21) & 0x1F;
    int32_t rt = logical_rshift(instruction, 16) & 0x1F;

    // Calculate address
    int64_t address = cpu->generalRegisters[rs] & 0xFFFFFFFFL;
    if ((immediate & 0x8000) == 0x8000) {
        immediate |= 0xFFFF0000;
    }
    address += immediate;

    // Check if address is allowed, trigger exception if not
    if (!Cop0_isAddressAllowed(&cpu->sccp, (int32_t)address)) {
        int64_t tempAddress = cpu->programCounter & 0xFFFFFFFFL;
        tempAddress -= 4;
        cpu->exception.badAddress = (int32_t)address;
        cpu->exception.exceptionReason = PHILPSX_EXCEPTION_ADES;
        cpu->exception.isInBranchDelaySlot = cpu->prevWasBranch;
        cpu->exception.programCounterOrigin
                = cpu->exception.isInBranchDelaySlot ?
                    (int32_t)tempAddress : cpu->programCounter;
        return;
    }

    // Align address, fetch word, and store shift index - sort byte order too
    int32_t tempAddress = (int32_t)(address & 0xFFFFFFFC);
    int32_t byteShiftIndex = (int32_t)(~address & 0x3);
    int32_t tempWord = cpu->generalRegisters[rt];

    // Swap byte order
    tempWord = R3051_swapWordEndianness(cpu, tempWord);

    // Shift word value left by required amount
    tempWord = tempWord << (byteShiftIndex * 8);

    // Fetch memory contents, and calculate mask
    int32_t tempVal = SystemInterlink_readWord(cpu->system, tempAddress);
    int32_t mask = ~(0xFFFFFFFF << (byteShiftIndex * 8));
    tempVal &= mask;

    // Merge contents
    tempWord |= tempVal;

    // Write word to memory
    R3051_writeDataValue(cpu, PHILPSX_R3051_WORD, tempAddress, tempWord);
}

/*
 * This function handles the SWR R3051 instruction.
 */
static void R3051_SWR(R3051 *cpu, int32_t instruction)
{
    // Get rs, rt and immediate
    int32_t immediate = instruction & 0xFFFF;
    int32_t rs = logical_rshift(instruction, 21) & 0x1F;
    int32_t rt = logical_rshift(instruction, 16) & 0x1F;

    // Calculate address
    int64_t address = cpu->generalRegisters[rs] & 0xFFFFFFFFL;
    if ((immediate & 0x8000) == 0x8000) {
        immediate |= 0xFFFF0000;
    }
    address += immediate;

    // Check if address is allowed, trigger exception if not
    if (!Cop0_isAddressAllowed(&cpu->sccp, (int32_t)address)) {
        int64_t tempAddress = cpu->programCounter & 0xFFFFFFFFL;
        tempAddress -= 4;
        cpu->exception.badAddress = (int32_t)address;
        cpu->exception.exceptionReason = PHILPSX_EXCEPTION_ADES;
        cpu->exception.isInBranchDelaySlot = cpu->prevWasBranch;
        cpu->exception.programCounterOrigin
                = cpu->exception.isInBranchDelaySlot ?
                    (int32_t)tempAddress : cpu->programCounter;
        return;
    }

    // Align address, fetch word, and store shift index - sort byte order too
    int32_t tempAddress = (int32_t)(address & 0xFFFFFFFC);
    int32_t byteShiftIndex = (int32_t)(address & 0x3);
    int32_t tempWord = cpu->generalRegisters[rt];

    // Swap byte order
    tempWord = R3051_swapWordEndianness(cpu, tempWord);

    // Shift word value right by required amount
    tempWord = logical_rshift(tempWord, (byteShiftIndex * 8));

    // Fetch rt contents, and calculate mask
    int32_t tempVal = SystemInterlink_readWord(cpu->system, tempAddress);
    int32_t mask = ~logical_rshift(0xFFFFFFFF, (byteShiftIndex * 8));
    tempVal &= mask;

    // Merge contents
    tempWord |= tempVal;

    // Write word to main memory
    R3051_writeDataValue(cpu, PHILPSX_R3051_WORD, tempAddress, tempWord);
}

/*
 * This function handles the SYSCALL R3051 instruction.
 */
static void R3051_SYSCALL(R3051 *cpu, int32_t instruction)
{
    // Trigger System Call Exception
    int64_t tempAddress = cpu->programCounter & 0xFFFFFFFFL;
    tempAddress -= 4;
    cpu->exception.exceptionReason = PHILPSX_EXCEPTION_SYS;
    cpu->exception.isInBranchDelaySlot = cpu->prevWasBranch;
    cpu->exception.programCounterOrigin
            = cpu->exception.isInBranchDelaySlot ?
                (int32_t)tempAddress : cpu->programCounter;
}

/*
 * This function handles the XOR R3051 instruction.
 */
static void R3051_XOR(R3051 *cpu, int32_t instruction)
{
    // Get rs, rt and rd
    int32_t rs = logical_rshift(instruction, 21) & 0x1F;
    int32_t rt = logical_rshift(instruction, 16) & 0x1F;
    int32_t rd = logical_rshift(instruction, 11) & 0x1F;

    // Bitwise XOR rsVal and rtVal, storing result
    cpu->generalRegisters[rd] =
            cpu->generalRegisters[rs] ^ cpu->generalRegisters[rt];
    cpu->generalRegisters[0] = 0;
}

/*
 * This function handles the XORI R3051 instruction.
 */
static void R3051_XORI(R3051 *cpu, int32_t instruction)
{
    // Get rs, rt and immediate
    int32_t immediate = instruction & 0xFFFF;
    int32_t rs = logical_rshift(instruction, 21) & 0x1F;
    int32_t rt = logical_rshift(instruction, 16) & 0x1F;

    // Zero extending immediate is already done for us
    // so just XOR with rsVal and store result
    cpu->generalRegisters[rt] = immediate ^ cpu->generalRegisters[rs];
    cpu->generalRegisters[0] = 0;
}
    
     */
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