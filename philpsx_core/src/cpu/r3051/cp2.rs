// SPDX-License-Identifier: GPL-3.0
// cp2.rs - Copyright Phillip Potter, 2025, under GPLv3 only.

use philpsx_utility::{CustomInteger, sign_extend};

// Unsigned Newton-Raphson algorithm array - values taken from NOPSX
// documentation to mimic NOPSX results (but with my own code of course).
const UNR_RESULTS: [i32; 257] = [
    0xFF, 0xFD, 0xFB, 0xF9, 0xF7, 0xF5, 0xF3, 0xF1, 0xEF, 0xEE, 0xEC, 0xEA,
    0xE8, 0xE6, 0xE4, 0xE3, 0xE1, 0xDF, 0xDD, 0xDC, 0xDA, 0xD8, 0xD6, 0xD5,
    0xD3, 0xD1, 0xD0, 0xCE, 0xCD, 0xCB, 0xC9, 0xC8, 0xC6, 0xC5, 0xC3, 0xC1,
    0xC0, 0xBE, 0xBD, 0xBB, 0xBA, 0xB8, 0xB7, 0xB5, 0xB4, 0xB2, 0xB1, 0xB0,
    0xAE, 0xAD, 0xAB, 0xAA, 0xA9, 0xA7, 0xA6, 0xA4, 0xA3, 0xA2, 0xA0, 0x9F,
    0x9E, 0x9C, 0x9B, 0x9A, 0x99, 0x97, 0x96, 0x95, 0x94, 0x92, 0x91, 0x90,
    0x8F, 0x8D, 0x8C, 0x8B, 0x8A, 0x89, 0x87, 0x86, 0x85, 0x84, 0x83, 0x82,
    0x81, 0x7F, 0x7E, 0x7D, 0x7C, 0x7B, 0x7A, 0x79, 0x78, 0x77, 0x75, 0x74,
    0x73, 0x72, 0x71, 0x70, 0x6F, 0x6E, 0x6D, 0x6C, 0x6B, 0x6A, 0x69, 0x68,
    0x67, 0x66, 0x65, 0x64, 0x63, 0x62, 0x61, 0x60, 0x5F, 0x5E, 0x5D, 0x5D,
    0x5C, 0x5B, 0x5A, 0x59, 0x58, 0x57, 0x56, 0x55, 0x54, 0x53, 0x53, 0x52,
    0x51, 0x50, 0x4F, 0x4E, 0x4D, 0x4D, 0x4C, 0x4B, 0x4A, 0x49, 0x48, 0x48,
    0x47, 0x46, 0x45, 0x44, 0x43, 0x43, 0x42, 0x41, 0x40, 0x3F, 0x3F, 0x3E,
    0x3D, 0x3C, 0x3C, 0x3B, 0x3A, 0x39, 0x39, 0x38, 0x37, 0x36, 0x36, 0x35,
    0x34, 0x33, 0x33, 0x32, 0x31, 0x31, 0x30, 0x2F, 0x2E, 0x2E, 0x2D, 0x2C,
    0x2C, 0x2B, 0x2A, 0x2A, 0x29, 0x28, 0x28, 0x27, 0x26, 0x26, 0x25, 0x24,
    0x24, 0x23, 0x22, 0x22, 0x21, 0x20, 0x20, 0x1F, 0x1E, 0x1E, 0x1D, 0x1D,
    0x1C, 0x1B, 0x1B, 0x1A, 0x19, 0x19, 0x18, 0x18, 0x17, 0x16, 0x16, 0x15,
    0x15, 0x14, 0x14, 0x13, 0x12, 0x12, 0x11, 0x11, 0x10, 0x0F, 0x0F, 0x0E,
    0x0E, 0x0D, 0x0D, 0x0C, 0x0C, 0x0B, 0x0A, 0x0A, 0x09, 0x09, 0x08, 0x08,
    0x07, 0x07, 0x06, 0x06, 0x05, 0x05, 0x04, 0x04, 0x03, 0x03, 0x02, 0x02,
    0x01, 0x01, 0x00, 0x00, 0x00
];

/// The CP2 structure models the Geometry Transformation Engine, which is a
/// co-processor in the PlayStation responsible for matrix calculations amongst
/// other things.
pub struct CP2 {

    // Control registers.
    control_registers: [i32; 32],

    // Data registers.
    data_registers: [i32; 32],

