// SPDX-License-Identifier: GPL-3.0
// cp0.rs - Copyright Phillip Potter, 2025, under GPLv3 only.

use philpsx_utility::CustomInteger;

/// The CP0 structure models the System Control Co-processor (CP0), which
/// is responsible for mememory management and exceptions.
pub struct CP0 {

    // Register definitions.
    cp_registers: [i32; 32],

    // Condition line.
    condition_line: bool,
}

impl CP0 {

    /// Creates a new CP0 object with the correct initial state.
    pub fn new() -> Self {

        let mut cp0 = CP0 {
            
            // Zero out all registers.
            cp_registers: [0; 32],

            // Set condition line to false.
            condition_line: false,
        };

        // Reset the CP0 object.
        cp0.reset();

        // Now return it.
        cp0        
    }

    /// This function resets the state of the co-processor as per the reset exception.
    fn reset(&mut self) {

        // Set random register to 63.
        self.cp_registers[1] = 63 << 8;

        // Set BEV and TS bits of status register to 0 and 0 (BEV should be 1 but
        // PSX doesn't run this way, TS should be 1 but other emulators don't seem
        // to do this).
        self.cp_registers[12] &= 0xFF9FFFFF_u32 as i32;

        // Set SWc, KUc and IEc bits of status register to 0.
        self.cp_registers[12] &= 0xFFFDFFFC_u32 as i32;

        // Set condition line to false.
        self.condition_line = false;
    }

    /// This function gets the state of the condition line.
    pub fn get_condition_line_status(&self) -> bool {
        self.condition_line
    }

    /// This function sets the state of the condition line.
    pub fn set_condition_line_status(&mut self, status: bool) {
        self.condition_line = status;
    }

    /// This function executes the RFE CP0 instruction.
    pub fn rfe(&mut self) {

        // Shift KUo/IEo/KUp/IEp bits into place of KUp/IEp/KUc/IEc bits and write back.
        let temp_reg = self.read_reg(12);
        let new_bits = temp_reg.logical_rshift(2) & 0xF;

        self.write_reg(12, (temp_reg & (0xFFFFFFF0_u32 as i32)) | new_bits, false);
    }

    /// This function returns the reset exception vector's virtual address.
    pub fn get_reset_exception_vector(&self) -> i32 {
        0xBFC00000_u32 as i32
    }

    /// This function returns the general exception vector's virtual address.
    pub fn get_general_exception_vector(&self) -> i32 {

        // Isolate BEV bit and return accordingly.
        let bev = (self.cp_registers[12] & 0x00400000).logical_rshift(22) != 0;

        if bev {
            0xBFC00180_u32 as i32
        } else {
            0x80000080_u32 as i32
        }
    }

    /// This function reads from a given register.
    pub fn read_reg(&self, reg: i32) -> i32 {

        // Determine which register we are reading.
        let array_index = reg as usize;
        match array_index {

            // Status register.
            12 => {
                // Mask out 0-read bits.
                self.cp_registers[array_index] & (0xF27FFF3F_u32 as i32)

                // We could also merge in TS bit (commented out to copy observed
                // behaviour of other emulators).
                //(self.cp_registers[array_index] & 0xF27FFF3F) | 0x00200000
            },

            // Cause register.
            13 => {
                // Mask out 0-read bits.
                self.cp_registers[array_index] & (0xB000FF7Cu32 as i32)
            },

            // PrId register.
            15 => {
                // PSX specific value.
                0x00000002
            },

            // A match on all of the following registers should just directly
            // return the value we want:
            //
            // 1:  Random register.
            // 8:  Bad virtual address register.
            // 14: Exception PC register.
            1 | 8 | 14 => self.cp_registers[array_index],

            // Return 0 for all other registers.
            _ => 0,
        }
    }