    // Condition line.
    condition_line: bool,
}

impl CP2 {

    /// Creates a new CP2 object with the correct initial state.
    pub fn new() -> Self {
        CP2 {
            
            // Zero-out both register arrays.
            control_registers: [0; 32],
            data_registers: [0; 32],

            // Set condition line to false.
            condition_line: false,
        }
    }

    /// This function just resets the condition line.
    pub fn reset(&mut self) {
        self.condition_line = false;
    }

    /// This function gets the state of the condition line.
    pub fn get_condition_line_status(&self) -> bool {
        self.condition_line
    }

    /// This function reads from the specified control register.
    pub fn read_control_reg(&self, reg: i32) -> i32 {

        // Determine which register we are reading.
        let array_index = reg as usize;
        match array_index {

            // For range 26-30 inclusive, return the register but
            // also sign extend if necessary. Register 26 should
            // actually be unsigned, but due to a hardware bug it
            // isn't, so we should preserve this behaviour here.
            26..=30 => sign_extend(self.control_registers[array_index]),

            // Return actual value.
            _ => self.control_registers[array_index]
        }
    }

    /// This function reads from the specified data register.
    pub fn read_data_reg(&self, reg: i32) -> i32 {

        // Determine which register we are reading.
        let array_index = reg as usize;
        match array_index {

            // Read and sign extend if necessary.
            1 | 3 | 5 | 8 | 9 | 10 | 11 =>
                sign_extend(self.data_registers[array_index]),

            // Return 0 for these.
            23 | 28 => 0,

            // Combine registers 9, 10 and 11 accordingly.
            29 => ((self.data_registers[11] << 3) | 0x7C00) |
                (self.data_registers[10].logical_rshift(2) & 0x3E0) |
                (self.data_registers[9].logical_rshift(7) & 0x1F),

            // LZCR
            31 => {

                // Determine whether we are counting leading 1s of 0s.
                let mut lzcs = self.data_registers[30];
                let bit = lzcs & 0x80000000_u32 as i32;

                let mut temp = 0;
                for _ in 0..32 {
                    if (lzcs & 0x80000000u32 as i32) == bit {
                        temp += 1;
                    } else {
                        break;
                    }

                    lzcs <<= 1;
                }

                temp
            },

            // Return actual value.
            _ => self.data_registers[array_index]
        }
    }

    /// This function writes to the specified control register.
    pub fn write_control_reg(&mut self, reg: i32, value: i32, _write_override: bool) {

        // For now, ignore override and just write to anywhere requested.
        self.control_registers[reg as usize] = value;
    }

    /// This function writes to the specified data register.
    pub fn write_data_reg(&mut self, reg: i32, value: i32, write_override: bool) {

        // Determine which register we are writing.
        let array_index = reg as usize;
        match write_override {

            // Override was specified, just write register directly.
            true => {
                self.data_registers[array_index] = value;
            },

            false => {
                match array_index {

                    // For these registers, do nothing.
                    7 | 23 | 29 | 31 => {},

                    // SXY2 - mirror of SXYP.
                    14 => {

                        // Set SXY2 and SXYP.
                        self.data_registers[14] = value; // SXY2
                        self.data_registers[15] = value; // SXYP
                    },

                    // SXYP - mirror of SXY2 but causes SXY1 to move to SXY0,
                    // and SXY2 to move to SXY1.
                    15 => {

                        // Move SXY1 to SXY0.
                        self.data_registers[12] = self.data_registers[13];

                        // Move SXY2 to SXY1.
                        self.data_registers[13] = self.data_registers[14];

                        // Set SXY2 and SXYP.
                        self.data_registers[14] = value; // SXY2
                        self.data_registers[15] = value; // SXYP
                    },

                    // IRGB
                    28 => {
                        self.data_registers[9] = (0x1F & value) << 7;                 // IR1
                        self.data_registers[10] = (0x3E0 & value) << 2;               // IR2
                        self.data_registers[11] = (0x7C00 & value).logical_rshift(3); // IR3
                    },

                    // For all other registers, just write the value back as-is.
                    _ => {
                        self.data_registers[array_index] = value;
                    },
                }
            }
        }
    }