    /// This function writes to a given register. It allows override of write protection
    /// on certain bits.
    pub fn write_reg(&mut self, reg: i32, value: i32, write_override: bool) {

        // Determine which register we are writing.
        let array_index = reg as usize;
        match write_override {

            // Override was specified, just write register directly.
            true => {
                self.cp_registers[array_index] = value;
            },

            false => {
                match array_index {

                    // Status register.
                    12 => {
                        // Mask out writable bits in existing register value.
                        let temp_val = self.cp_registers[array_index] & 0x0DB400C0;

                        // Mask out read-only bits in supplied value, merge with previously
                        // masked contents, and store back.
                        self.cp_registers[array_index] = (value & (0xF24BFF3F_u32 as i32)) | temp_val;
                    },

                    // Cause register.
                    13 => {
                        // Mask out writable bits in existing register value.
                        let temp_val = self.cp_registers[array_index] & (0xFFFFFCFF_u32 as i32);

                        // Mask out read-only bits in supplied value, merg with previously
                        // masked contents, and store back.
                        self.cp_registers[array_index] = (value & 0x00000300) | temp_val;
                    },

                    // For all other registers, just write the value back as-is.
                    _ => {
                        self.cp_registers[array_index] = value;
                    },
                }
            }
        }
    }

    /// This function sets or resets the cache miss flag.
    pub fn set_cache_miss(&mut self, value: bool) {

        // Determine value to merge in.
        let cm_flag = if value {
            0x00080000
        } else {
            0
        };

        // Merge flag into status register and write it back.
        let status_reg = (self.cp_registers[12] & (0xFFF7FFFF_u32 as i32)) | cm_flag;
        self.write_reg(12, status_reg, true);
    }

    /// This function transforms a virtual address into a physical one.
    /// It is designed for the base model of the processor which has no TLB.
    pub fn virtual_to_physical(&self, virtual_address: i32) -> i32 {

        // Declare temporary i64 variable to hold new address and mask the
        // top 32 bits so we always get an unsigned value.
        let mut physical_address = (virtual_address as i64) & 0xFFFFFFFF_i64;

        // Make correct modifications to address.
        match physical_address {

            // Address ranges (exclusive of end address):
            // kuseg address: 0x00000000 to 0x80000000 (do nothing).
            // kseg0 address: 0x80000000 to 0xA0000000 (subtract 0x80000000).
            // kseg1 address: 0xA0000000 to 0xC0000000 (subtract 0xA0000000).
            // kseg2 address: 0xC0000000 to end (do nothing - only includes cache control on PSX).
            0x80000000..0xA0000000 => {
                physical_address -= 0x80000000;
            },
            0xA0000000..0xC0000000 => {
                physical_address -= 0xA0000000;
            },
            _ => (),
        }

        physical_address as i32
    }

    /// This function determines whetheer the supplied virtual address is cacheable.
    pub fn is_cacheable(&self, virtual_address: i32) -> bool {

        // Declare temporary i64 variable to hold new address and mask the
        // top 32 bits so we always get an unsigned value.
        let temp_address = (virtual_address as i64) & 0xFFFFFFFF_i64;

        (0x00000000..0xA0000000).contains(&temp_address)
    }

    /// This function determines whether or not we are in kernel mode.
    pub fn are_we_in_kernel_mode(&self) -> bool {
        (self.cp_registers[12] & 0x02) == 0
    }

    /// This function tells us if opposite byte ordering is in effect in user mode.
    pub fn user_mode_opposite_byte_ordering(&self) -> bool {
        (self.cp_registers[12] & 0x02000000) == 0x02000000
    }

    /// This function allows us to check if a virtual address is allowed to be accessed.
    /// It is useful for checking if we are attempting to access a prohibited address
    /// whilst in user mode.
    pub fn is_address_allowed(&self, virtual_address: i32) -> bool {
        (virtual_address & (0x80000000_u32 as i32)) == 0 || self.are_we_in_kernel_mode()
    }

    /// This function tells us if the caches have been swapped. It is currently hardcoded
    /// to false.
    pub fn are_caches_swapped(&self) -> bool {

        // Commented out (cache swapping hardcoded off)
        //self.cp_registers[12] & 0x00020000 == 0x00020000

        false
    }

    /// This function tells us if the data cache is isolated.
    pub fn is_data_cache_isolated(&self) -> bool {
        self.cp_registers[12] & 0x00010000 == 0x00010000
    }

    /// This function tells us if a co-processor is usable.
    pub fn is_co_processor_usable(&self, co_processor_num: i32) -> bool {

        let usable_flags = self.cp_registers[12].logical_rshift(28);
        usable_flags.logical_rshift(co_processor_num) & 0x1 == 1
    }
}

#[cfg(test)]
mod tests;