    /// This function deals with GTE functions that are invoked on CP2 from the CPU.
    /// It calls the correct private function and determines the number of cycles it
    /// should take.
    pub fn gte_function(&mut self, opcode: i32) -> i32 {

        // Match on the correct function and call it, returning the
        // correct number of cycles.
        match opcode & 0x3F {

            0x01 => {
                self.handle_rtps(opcode);
                15
            },

            0x06 => {
                self.handle_nclip(opcode);
                8
            },

            0x0C => {
                self.handle_op(opcode);
                6
            },

            0x10 => {
                self.handle_dpcs(opcode);
                8
            },

            0x11 => {
                self.handle_intpl(opcode);
                8
            },

            0x12 => {
                self.handle_mvmva(opcode);
                8
            },

            0x13 => {
                self.handle_ncds(opcode);
                19
            },

            0x14 => {
                self.handle_cdp(opcode);
                13
            },

            0x16 => {
                self.handle_ncdt(opcode);
                44
            },

            0x1B => {
                self.handle_nccs(opcode);
                17
            },

            0x1C => {
                self.handle_cc(opcode);
                11
            },

            0x1E => {
                self.handle_ncs(opcode);
                14
            },

            0x20 => {
                self.handle_nct(opcode);
                30
            },

            0x28 => {
                self.handle_sqr(opcode);
                5
            },

            0x29 => {
                self.handle_dcpl(opcode);
                8
            },

            0x2A => {
                self.handle_dpct(opcode);
                17
            },

            0x2D => {
                self.handle_avsz3(opcode);
                5
            },

            0x2E => {
                self.handle_avsz4(opcode);
                6
            },

            0x30 => {
                self.handle_rtpt(opcode);
                23
            },

            0x3D => {
                self.handle_gpf(opcode);
                5
            },

            0x3E => {
                self.handle_gpl(opcode);
                5
            },

            0x3F => {
                self.handle_ncct(opcode);
                39
            },


            // Do nothing and return 0 cycles.
            _ => 0
        }
    }

    /// This function handles the RTPS GTE function.
    fn handle_rtps(&mut self, opcode: i32) {

    }

    /// This function handles the NCLIP GTE function.
    fn handle_nclip(&mut self, opcode: i32) {

    }

    /// This function handles the OP GTE function.
    fn handle_op(&mut self, opcode: i32) {

    }

    /// This function handles the DPCS GTE function.
    fn handle_dpcs(&mut self, opcode: i32) {

    }

    /// This function handles the INTPL GTE function.
    fn handle_intpl(&mut self, opcode: i32) {

    }

    /// This function handles the MVMVA GTE function.
    fn handle_mvmva(&mut self, opcode: i32) {

    }

    /// This function handles the NCDS GTE function.
    fn handle_ncds(&mut self, opcode: i32) {

    }

    /// This function handles the CDP GTE function.
    fn handle_cdp(&mut self, opcode: i32) {

    }

    /// This function handles the NCDT GTE function.
    fn handle_ncdt(&mut self, opcode: i32) {

    }

    /// This function handles the NCCS GTE function.
    fn handle_nccs(&mut self, opcode: i32) {

    }

    /// This function handles the CC GTE function.
    fn handle_cc(&mut self, opcode: i32) {

    }

    /// This function handles the NCS GTE function.
    fn handle_ncs(&mut self, opcode: i32) {

    }

    /// This function handles the NCT GTE function.
    fn handle_nct(&mut self, opcode: i32) {

    }

    /// This function handles the SQR GTE function.
    fn handle_sqr(&mut self, opcode: i32) {

    }

    /// This function handles the DCPL GTE function.
    fn handle_dcpl(&mut self, opcode: i32) {

    }

    /// This function handles the DPCT GTE function.
    fn handle_dpct(&mut self, opcode: i32) {

    }

    /// This function handles the AVSZ3 GTE function.
    fn handle_avsz3(&mut self, opcode: i32) {

    }

    /// This function handles the AVSZ4 GTE function.
    fn handle_avsz4(&mut self, opcode: i32) {

    }

    /// This function handles the RTPT GTE function.
    fn handle_rtpt(&mut self, opcode: i32) {

    }

    /// This function handles the GPF GTE function.
    fn handle_gpf(&mut self, opcode: i32) {

    }

    /// This function handles the GPL GTE function.
    fn handle_gpl(&mut self, opcode: i32) {

    }

    /// This function handles the NCCT GTE function.
    fn handle_ncct(&mut self, opcode: i32) {

    }

}

#[cfg(test)]
mod tests;