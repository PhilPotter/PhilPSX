// SPDX-License-Identifier: GPL-3.0
// cp2.rs - Copyright Phillip Potter, 2025, under GPLv3 only.

use philpsx_utility::{CustomInteger, min};
use math::{CP2Matrix, CP2Vector};

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

/// This enum tells various common functions whether the single or triple variant
/// should be executed.
enum InstructionVariant {
    Single,
    Triple,
}

/// This enum repesents flag register fields that are larger or smaller than some
/// given boundary, and should trigger a flag register field to be set in such a case.
/// These particular fields to not lead to saturation of a given value, purely flag setting.
enum UnsaturatedFlagRegisterField {
    MAC0,
    MAC1,
    MAC2,
    MAC3,
}
use UnsaturatedFlagRegisterField::*;

/// This enum represents flag register fields that are larger or smaller than some
/// given boundary, and should trigger a flag register to be set as well as a saturated
/// value to be returned.
enum SaturatedFlagRegisterField {
    IR0,
    IR1,
    IR2,
    IR3,
    IR3Quirk,
    ColourFifoR,
    ColourFifoG,
    ColourFifoB,
    SX2,
    SY2,
    SZ3
}
use SaturatedFlagRegisterField::*;

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
            26..=30 => self.control_registers[array_index].sign_extend(15),

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
                self.data_registers[array_index].sign_extend(15),

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
                self.handle_common_rtp(opcode, InstructionVariant::Single);
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
                self.handle_common_dpc(opcode, InstructionVariant::Single);
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
                self.handle_common_nc(opcode, InstructionVariant::Single);
                14
            },

            0x20 => {
                self.handle_common_nc(opcode, InstructionVariant::Triple);
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
                self.handle_common_dpc(opcode, InstructionVariant::Triple);
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
                self.handle_common_rtp(opcode, InstructionVariant::Triple);
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

    /// This function implements the functionality for the RTPS and RTPT instructions.
    /// Figured I'm porting/re-writing from C anyway and these are largely identical.
    fn handle_common_rtp(&mut self, opcode: i32, variant: InstructionVariant) {

        // Filter out sf bit.
        let sf = opcode.bit_value(19);

        // Setup translation vector - no sign-extension necessary as we are treating these
        // values as full-width 32-bit values anyway, so sign-extension will happen automatically
        // upon conversion from i32 to i64.
        //
        // In addition, multiply each element by 0x1000.
        let translation_vector = CP2Vector::new(
            (self.control_registers[5] as i64) * 0x1000, // TRX
            (self.control_registers[6] as i64) * 0x1000, // TRY
            (self.control_registers[7] as i64) * 0x1000  // TRZ
        );

        // Setup rotation matrix, sign-extending values as needed.
        let rotation_matrix = CP2Matrix::new(

            // Top row:
            [
                ((self.control_registers[0] & 0xFFFF) as i64).sign_extend(15), // RT11.
                ((self.control_registers[0].logical_rshift(16) & 0xFFFF) as i64).sign_extend(15), // RT12.
                ((self.control_registers[1] & 0xFFFF) as i64).sign_extend(15) // RT13.
            ],

            // Middle row:
            [
                ((self.control_registers[1].logical_rshift(16) & 0xFFFF) as i64).sign_extend(15), // RT21.
                ((self.control_registers[2] & 0xFFFF) as i64).sign_extend(15), // RT22.
                ((self.control_registers[2].logical_rshift(16) & 0xFFFF) as i64).sign_extend(15) // RT23.
            ],

            // Bottom row:
            [
                ((self.control_registers[3] & 0xFFFF) as i64).sign_extend(15), // RT31.
                ((self.control_registers[3].logical_rshift(16) & 0xFFFF) as i64).sign_extend(15), // RT32.
                ((self.control_registers[4] & 0xFFFF) as i64).sign_extend(15) // RT33.
            ]
        );

        // Setup offset and distance values - again only sign-extending when needed.
        let ofx = self.control_registers[24] as i64;
        let ofy = self.control_registers[25] as i64;
        let h = (self.control_registers[26] & 0xFFFF) as i64; // Explicitly avoid sign-extension here.
        let dqa = ((self.control_registers[27] & 0xFFFF) as i64).sign_extend(15);
        let dqb = self.control_registers[28] as i64;

        // Now, we perform the remaining tasks based on the specified number of iterations.
        let iterations = match variant {
            InstructionVariant::Single => 1,
            InstructionVariant::Triple => 3,
        };
        for i in 0..iterations {

            // Clear flag register.
            self.control_registers[31] = 0;

            // Setup vector with one of V0, V1 or V2, sign-extending values as needed.
            let v_any = CP2Vector::new(
                ((self.data_registers[i * 2] & 0xFFFF) as i64).sign_extend(15), // VX.
                ((self.data_registers[i * 2].logical_rshift(16) & 0xFFFF) as i64).sign_extend(15), // VY.
                ((self.data_registers[i * 2 + 1] & 0xFFFF) as i64).sign_extend(15), // VZ.
            );

            // Rotate vector and translate it too. Then, right shift all result values
            // by 12 bits (depending on value of sf bit), while also preserving sign bits.
            let mut mac_results = rotation_matrix * v_any + translation_vector;
            mac_results = CP2Vector::new(
                mac_results.top() >> (sf * 12),
                mac_results.middle() >> (sf * 12),
                mac_results.bottom() >> (sf * 12)
            );

            // Set MAC1, MAC2 and MAC3 flags accordingly.
            let mac1 = mac_results.top();
            let mac2 = mac_results.middle();
            let mac3 = mac_results.bottom();

            // Check bounds.
            self.handle_unsaturated_result(mac1, MAC1);
            self.handle_unsaturated_result(mac2, MAC2);
            self.handle_unsaturated_result(mac3, MAC3);

            // Set IR1, IR2 and IR3 - dealing with flags too,
            // saturation should be -0x8000..0x7FFF, regardless of lm bit.
            // IR3 flag should be handled in quirk mode.
            let ir1 = self.handle_saturated_result(mac1, IR1, false, sf);
            let ir2 = self.handle_saturated_result(mac2, IR2, false, sf);
            let ir3 = self.handle_saturated_result(mac3, IR3Quirk, false, sf);

            // Write back to real registers.
            self.data_registers[25] = mac1 as i32; // MAC1.
            self.data_registers[26] = mac2 as i32; // MAC2.
            self.data_registers[27] = mac3 as i32; // MAC3.
            self.data_registers[9] = ir1 as i32; // IR1.
            self.data_registers[10] = ir2 as i32; // IR2.
            self.data_registers[11] = ir3 as i32; // IR3.

            // Calculate SZ3 and move FIFO along, also setting SZ3 flag if needed.
            self.data_registers[16] = self.data_registers[17]; // SZ1 to SZ0.
            self.data_registers[17] = self.data_registers[18]; // SZ2 to SZ1.
            self.data_registers[18] = self.data_registers[19]; // SZ3 to SZ2.

            let shift_by = (1 - sf) * 12;
            let temp_sz3 = self.handle_saturated_result(mac3 >> shift_by, SZ3, false, sf);
            self.data_registers[19] = temp_sz3 as i32;

            // Begin second phase of calculations - use Unsigned Newton-Raphson
            // division algorithm from NOPSX documentation.
            let division_result = if h < temp_sz3 * 2 {
                // Count leading zeroes in SZ3, from bit 15
                // as it's saturated to 16-bit value.
                let z = temp_sz3.leading_zeroes(15);

                let mut division_result = h << z;
                let mut d = temp_sz3 << z;
                let u = UNR_RESULTS[((d as i32) - 0x7FC0).logical_rshift(7) as usize] + 0x101;
                d = (0x2000080 - (d * (u as i64))).logical_rshift(8);
                d = (0x80 + (d * (u as i64))).logical_rshift(8);
                division_result = min(0x1FFFF, ((division_result * d) + 0x8000).logical_rshift(16));

                division_result
            } else {
                self.control_registers[31] |= 0x20000;
                0x1FFFF
            };

            // Use division result and set MAC0 flag if needed.
            // Also handle SX2/SY2/IR0 saturation and flags as needed.
            let mut mac0 = division_result * ir1 + ofx;
            self.handle_unsaturated_result(mac0, MAC0);
            mac0 &= 0xFFFFFFFF;
            mac0 = mac0.sign_extend(31);

            let sx2 = self.handle_saturated_result(mac0 / 0x10000, SX2, false, sf);
            mac0 = division_result * ir2 + ofy;
            self.handle_unsaturated_result(mac0, MAC0);
            mac0 &= 0xFFFFFFFF;
            mac0 = mac0.sign_extend(31);

            let sy2 = self.handle_saturated_result(mac0 / 0x10000, SY2, false, sf);
            mac0 = division_result * dqa + dqb;
            self.handle_unsaturated_result(mac0, MAC0);
            mac0 &= 0xFFFFFFFF;
            mac0.sign_extend(31);

            let ir0 = self.handle_saturated_result(mac0 / 0x1000, IR0, false, sf);

            // Store values back to correct registers.

            // SXY FIFO registers.
            self.data_registers[12] = self.data_registers[13]; // SXY1 to SXY0.
            self.data_registers[13] = self.data_registers[14]; // SXY2 to SXY1.
            self.data_registers[14] = (((sy2 as i32) & 0xFFFF) << 16) | ((sx2 as i32) & 0xFFFF); // SXY2.
            self.data_registers[15] = self.data_registers[14]; // SXYP mirror of SXY2.

            // MAC0.
            self.data_registers[24] = mac0 as i32;

            // IR0.
            self.data_registers[8] = ir0 as i32;
        }

        // Calculate bit 31 of flag register.
        if (self.control_registers[31] & 0x7F87E000) != 0 {
            self.control_registers[31] |= 0x80000000_u32 as i32;
        }
    }

    /// This function handles the NCLIP GTE function.
    fn handle_nclip(&mut self, opcode: i32) {

        // Clear flag register.
        self.control_registers[31] = 0;

        // Retrieve SXY values, sign extending if necessary.
        let sx0 = ((self.data_registers[12] & 0xFFFF) as i64).sign_extend(15); // SX0.
        let sy0 = ((self.data_registers[12].logical_rshift(16) & 0xFFFF) as i64).sign_extend(15); // SY0.
        let sx1 = ((self.data_registers[13] & 0xFFFF) as i64).sign_extend(15); // SX1.
        let sy1 = ((self.data_registers[13].logical_rshift(16) & 0xFFFF) as i64).sign_extend(15); // SY1.
        let sx2 = ((self.data_registers[14] & 0xFFFF) as i64).sign_extend(15); // SX2.
        let sy2 = ((self.data_registers[14].logical_rshift(16) & 0xFFFF) as i64).sign_extend(15); // SY2.

        // Perform calculation.
        let mac0 = sx0 * sy1 + sx1 * sy2 + sx2 * sy0 - sx0 * sy2 - sx1 * sy0 - sx2 * sy1;

        // Check and set flags if needed.
        self.handle_unsaturated_result(mac0, MAC0);

        // Calculate flag bit 31.
        if (self.control_registers[31] & 0x7F87E000) != 0 {
            self.control_registers[31] |= 0x80000000_u32 as i32;
        }

        // Store MAC0 result back to register.
        self.data_registers[24] = mac0 as i32;
    }

    /// This function handles the OP GTE function.
    fn handle_op(&mut self, opcode: i32) {

        // Clear flag register.
        self.control_registers[31] = 0;

        // Filter out sf bit.
        let sf = opcode.bit_value(19);

        // Get lm bit status.
        let lm = opcode.bit_is_set(10);

        // Fetch IR values, sign extending as necessary.
        let ir1 = ((self.data_registers[9] & 0xFFFF) as i64).sign_extend(15); // IR1.
        let ir2 = ((self.data_registers[10] & 0xFFFF) as i64).sign_extend(15); // IR2.
        let ir3 = ((self.data_registers[11] & 0xFFFF) as i64).sign_extend(15); // IR3.

        // Fetch RT11, RT22, RT33 values, sign extending as necessary.
        let d1 = ((self.control_registers[0] & 0xFFFF) as i64).sign_extend(15); // RT11.
        let d2 = ((self.control_registers[2] & 0xFFFF) as i64).sign_extend(15); // RT22.
        let d3 = ((self.control_registers[4] & 0xFFFF) as i64).sign_extend(15); // RT33.

        // Perform calculation, and shift result right by (sf * 12), preserving sign bit.
        let temp1 = (ir3 * d2 - ir2 * d3) >> (sf * 12);
        let temp2 = (ir1 * d3 - ir3 * d1) >> (sf * 12);
        let temp3 = (ir2 * d1 - ir1 * d2) >> (sf * 12);

        // Store results in MAC1, MAC2 and MAC3 registers.
        self.data_registers[25] = temp1 as i32; // MAC1.
        self.data_registers[26] = temp2 as i32; // MAC2.
        self.data_registers[27] = temp3 as i32; // MAC3.

        // Set relevant MAC1, MAC2 and MAC3 flag bits.
        self.handle_unsaturated_result(temp1, MAC1);
        self.handle_unsaturated_result(temp2, MAC2);
        self.handle_unsaturated_result(temp3, MAC3);

        // Set IR1, IR2 and IR3 registers and saturation flag bits.
        // Determine the lower saturation bound using lm bit status.
        // Upper bound is always 0x7FFF.
        self.data_registers[9] = self.handle_saturated_result(temp1, IR1, lm, sf) as i32;
        self.data_registers[10] = self.handle_saturated_result(temp2, IR2, lm, sf) as i32;
        self.data_registers[11] = self.handle_saturated_result(temp3, IR3, lm, sf) as i32;

        // Calculate bit 31 of flag register.
        if (self.control_registers[31] & 0x7F87E000) != 0 {
            self.control_registers[31] |= 0x80000000_u32 as i32;
        }
    }

    /// This function implements the functionality for the DCPS and DCPT instructions.
    /// Figured I'm porting/re-writing from C anyway and these are largely identical.
    fn handle_common_dpc(&mut self, opcode: i32, variant: InstructionVariant) {

        // Filter out sf bit.
        let sf = opcode.bit_value(19);

        // Get lm bit status.
        let lm = opcode.bit_is_set(10);

        // Retrieve IR0 value, sign extending as needed.
        let ir0 = ((self.data_registers[8] & 0xFFFF) as i64).sign_extend(15); // IR0.

        // Retrieve far colour values - let natural sign extension happen.
        let rfc = self.control_registers[21] as i64;
        let gfc = self.control_registers[22] as i64;
        let bfc = self.control_registers[23] as i64;

        // Now, we perform the remaining tasks based on the specified number of iterations.
        let iterations = match variant {
            InstructionVariant::Single => 1,
            InstructionVariant::Triple => 3,
        };
        for i in 0..iterations {

            // Clear flag register.
            self.control_registers[31] = 0;

            // Retrieve RGB values from either RGBC (single) or RGB0 (triple) register,
            // depending on variant passed to common function.
            // Also retrieve CODE value, but always from RGBC.
            let colour_register_index = match variant {
                InstructionVariant::Single => 6,
                InstructionVariant::Triple => 20,
            };
            let (r, g, b, code) = (
                (self.data_registers[colour_register_index] & 0xFF) as i64, // R or R0.
                ((self.data_registers[colour_register_index].logical_rshift(8) & 0xFF) as i64), // G or G0.
                ((self.data_registers[colour_register_index].logical_rshift(16) & 0xFF) as i64), // B or B0.
                ((self.data_registers[6].logical_rshift(24) & 0xFF) as i64) // CODE.
            );

            // This left-shifting should happen for both DPCS and DPCT,
            // my comment was misleading in the original version.
            let mut mac1 = r << 16;
            let mut mac2 = g << 16;
            let mut mac3 = b << 16;

            // Check for and set MAC1, MAC2 and MAC3 flags if needed.
            self.handle_unsaturated_result(mac1, MAC1);
            self.handle_unsaturated_result(mac2, MAC2);
            self.handle_unsaturated_result(mac3, MAC3);

            // Perform first common stage of calculation.
            // Saturate IR1, IR2 and IR3 results, setting flags as needed.
            // Ignore lm bit for this first set of writes.
            let mut ir1 = self.handle_saturated_result(((rfc << 12) - mac1) >> (sf * 12), IR1, false, sf);
            let mut ir2 = self.handle_saturated_result(((gfc << 12) - mac2) >> (sf * 12), IR2, false, sf);
            let mut ir3 = self.handle_saturated_result(((bfc << 12) - mac3) >> (sf * 12), IR3, false, sf);

            // Continue first common stage of calculation.
            mac1 += ir1 * ir0;
            mac2 += ir2 * ir0;
            mac3 += ir3 * ir0;

            // Check for and set MAC1, MAC2 and MAC3 flags again if needed.
            self.handle_unsaturated_result(mac1, MAC1);
            self.handle_unsaturated_result(mac2, MAC2);
            self.handle_unsaturated_result(mac3, MAC3);

            // Shift MAC1, MAC2 and MAC3 by (sf * 12) bits, preserving sign bit.
	        mac1 >>= sf * 12;
	        mac2 >>= sf * 12;
	        mac3 >>= sf * 12;

            // Check for and set MAC1, MAC2 and MAC3 flags again if needed.
            self.handle_unsaturated_result(mac1, MAC1);
            self.handle_unsaturated_result(mac2, MAC2);
            self.handle_unsaturated_result(mac3, MAC3);

            // Store MAC1, MAC2 and MAC3 to IR1, IR2 and IR3, saturating as needed.
            ir1 = self.handle_saturated_result(mac1, IR1, lm, sf);
            ir2 = self.handle_saturated_result(mac2, IR2, lm, sf);
            ir3 = self.handle_saturated_result(mac3, IR3, lm, sf);

            // Generate colour FIFO values and check/set flags as needed.
            let r_out = self.handle_saturated_result(mac1 / 16, ColourFifoR, lm, sf);
            let g_out = self.handle_saturated_result(mac2 / 16, ColourFifoG, lm, sf);
            let b_out = self.handle_saturated_result(mac3 / 16, ColourFifoB, lm, sf);

            // Calculate flag bit 31.
            if (self.control_registers[31] & 0x7F87E000) != 0 {
                self.control_registers[31] |= 0x80000000_u32 as i32;
            }

            // Store values back to registers.
            self.data_registers[25] = mac1 as i32; // MAC1.
            self.data_registers[26] = mac2 as i32; // MAC2.
            self.data_registers[27] = mac3 as i32; // MAC3.

            self.data_registers[9] = ir1 as i32;   // IR1.
            self.data_registers[10] = ir2 as i32;  // IR2.
            self.data_registers[11] = ir3 as i32;  // IR3.

            self.data_registers[20] = self.data_registers[21]; // RGB1 to RGB0.
            self.data_registers[21] = self.data_registers[22]; // RGB2 to RGB1.
            self.data_registers[22] = ((code << 24) | (b_out << 16) | (g_out << 8) | r_out) as i32; //RGB2.
        }
    }

    /// This function handles the INTPL GTE function.
    fn handle_intpl(&mut self, opcode: i32) {

        // Clear flag register.
        self.control_registers[31] = 0;

        // Filter out sf bit.
        let sf = opcode.bit_value(19);

        // Get lm bit status.
        let lm = opcode.bit_is_set(10);

        // Retrieve R0, IR1, IR2 and IR3, sign extending as necessary.
        let ir0 = ((self.data_registers[8] & 0xFFFF) as i64).sign_extend(15);
        let mut ir1 = ((self.data_registers[9] & 0xFFFF) as i64).sign_extend(15);
        let mut ir2 = ((self.data_registers[10] & 0xFFFF) as i64).sign_extend(15);
        let mut ir3 = ((self.data_registers[11] & 0xFFFF) as i64).sign_extend(15);

        // Retrieve far colour values, sign extending naturally.
        let rfc = self.control_registers[21] as i64;
        let gfc = self.control_registers[22] as i64;
        let bfc = self.control_registers[23] as i64;

        // Fetch code value from RGBC.
        let code = self.data_registers[6].logical_rshift(24) as i64;

        // Perform INTPL-only calculation.
        let mut mac1 = ir1 << 12;
        let mut mac2 = ir2 << 12;
        let mut mac3 = ir3 << 12;

        // Handle MAC1, MAC2 and MAC3 flags.
        self.handle_unsaturated_result(mac1, MAC1);
        self.handle_unsaturated_result(mac2, MAC2);
        self.handle_unsaturated_result(mac3, MAC3);

        // Perform first common stage of calculation.
        ir1 = ((rfc << 12) - mac1) >> (sf * 12);
        ir2 = ((gfc << 12) - mac2) >> (sf * 12);
        ir3 = ((bfc << 12) - mac3) >> (sf * 12);

        // Saturate and check flags for IR1, IR2 and IR3. Note that for this specific
        // check, we ignore the lm bit, and always use the negative lower bound.
        ir1 = self.handle_saturated_result(ir1, IR1, false, sf);
        ir2 = self.handle_saturated_result(ir2, IR2, false, sf);
        ir3 = self.handle_saturated_result(ir3, IR3, false, sf);

        // Continue first common stage of calculation.
        mac1 += ir1 * ir0;
        mac2 += ir2 * ir0;
        mac3 += ir3 * ir0;

        // Handle MAC1, MAC2 and MAC3 flags again.
        self.handle_unsaturated_result(mac1, MAC1);
        self.handle_unsaturated_result(mac2, MAC2);
        self.handle_unsaturated_result(mac3, MAC3);

        // Shift MAC1, MAC2 and MAC3 by (sf * 12) bits, preserving sign bit.
        mac1 >>= sf * 12;
        mac2 >>= sf * 12;
        mac3 >>= sf * 12;

        // Handle MAC1, MAC2 and MAC3 flags again.
        self.handle_unsaturated_result(mac1, MAC1);
        self.handle_unsaturated_result(mac2, MAC2);
        self.handle_unsaturated_result(mac3, MAC3);

        // Store MAC1, MAC2 and MAC3 to IR1, IR2 and IR3, handling saturation and flags.
        ir1 = self.handle_saturated_result(mac1, IR1, lm, sf);
        ir2 = self.handle_saturated_result(mac2, IR2, lm, sf);
        ir3 = self.handle_saturated_result(mac3, IR3, lm, sf);

        // Calculate colour FIFO entries, saturating and flag setting as needed.
        let r_out = self.handle_saturated_result(mac1 / 16, ColourFifoR, lm, sf);
        let g_out = self.handle_saturated_result(mac2 / 16, ColourFifoG, lm, sf);
        let b_out = self.handle_saturated_result(mac3 / 16, ColourFifoB, lm, sf);

        // Calculate flag bit 31.
        if (self.control_registers[31] & 0x7F87E000) != 0 {
            self.control_registers[31] |= 0x80000000_u32 as i32;
        }

        // Store all values back.
        self.data_registers[25] = mac1 as i32; // MAC1.
        self.data_registers[26] = mac2 as i32; // MAC2.
        self.data_registers[27] = mac3 as i32; // MAC3.

        self.data_registers[9] = ir1 as i32;  // IR1.
        self.data_registers[10] = ir2 as i32; // IR2.
        self.data_registers[11] = ir3 as i32; // IR3.

        self.data_registers[20] = self.data_registers[21]; // RGB1 to RGB0.
        self.data_registers[21] = self.data_registers[22]; // RGB2 to RGB1.
        self.data_registers[22] = ((code << 24) | (b_out << 16) | (g_out << 8) | r_out) as i32; // RGB2.
    }

    /// This function handles the MVMVA GTE function.
    fn handle_mvmva(&mut self, opcode: i32) {

        // Clear flag register.
        self.control_registers[31] = 0;

        // Filter out sf bit.
        let sf = opcode.bit_value(19);

        // Get lm bit status.
        let lm = opcode.bit_is_set(10);

        // Filter out translation vector value.
        let t_vec = (opcode & 0x6000).logical_rshift(13);

        // Filter out multiply vector value.
        let m_vec = (opcode & 0x18000).logical_rshift(15);

        // Filter out multiply matrix value.
        let m_matrix = (opcode & 0x60000).logical_rshift(17);

        // Declare and store correct translation vector values, allowing for
        // natural sign extension.
        let mut translation_vector = match t_vec {

            // TR.
            0 => CP2Vector::new(
                self.control_registers[5] as i64, // TRX.
                self.control_registers[6] as i64, // TRY.
                self.control_registers[7] as i64  // TRZ.
            ),

            // BK.
            1 => CP2Vector::new(
                self.control_registers[13] as i64, // RBK.
                self.control_registers[14] as i64, // GBK.
                self.control_registers[15] as i64  // BBK.
            ),

            // FC.
            2 => CP2Vector::new(
                self.control_registers[21] as i64, // RFC.
                self.control_registers[22] as i64, // GFC.
                self.control_registers[23] as i64  // BFC.
            ),

            // None, use empty vector. In C version we checked for 3 here, but just used break.
            // Effect here is the same therefore as 0 to 3 are the only possible values based on
            // our math above.
            _ => CP2Vector::new(
                0,
                0,
                0
            )
        };

        // Multiply all translation vector elements by 0x1000. Needed later on for our calculation.
        translation_vector = CP2Vector::new(
            translation_vector.top() * 0x1000,
            translation_vector.middle() * 0x1000,
            translation_vector.bottom() * 0x1000
        );

        // Declare and store correct multiply vector values, manually
        // sign extending as needed.
        let multiply_vector = match m_vec {

            // V0.
            0 => CP2Vector::new(
                ((self.data_registers[0] & 0xFFFF) as i64).sign_extend(15),                    // VX0.
                ((self.data_registers[0].logical_rshift(16) & 0xFFFF) as i64).sign_extend(15), // VY0.
                ((self.data_registers[1] & 0xFFFF) as i64).sign_extend(15)                     // VZ0.
            ),

            // V1.
            1 => CP2Vector::new(
                ((self.data_registers[2] & 0xFFFF) as i64).sign_extend(15),                    // VX1.
                ((self.data_registers[2].logical_rshift(16) & 0xFFFF) as i64).sign_extend(15), // VY1.
                ((self.data_registers[3] & 0xFFFF) as i64).sign_extend(15),                    // VZ1.
            ),

            // V2.
            2 => CP2Vector::new(
                ((self.data_registers[4] & 0xFFFF) as i64).sign_extend(15),                    // VX2.
                ((self.data_registers[4].logical_rshift(16) & 0xFFFF) as i64).sign_extend(15), // VY2.
                ((self.data_registers[5] & 0xFFFF) as i64).sign_extend(15),                    // VZ2.
            ),

            // [IR1, IR2, IR3]. If we get here, then m_vec must be 3 due to the math above.
            _ => CP2Vector::new(
                ((self.data_registers[9] & 0xFFFF) as i64).sign_extend(15),  // IR1.
                ((self.data_registers[10] & 0xFFFF) as i64).sign_extend(15), // IR2.
                ((self.data_registers[11] & 0xFFFF) as i64).sign_extend(15)  // IR3.
            ),
        };

        // Declare and store correct multiply matrix values, manually
        // sign extending as needed.
        let multiply_matrix = match m_matrix {

            // Rotation matrix.
            0 => CP2Matrix::new(

                // Top row.
                [
                    ((self.control_registers[0] & 0xFFFF) as i64).sign_extend(15),                    // RT11.
                    ((self.control_registers[0].logical_rshift(16) & 0xFFFF) as i64).sign_extend(15), // RT12.
                    ((self.control_registers[1] & 0xFFFF) as i64).sign_extend(15)                     // RT13.
                ],

                // Middle row.
                [
                    ((self.control_registers[1].logical_rshift(16) & 0xFFFF) as i64).sign_extend(15), // RT21.
                    ((self.control_registers[2] & 0xFFFF) as i64).sign_extend(15),                    // RT22.
                    ((self.control_registers[2].logical_rshift(16) & 0xFFFF) as i64).sign_extend(15)  // RT23.
                ],

                // Bottom row.
                [
                    ((self.control_registers[3] & 0xFFFF) as i64).sign_extend(15),                    // RT31.
                    ((self.control_registers[3].logical_rshift(16) & 0xFFFF) as i64).sign_extend(15), // RT32.
                    ((self.control_registers[4] & 0xFFFF) as i64).sign_extend(15)                     // RT33.
                ]
            ),

            // Light matrix.
            1 => CP2Matrix::new(

                // Top row.
                [
                    ((self.control_registers[8] & 0xFFFF) as i64).sign_extend(15),                    // L11.
                    ((self.control_registers[8].logical_rshift(16) & 0xFFFF) as i64).sign_extend(15), // L12.
                    ((self.control_registers[9] & 0xFFFF) as i64).sign_extend(15)                     // L13.
                ],

                // Middle row.
                [
                    ((self.control_registers[9].logical_rshift(16) & 0xFFFF) as i64).sign_extend(15), // L21.
                    ((self.control_registers[10] & 0xFFFF) as i64).sign_extend(15),                   // L22.
                    ((self.control_registers[10].logical_rshift(16) & 0xFFFF) as i64).sign_extend(15) // L23.
                ],

                // Bottom row.
                [
                    ((self.control_registers[11] & 0xFFFF) as i64).sign_extend(15),                    // L31.
                    ((self.control_registers[11].logical_rshift(16) & 0xFFFF) as i64).sign_extend(15), // L32.
                    ((self.control_registers[12] & 0xFFFF) as i64).sign_extend(15)                     // L33.
                ]
            ),

            // Colour matrix.
            2 => CP2Matrix::new(

                // Top row.
                [
                    ((self.control_registers[16] & 0xFFFF) as i64).sign_extend(15),                    // LR1.
                    ((self.control_registers[16].logical_rshift(16) & 0xFFFF) as i64).sign_extend(15), // LR2.
                    ((self.control_registers[17] & 0xFFFF) as i64).sign_extend(15)                     // LR3.
                ],

                // Middle row.
                [
                    ((self.control_registers[17].logical_rshift(16) & 0xFFFF) as i64).sign_extend(15), // LG1.
                    ((self.control_registers[18] & 0xFFFF) as i64).sign_extend(15),                    // LG2.
                    ((self.control_registers[18].logical_rshift(16) & 0xFFFF) as i64).sign_extend(15)  // LG3.
                ],

                // Bottom row.
                [
                    ((self.control_registers[19] & 0xFFFF) as i64).sign_extend(15),                    // LB1.
                    ((self.control_registers[19].logical_rshift(16) & 0xFFFF) as i64).sign_extend(15), // LB2.
                    ((self.control_registers[20] & 0xFFFF) as i64).sign_extend(15)                     // LB3.
                ]
            ),

            // Reserved (garbage matrix). If we get here, then m_matrix must be 3 due to the math above.
            _ => CP2Matrix::new(

                // Top row.
                [
                    -0x60,
                    0x60,
                    ((self.data_registers[8] & 0xFFFF) as i64).sign_extend(15) // IR0.
                ],

                // Middle row.
                [
                    ((self.control_registers[1] & 0xFFFF) as i64).sign_extend(15), // RT13.
                    ((self.control_registers[1] & 0xFFFF) as i64).sign_extend(15), // RT13.
                    ((self.control_registers[1] & 0xFFFF) as i64).sign_extend(15)  // RT13.
                ],

                // Bottom row.
                [
                    ((self.control_registers[2] & 0xFFFF) as i64).sign_extend(15), // RT22.
                    ((self.control_registers[2] & 0xFFFF) as i64).sign_extend(15), // RT22.
                    ((self.control_registers[2] & 0xFFFF) as i64).sign_extend(15)  // RT22.
                ]
            ),
        };

        // Perform calculation now.
        let mut result_vector = if t_vec != 2 {
            multiply_matrix * multiply_vector + translation_vector
        } else {
            // Account for faulty FC vector calculation on real hardware.
            CP2Vector::new(
                multiply_matrix.top_right() * multiply_vector.bottom(),
                multiply_matrix.middle_right() * multiply_vector.bottom(),
                multiply_matrix.bottom_right() * multiply_vector.bottom()
            )
        };

        // Shift results right by (sf * 12) bits, preserving sign bit.
        result_vector = CP2Vector::new(
            result_vector.top() >> (sf * 12),
            result_vector.middle() >> (sf * 12),
            result_vector.bottom() >> (sf * 12)
        );

        // Set MAC1, MAC2 and MAC3 registers, handling flags too.
        self.data_registers[25] = result_vector.top() as i32;    // MAC1.
        self.data_registers[26] = result_vector.middle() as i32; // MAC2.
        self.data_registers[27] = result_vector.bottom() as i32; // MAC3.

        self.handle_unsaturated_result(result_vector.top(), MAC1);
        self.handle_unsaturated_result(result_vector.middle(), MAC2);
        self.handle_unsaturated_result(result_vector.bottom(), MAC3);

        // Set IR1, IR2 and IR3 registers and saturation flag bits.
        self.data_registers[9] = self.handle_saturated_result(result_vector.top(), IR1, lm, sf) as i32;
        self.data_registers[10] = self.handle_saturated_result(result_vector.middle(), IR2, lm, sf) as i32;
        self.data_registers[11] = self.handle_saturated_result(result_vector.bottom(), IR3, lm, sf) as i32;

        // Calculate flag bit 31.
        if (self.control_registers[31] & 0x7F87E000) != 0 {
            self.control_registers[31] |= 0x80000000_u32 as i32;
        }
    }

    /// This function handles the NCDS GTE function.
    fn handle_ncds(&mut self, opcode: i32) {

    }

    /// This function handles the CDP GTE function.
    fn handle_cdp(&mut self, opcode: i32) {

        // Clear flag register.
        self.control_registers[31] = 0;

        // Filter out sf bit.
        let sf = opcode.bit_value(19);

        // Get lm bit status.
        let lm = opcode.bit_is_set(10);

        // Retrieve background colour values - let natural sign extension happen,
        // and store them inside a vector. Also, multiple each one by 0x1000.
        let background_colour_vector = CP2Vector::new(
            (self.control_registers[13] as i64) * 0x1000, // RBK.
            (self.control_registers[14] as i64) * 0x1000, // GBK.
            (self.control_registers[15] as i64) * 0x1000  // BBK.
        );

        // Retrieve far colour values - let natural sign extension happen.
        let far_colour_vector = CP2Vector::new(
            self.control_registers[21] as i64, // RFC.
            self.control_registers[22] as i64, // GFC.
            self.control_registers[23] as i64  // BFC.
        );

        // Fetch IR1, IR2 and IR3, sign extending if needed, and also storing
        // into a vector.
        let mut ir_vector = CP2Vector::new(
            ((self.data_registers[9] & 0xFFFF) as i64).sign_extend(15),  // IR1.
            ((self.data_registers[10] & 0xFFFF) as i64).sign_extend(15), // IR2.
            ((self.data_registers[11] & 0xFFFF) as i64).sign_extend(15)  // IR3.
        );

        // Fetch IR0 too, also sign extending if needed.
        let ir0 = ((self.data_registers[8] & 0xFFFF) as i64).sign_extend(15); // IR0.

        // Retrieve RGBC values.
        let r = (self.data_registers[6] & 0xFF) as i64; // R.
        let g = (self.data_registers[6].logical_rshift(8) & 0xFF) as i64; // G.
        let b = (self.data_registers[6].logical_rshift(16) & 0xFF) as i64; // B.
        let code = (self.data_registers[6].logical_rshift(24) & 0xFF) as i64; // CODE.

        // Retrieve light colour matrix values, sign extending as needed.
        let light_colour_matrix = CP2Matrix::new(

            // Top row:
            [
                ((self.control_registers[16] & 0xFFFF) as i64).sign_extend(15), // LR1.
                ((self.control_registers[16].logical_rshift(16) & 0xFFFF) as i64).sign_extend(15), // LR2.
                ((self.control_registers[17] & 0xFFFF) as i64).sign_extend(15) // LR3.
            ],

            // Middle row:
            [
                ((self.control_registers[17].logical_rshift(16) & 0xFFFF) as i64).sign_extend(15), // LG1.
                ((self.control_registers[18] & 0xFFFF) as i64).sign_extend(15), // LG2.
                ((self.control_registers[18].logical_rshift(16) & 0xFFFF) as i64).sign_extend(15) // LG3.
            ],

            // Bottom row:
            [
                ((self.control_registers[19] & 0xFFFF) as i64).sign_extend(15), // LB1.
                ((self.control_registers[19].logical_rshift(16) & 0xFFFF) as i64).sign_extend(15), // LB2.
                ((self.control_registers[20] & 0xFFFF) as i64).sign_extend(15) // LB3.
            ]
        );

        // Perform first stage of calculaton, and shift results right by (sf * 12),
        // preserving sign bit.
        let mut mac_results = light_colour_matrix * ir_vector + background_colour_vector;
        mac_results = CP2Vector::new(
            mac_results.top() >> (sf * 12),
            mac_results.middle() >> (sf * 12),
            mac_results.bottom() >> (sf * 12)
        );

        // Now set MAC1, MAC2 and MAC3 flags accordingly.
        self.handle_unsaturated_result(mac_results.top(), MAC1);
        self.handle_unsaturated_result(mac_results.middle(), MAC2);
        self.handle_unsaturated_result(mac_results.bottom(), MAC3);

        // Store results to IR1, IR2 and IR3.
        ir_vector = CP2Vector::new(
            self.handle_saturated_result(mac_results.top(), IR1, lm, sf),
            self.handle_saturated_result(mac_results.middle(), IR2, lm, sf),
            self.handle_saturated_result(mac_results.bottom(), IR3, lm, sf)
        );

        // Perform second stage of calculation.
        mac_results = CP2Vector::new(
            (r * ir_vector.top()) << 4,
            (g * ir_vector.middle()) << 4,
            (b * ir_vector.bottom()) << 4
        );

        // Now again set MAC1, MAC2 and MAC3 flags accordingly.
        self.handle_unsaturated_result(mac_results.top(), MAC1);
        self.handle_unsaturated_result(mac_results.middle(), MAC2);
        self.handle_unsaturated_result(mac_results.bottom(), MAC3);

        // Perform first part of CDP-only stage of calculation.
        ir_vector = CP2Vector::new(
            ((far_colour_vector.top() << 12) - mac_results.top()) >> (sf * 12),
            ((far_colour_vector.middle() << 12) - mac_results.middle()) >> (sf * 12),
            ((far_colour_vector.bottom() << 12) - mac_results.bottom()) >> (sf * 12)
        );

        // Saturate and set IR1/IR2/IR3 as appropriate. Note that for this specific
        // check, we ignore the lm bit, and always use the negative lower bound.
        ir_vector = CP2Vector::new(
            self.handle_saturated_result(ir_vector.top(), IR1, false, sf),
            self.handle_saturated_result(ir_vector.middle(), IR2, false, sf),
            self.handle_saturated_result(ir_vector.bottom(), IR3, false, sf)
        );

        // Peform second part of CDP-only stage of calculation.
        mac_results = CP2Vector::new(
            ir_vector.top() * ir0 + mac_results.top(),
            ir_vector.middle() * ir0 + mac_results.middle(),
            ir_vector.bottom() * ir0 + mac_results.bottom()
        );

        // Now again set MAC1, MAC2 and MAC3 flags accordingly.
        self.handle_unsaturated_result(mac_results.top(), MAC1);
        self.handle_unsaturated_result(mac_results.middle(), MAC2);
        self.handle_unsaturated_result(mac_results.bottom(), MAC3);

        // Shift MAC1, MAC2 and MAC3 right by (sf * 12) bits, preserving sign bit.
        mac_results = CP2Vector::new(
            mac_results.top() >> (sf * 12),
            mac_results.middle() >> (sf * 12),
            mac_results.bottom() >> (sf * 12)
        );

        // Now again set MAC1, MAC2 and MAC3 flags accordingly.
        self.handle_unsaturated_result(mac_results.top(), MAC1);
        self.handle_unsaturated_result(mac_results.middle(), MAC2);
        self.handle_unsaturated_result(mac_results.bottom(), MAC3);

        // Store results to IR1, IR2 and IR3 again.
        ir_vector = CP2Vector::new(
            self.handle_saturated_result(mac_results.top(), IR1, lm, sf),
            self.handle_saturated_result(mac_results.middle(), IR2, lm, sf),
            self.handle_saturated_result(mac_results.bottom(), IR3, lm, sf)
        );

        // Generate colour FIFO values, setting flags as needed.
        let r_out = self.handle_saturated_result(mac_results.top() / 16, ColourFifoR, lm, sf);
        let g_out = self.handle_saturated_result(mac_results.middle() / 16, ColourFifoG, lm, sf);
        let b_out = self.handle_saturated_result(mac_results.bottom() / 16, ColourFifoB, lm, sf);

        // Calculate flag bit 31.
        if (self.control_registers[31] & 0x7F87E000) != 0 {
            self.control_registers[31] |= 0x80000000_u32 as i32;
        }

        // Store values back to registers.
        self.data_registers[25] = mac_results.top() as i32;    // MAC1.
        self.data_registers[26] = mac_results.middle() as i32; // MAC2.
        self.data_registers[27] = mac_results.bottom() as i32; // MAC3.

        self.data_registers[9] = ir_vector.top() as i32;     // IR1.
        self.data_registers[10] = ir_vector.middle() as i32; // IR2.
        self.data_registers[11] = ir_vector.bottom() as i32; // IR3.

        self.data_registers[20] = self.data_registers[21]; // RGB1 to RGB0.
        self.data_registers[21] = self.data_registers[22]; // RGB2 to RGB1.
        self.data_registers[22] = ((code << 24) | (b_out << 16) | (g_out << 8) | r_out) as i32; // RGB2.
    }

    /// This function handles the NCDT GTE function.
    fn handle_ncdt(&mut self, opcode: i32) {

    }

    /// This function handles the NCCS GTE function.
    fn handle_nccs(&mut self, opcode: i32) {

    }

    /// This function handles the CC GTE function.
    fn handle_cc(&mut self, opcode: i32) {

        // Clear flag register.
        self.control_registers[31] = 0;

        // Filter out sf bit.
        let sf = opcode.bit_value(19);

        // Get lm bit status.
        let lm = opcode.bit_is_set(10);

        // Retrieve background colour values - let natural sign extension happen,
        // and store them inside a vector. Also, multiple each one by 0x1000.
        let background_colour_vector = CP2Vector::new(
            (self.control_registers[13] as i64) * 0x1000, // RBK.
            (self.control_registers[14] as i64) * 0x1000, // GBK.
            (self.control_registers[15] as i64) * 0x1000  // BBK.
        );

        // Fetch IR1, IR2 and IR3, sign extending if needed, and also storing
        // into a vector.
        let mut ir_vector = CP2Vector::new(
            ((self.data_registers[9] & 0xFFFF) as i64).sign_extend(15),  // IR1.
            ((self.data_registers[10] & 0xFFFF) as i64).sign_extend(15), // IR2.
            ((self.data_registers[11] & 0xFFFF) as i64).sign_extend(15)  // IR3.
        );

        // Retrieve RGBC values.
        let r = (self.data_registers[6] & 0xFF) as i64; // R.
        let g = (self.data_registers[6].logical_rshift(8) & 0xFF) as i64; // G.
        let b = (self.data_registers[6].logical_rshift(16) & 0xFF) as i64; // B.
        let code = (self.data_registers[6].logical_rshift(24) & 0xFF) as i64; // CODE.

        // Retrieve light colour matrix values, sign extending as needed.
        let light_colour_matrix = CP2Matrix::new(

            // Top row:
            [
                ((self.control_registers[16] & 0xFFFF) as i64).sign_extend(15), // LR1.
                ((self.control_registers[16].logical_rshift(16) & 0xFFFF) as i64).sign_extend(15), // LR2.
                ((self.control_registers[17] & 0xFFFF) as i64).sign_extend(15) // LR3.
            ],

            // Middle row:
            [
                ((self.control_registers[17].logical_rshift(16) & 0xFFFF) as i64).sign_extend(15), // LG1.
                ((self.control_registers[18] & 0xFFFF) as i64).sign_extend(15), // LG2.
                ((self.control_registers[18].logical_rshift(16) & 0xFFFF) as i64).sign_extend(15) // LG3.
            ],

            // Bottom row:
            [
                ((self.control_registers[19] & 0xFFFF) as i64).sign_extend(15), // LB1.
                ((self.control_registers[19].logical_rshift(16) & 0xFFFF) as i64).sign_extend(15), // LB2.
                ((self.control_registers[20] & 0xFFFF) as i64).sign_extend(15) // LB3.
            ]
        );

        // Perform first stage of calculaton, and shift results right by (sf * 12),
        // preserving sign bit.
        let mut mac_results = light_colour_matrix * ir_vector + background_colour_vector;
        mac_results = CP2Vector::new(
            mac_results.top() >> (sf * 12),
            mac_results.middle() >> (sf * 12),
            mac_results.bottom() >> (sf * 12)
        );

        // Now set MAC1, MAC2 and MAC3 flags accordingly.
        self.handle_unsaturated_result(mac_results.top(), MAC1);
        self.handle_unsaturated_result(mac_results.middle(), MAC2);
        self.handle_unsaturated_result(mac_results.bottom(), MAC3);

        // Store results to IR1, IR2 and IR3.
        ir_vector = CP2Vector::new(
            self.handle_saturated_result(mac_results.top(), IR1, lm, sf),
            self.handle_saturated_result(mac_results.middle(), IR2, lm, sf),
            self.handle_saturated_result(mac_results.bottom(), IR3, lm, sf)
        );

        // Perform second stage of calculation.
        mac_results = CP2Vector::new(
            (r * ir_vector.top()) << 4,
            (g * ir_vector.middle()) << 4,
            (b * ir_vector.bottom()) << 4
        );

        // Now again set MAC1, MAC2 and MAC3 flags accordingly.
        self.handle_unsaturated_result(mac_results.top(), MAC1);
        self.handle_unsaturated_result(mac_results.middle(), MAC2);
        self.handle_unsaturated_result(mac_results.bottom(), MAC3);

        // Shift MAC1, MAC2 and MAC3 right by (sf * 12) bits, preserving sign bit.
        mac_results = CP2Vector::new(
            mac_results.top() >> (sf * 12),
            mac_results.middle() >> (sf * 12),
            mac_results.bottom() >> (sf * 12)
        );

        // Now again set MAC1, MAC2 and MAC3 flags accordingly.
        self.handle_unsaturated_result(mac_results.top(), MAC1);
        self.handle_unsaturated_result(mac_results.middle(), MAC2);
        self.handle_unsaturated_result(mac_results.bottom(), MAC3);

        // Store results to IR1, IR2 and IR3 again.
        ir_vector = CP2Vector::new(
            self.handle_saturated_result(mac_results.top(), IR1, lm, sf),
            self.handle_saturated_result(mac_results.middle(), IR2, lm, sf),
            self.handle_saturated_result(mac_results.bottom(), IR3, lm, sf)
        );

        // Generate colour FIFO values, setting flags as needed.
        let r_out = self.handle_saturated_result(mac_results.top() / 16, ColourFifoR, lm, sf);
        let g_out = self.handle_saturated_result(mac_results.middle() / 16, ColourFifoG, lm, sf);
        let b_out = self.handle_saturated_result(mac_results.bottom() / 16, ColourFifoB, lm, sf);

        // Calculate flag bit 31.
        if (self.control_registers[31] & 0x7F87E000) != 0 {
            self.control_registers[31] |= 0x80000000_u32 as i32;
        }

        // Store values back to registers.
        self.data_registers[25] = mac_results.top() as i32;    // MAC1.
        self.data_registers[26] = mac_results.middle() as i32; // MAC2.
        self.data_registers[27] = mac_results.bottom() as i32; // MAC3.

        self.data_registers[9] = ir_vector.top() as i32;     // IR1.
        self.data_registers[10] = ir_vector.middle() as i32; // IR2.
        self.data_registers[11] = ir_vector.bottom() as i32; // IR3.

        self.data_registers[20] = self.data_registers[21]; // RGB1 to RGB0.
        self.data_registers[21] = self.data_registers[22]; // RGB2 to RGB1.
        self.data_registers[22] = ((code << 24) | (b_out << 16) | (g_out << 8) | r_out) as i32; // RGB2.
    }

    /// This function implements the functionality for the NCS and NCT instructions.
    /// Figured I'm porting/re-writing from C anyway and these are largely identical.
    fn handle_common_nc(&mut self, opcode: i32, variant: InstructionVariant) {

        // Filter out sf bit.
        let sf = opcode.bit_value(19);

        // Get lm bit status.
        let lm = opcode.bit_is_set(10);

        // Retrieve light matrix values and sign extend as needed.
        let light_matrix = CP2Matrix::new(

            // Top row.
            [
                ((self.control_registers[8] & 0xFFFF) as i64).sign_extend(15),                    // L11.
                ((self.control_registers[8].logical_rshift(16) & 0xFFFF) as i64).sign_extend(15), // L12.
                ((self.control_registers[9] & 0xFFFF) as i64).sign_extend(15)                     // L13.
            ],

            // Middle row.
            [
                ((self.control_registers[9].logical_rshift(16) & 0xFFFF) as i64).sign_extend(15), // L21.
                ((self.control_registers[10] & 0xFFFF) as i64).sign_extend(15),                   // L22.
                ((self.control_registers[10].logical_rshift(16) & 0xFFFF) as i64).sign_extend(15) // L23.
            ],

            // Bottom row.
            [
                ((self.control_registers[11] & 0xFFFF) as i64).sign_extend(15),                    // L31.
                ((self.control_registers[11].logical_rshift(16) & 0xFFFF) as i64).sign_extend(15), // L32.
                ((self.control_registers[12] & 0xFFFF) as i64).sign_extend(15)                     // L33.
            ]
        );

        // Retrieve light colour matrix values and sign extend as needed.
        let light_colour_matrix = CP2Matrix::new(

            // Top row.
            [
                ((self.control_registers[16] & 0xFFFF) as i64).sign_extend(15),                    // LR1.
                ((self.control_registers[16].logical_rshift(16) & 0xFFFF) as i64).sign_extend(15), // LR2.
                ((self.control_registers[17] & 0xFFFF) as i64).sign_extend(15)                     // LR3.
            ],

            // Middle row.
            [
                ((self.control_registers[17].logical_rshift(16) & 0xFFFF) as i64).sign_extend(15), // LG1.
                ((self.control_registers[18] & 0xFFFF) as i64).sign_extend(15),                    // LG2.
                ((self.control_registers[18].logical_rshift(16) & 0xFFFF) as i64).sign_extend(15)  // LG3.
            ],

            // Bottom row.
            [
                ((self.control_registers[19] & 0xFFFF) as i64).sign_extend(15),                    // LB1.
                ((self.control_registers[19].logical_rshift(16) & 0xFFFF) as i64).sign_extend(15), // LB2.
                ((self.control_registers[20] & 0xFFFF) as i64).sign_extend(15)                     // LB3.
            ]
        );

        // Retrieve background colour vector values and sign extend naturally.
        // Also multiply by 0x1000 for usage in the second stage calculation.
        let background_colour_vector = CP2Vector::new(
            (self.control_registers[13] as i64) * 0x1000, // RBK.
            (self.control_registers[14] as i64) * 0x1000, // GBK.
            (self.control_registers[15] as i64) * 0x1000  // BBK.
        );

        // Retrieve CODE value.
        let code = (self.data_registers[6].logical_rshift(24) & 0xFF) as i64; // CODE.

        // Now, we perform the remaining tasks based on the specified number of iterations.
        let iterations = match variant {
            InstructionVariant::Single => 1,
            InstructionVariant::Triple => 3,
        };
        for i in 0..iterations {

            // Clear flag register.
            self.control_registers[31] = 0;

            // Retrieve V0, V1 or V2 values depending on iteration.
            let vx_vector = CP2Vector::new(
                ((self.data_registers[i * 2] & 0xFFFF) as i64).sign_extend(15),                    // VX0/VX1/VX2.
                ((self.data_registers[i * 2].logical_rshift(16) & 0xFFFF) as i64).sign_extend(15), // VY0/VY1/VY2.
                ((self.data_registers[i * 2 + 1] & 0xFFFF) as i64).sign_extend(15)                   // VZ0/VZ1/VZ2.
            );

            // Perform first stage of calculation, then shift right by (sf * 12) bits,
            // preserving sign bit.
            let mut mac_results = light_matrix * vx_vector;
            mac_results = CP2Vector::new(
                mac_results.top() >> (sf * 12),
                mac_results.middle() >> (sf * 12),
                mac_results.bottom() >> (sf * 12)
            );

            // Handle flags for MAC1, MAC2 and MAC3.
            self.handle_unsaturated_result(mac_results.top(), MAC1);
            self.handle_unsaturated_result(mac_results.middle(), MAC2);
            self.handle_unsaturated_result(mac_results.bottom(), MAC3);

            // Setup IR1, IR2 and IR3, handling flags too.
            let mut ir_vector = CP2Vector::new(
                self.handle_saturated_result(mac_results.top(), IR1, lm, sf),
                self.handle_saturated_result(mac_results.middle(), IR2, lm, sf),
                self.handle_saturated_result(mac_results.bottom(), IR3, lm, sf)
            );

            // Perform second stage of calculation, then shift right by (sf * 12) bits,
            // preserving sign bit.
            mac_results = light_colour_matrix * ir_vector + background_colour_vector;
            mac_results = CP2Vector::new(
                mac_results.top() >> (sf * 12),
                mac_results.middle() >> (sf * 12),
                mac_results.bottom() >> (sf * 12)
            );

            // Handle flags for MAC1, MAC2 and MAC3 again.
            self.handle_unsaturated_result(mac_results.top(), MAC1);
            self.handle_unsaturated_result(mac_results.middle(), MAC2);
            self.handle_unsaturated_result(mac_results.bottom(), MAC3);

            // Deal with IR1, IR2 and IR3 again, handling flags too.
            ir_vector = CP2Vector::new(
                self.handle_saturated_result(mac_results.top(), IR1, lm, sf),
                self.handle_saturated_result(mac_results.middle(), IR2, lm, sf),
                self.handle_saturated_result(mac_results.bottom(), IR3, lm, sf)
            );

            // Calculate result to be stored to colour FIFO. Also handle saturation and flags.
            let r_out = self.handle_saturated_result(mac_results.top() / 16, ColourFifoR, lm, sf);
            let g_out = self.handle_saturated_result(mac_results.middle() / 16, ColourFifoG, lm, sf);
            let b_out = self.handle_saturated_result(mac_results.bottom() / 16, ColourFifoB, lm, sf);

            // Calculate flag bit 31.
            if (self.control_registers[31] & 0x7F87E000) != 0 {
                self.control_registers[31] |= 0x80000000_u32 as i32;
            }

            // Store all values back.
            self.data_registers[25] = mac_results.top() as i32;    // MAC1.
            self.data_registers[26] = mac_results.middle() as i32; // MAC2.
            self.data_registers[27] = mac_results.bottom() as i32; // MAC3.

            self.data_registers[9] = ir_vector.top() as i32;     // IR1.
            self.data_registers[10] = ir_vector.middle() as i32; // IR2.
            self.data_registers[11] = ir_vector.bottom() as i32; // IR3.

            self.data_registers[20] = self.data_registers[21]; // RGB1 to RGB0.
            self.data_registers[21] = self.data_registers[22]; // RGB2 to RGB1.
            self.data_registers[22] = ((code << 24) | (b_out << 16) | (g_out << 8) | r_out) as i32; // RGB2.
        }
    }

    /// This function handles the SQR GTE function.
    fn handle_sqr(&mut self, opcode: i32) {

        // Clear flag register.
        self.control_registers[31] = 0;

        // Filter out sf bit.
        let sf = opcode.bit_value(19);

        // Fetch IR1, IR2 and IR3, sign extending if needed.
        let mut ir1 = ((self.data_registers[9] & 0xFFFF) as i64).sign_extend(15); // IR1.
        let mut ir2 = ((self.data_registers[10] & 0xFFFF) as i64).sign_extend(15); // IR2.
        let mut ir3 = ((self.data_registers[11] & 0xFFFF) as i64).sign_extend(15); // IR3.

        // Perform calculations.
        ir1 *= ir1;
        ir2 *= ir2;
        ir3 *= ir3;

        // Shift if specified.
        ir1 = ir1.logical_rshift(12 * sf);
        ir2 = ir2.logical_rshift(12 * sf);
        ir3 = ir3.logical_rshift(12 * sf);

        // Set MAC1, MAC2 and MAC3 registers.
        self.data_registers[25] = ir1 as i32; // MAC1.
        self.data_registers[26] = ir2 as i32; // MAC2.
        self.data_registers[27] = ir3 as i32; // MAC3.

        // Set IR1, IR2 and IR3 registers.
        //
        // Also handle saturation. Although saturation function checks for lower
        // bound too, that shouldn't cause problems here - the result will always be
        // positive due to squaring or zero.
        //
        // The max result of two i16 values multiplied together (allowing for overflow)
        // is thus: -32,768 x -32,768 = 1,073,741,824.
        // Bigger than an i16, but easily representable with i64 as we're using here.
        self.data_registers[9] = self.handle_saturated_result(ir1, IR1, false, sf) as i32;
        self.data_registers[10] = self.handle_saturated_result(ir2, IR2, false, sf) as i32;
        self.data_registers[11] = self.handle_saturated_result(ir3, IR3, false, sf) as i32;

        // Calculate flag bit 31.
        if (self.control_registers[31] & 0x7F87E000) != 0 {
            self.control_registers[31] |= 0x80000000_u32 as i32;
        }
    }

    /// This function handles the DCPL GTE function.
    fn handle_dcpl(&mut self, opcode: i32) {

        // Clear flag register.
        self.control_registers[31] = 0;

        // Filter out sf bit.
        let sf = opcode.bit_value(19);

        // Get lm bit status.
        let lm = opcode.bit_is_set(10);

        // Retrieve R0, IR1, IR2 and IR3, sign extending as necessary.
        let ir0 = ((self.data_registers[8] & 0xFFFF) as i64).sign_extend(15);
        let mut ir1 = ((self.data_registers[9] & 0xFFFF) as i64).sign_extend(15);
        let mut ir2 = ((self.data_registers[10] & 0xFFFF) as i64).sign_extend(15);
        let mut ir3 = ((self.data_registers[11] & 0xFFFF) as i64).sign_extend(15);

        // Retrieve far colour values - let natural sign extension happen.
        let rfc = self.control_registers[21] as i64;
        let gfc = self.control_registers[22] as i64;
        let bfc = self.control_registers[23] as i64;

        // Retrieve RGBC values.
        let r = (self.data_registers[6] & 0xFF) as i64; // R.
        let g = (self.data_registers[6].logical_rshift(8) & 0xFF) as i64; // G.
        let b = (self.data_registers[6].logical_rshift(16) & 0xFF) as i64; // B.
        let code = (self.data_registers[6].logical_rshift(24) & 0xFF) as i64; // CODE.

        // Perform DCPL-only calculation.
        let mut mac1 = (r * ir1) << 4;
        let mut mac2 = (g * ir2) << 4;
        let mut mac3 = (b * ir3) << 4;

        // Handle MAC1, MAC2 and MAC3 flags.
        self.handle_unsaturated_result(mac1, MAC1);
        self.handle_unsaturated_result(mac2, MAC2);
        self.handle_unsaturated_result(mac3, MAC3);

        // Perform first common stage of calculation, saturating and flag setting as needed.
        // Ignore lm bit for this first set of writes.
        ir1 = self.handle_saturated_result(((rfc << 12) - mac1) >> (sf * 12), IR1, false, sf);
        ir2 = self.handle_saturated_result(((gfc << 12) - mac2) >> (sf * 12), IR2, false, sf);
        ir3 = self.handle_saturated_result(((bfc << 12) - mac3) >> (sf * 12), IR3, false, sf);

        // Continue first common stage of calculation.
        mac1 += ir1 * ir0;
        mac2 += ir2 * ir0;
        mac3 += ir3 * ir0;

        // Handle MAC1, MAC2 and MAC3 flags again.
        self.handle_unsaturated_result(mac1, MAC1);
        self.handle_unsaturated_result(mac2, MAC2);
        self.handle_unsaturated_result(mac3, MAC3);

        // Shift MAC1, MAC2 and MAC3 by (sf * 12) bits, preserving sign bit.
        mac1 >>= sf * 12;
        mac2 >>= sf * 12;
        mac3 >>= sf * 12;

        // Handle MAC1, MAC2 and MAC3 flags again.
        self.handle_unsaturated_result(mac1, MAC1);
        self.handle_unsaturated_result(mac2, MAC2);
        self.handle_unsaturated_result(mac3, MAC3);

        // Store MAC1, MAC2 and MAC3 to IR1, IR3 and IR3, handling saturation + flags.
        ir1 = self.handle_saturated_result(mac1, IR1, lm, sf);
        ir2 = self.handle_saturated_result(mac2, IR2, lm, sf);
        ir3 = self.handle_saturated_result(mac3, IR3, lm, sf);

        // Generate colour FIFO values and check/set flags as needed.
        let r_out = self.handle_saturated_result(mac1 / 16, ColourFifoR, lm, sf);
        let g_out = self.handle_saturated_result(mac2 / 16, ColourFifoG, lm ,sf);
        let b_out = self.handle_saturated_result(mac3 / 16, ColourFifoB, lm, sf);

        // Calculate flag bit 31.
        if (self.control_registers[31] & 0x7F87E000) != 0 {
            self.control_registers[31] |= 0x80000000_u32 as i32;
        }

        // Store values back to registers.
        self.data_registers[25] = mac1 as i32; // MAC1.
        self.data_registers[26] = mac2 as i32; // MAC2.
        self.data_registers[27] = mac3 as i32; // MAC3.

        self.data_registers[9] = ir1 as i32;  // IR1.
        self.data_registers[10] = ir2 as i32; // IR2.
        self.data_registers[11] = ir3 as i32; // IR3.

        self.data_registers[20] = self.data_registers[21]; // RGB1 to RGB0.
        self.data_registers[21] = self.data_registers[22]; // RGB2 to RGB1.
        self.data_registers[22] = ((code << 24) | (b_out << 16) | (g_out << 8) | r_out) as i32; // RGB2.
    }

    /// This function handles the AVSZ3 GTE function.
    fn handle_avsz3(&mut self, opcode: i32) {

        // Clear flag register.
        self.control_registers[31] = 0;

        // Retrieve ZSF3, SZ1, SZ2 and SZ3, also sign extending
        // ZSF3 if needed.
        let zsf3 = ((self.control_registers[29] & 0xFFFF) as i64).sign_extend(15); // ZSF3.
        let sz1 = (self.data_registers[17] & 0xFFFF) as i64; // SZ1.
        let sz2 = (self.data_registers[18] & 0xFFFF) as i64; // SZ2.
        let sz3 = (self.data_registers[19] & 0xFFFF) as i64; // SZ3.

        // Perform calculation.
        // Set flags where needed, and apply saturation to OTZ if needed.
        let mac0 = zsf3 * (sz1 + sz2 + sz3);
        self.handle_unsaturated_result(mac0, MAC0);
        let otz = self.handle_saturated_result(mac0 / 0x1000, SZ3, false, 0);

        // Calculate flag bit 31.
        if (self.control_registers[31] & 0x7F87E000) != 0 {
            self.control_registers[31] |= 0x80000000_u32 as i32;
        }

        // Store results back to registers.
        self.data_registers[24] = mac0 as i32;
        self.data_registers[7] = otz as i32;
    }

    /// This function handles the AVSZ4 GTE function.
    fn handle_avsz4(&mut self, opcode: i32) {

        // Clear flag register.
        self.control_registers[31] = 0;

        // Retrieve ZSF4, SZ0, SZ1, SZ2 and SZ3, also sign extending
        // ZSF4 if needed.
        let zsf4 = ((self.control_registers[30] & 0xFFFF) as i64).sign_extend(15); // ZSF4.
        let sz0 = (self.data_registers[16] & 0xFFFF) as i64; // SZ0.
        let sz1 = (self.data_registers[17] & 0xFFFF) as i64; // SZ1.
        let sz2 = (self.data_registers[18] & 0xFFFF) as i64; // SZ2.
        let sz3 = (self.data_registers[19] & 0xFFFF) as i64; // SZ3.

        // Perform calculation.
        // Set flags where needed, and apply saturation to OTZ if needed.
        let mac0 = zsf4 * (sz0 + sz1 + sz2 + sz3);
        self.handle_unsaturated_result(mac0, MAC0);
        let otz = self.handle_saturated_result(mac0 / 0x1000, SZ3, false, 0);

        // Calculate flag bit 31.
        if (self.control_registers[31] & 0x7F87E000) != 0 {
            self.control_registers[31] |= 0x80000000_u32 as i32;
        }

        // Store results back to registers.
        self.data_registers[24] = mac0 as i32;
        self.data_registers[7] = otz as i32;
    }

    /// This function handles the GPF GTE function.
    fn handle_gpf(&mut self, opcode: i32) {

        // Clear flag register.
        self.control_registers[31] = 0;

        // Filter out sf bit.
        let sf = opcode.bit_value(19);

        // Get lm bit status.
        let lm = opcode.bit_is_set(10);

        // Retrieve R0, IR1, IR2 and IR3, sign extending as necessary.
        let ir0 = ((self.data_registers[8] & 0xFFFF) as i64).sign_extend(15);
        let mut ir1 = ((self.data_registers[9] & 0xFFFF) as i64).sign_extend(15);
        let mut ir2 = ((self.data_registers[10] & 0xFFFF) as i64).sign_extend(15);
        let mut ir3 = ((self.data_registers[11] & 0xFFFF) as i64).sign_extend(15);

        // Perform calculations.
        let mac1 = (ir1 * ir0) >> (sf * 12);
        let mac2 = (ir2 * ir0) >> (sf * 12);
        let mac3 = (ir3 * ir0) >> (sf * 12);

        // Check/set flags for MAC1, MAC2 and MAC3.
        self.handle_unsaturated_result(mac1, MAC1);
        self.handle_unsaturated_result(mac2, MAC2);
        self.handle_unsaturated_result(mac3, MAC3);

        // Store MAC1, MAC2 and MAC3 to IR1, IR2 and IR3.
        ir1 = self.handle_saturated_result(mac1, IR1, lm, sf);
        ir2 = self.handle_saturated_result(mac2, IR2, lm, sf);
        ir3 = self.handle_saturated_result(mac3, IR3, lm, sf);

        // Fetch code value from RGBC.
        let code = self.data_registers[6].logical_rshift(24) as i64;

        // Calculate colour FIFO entries, saturating and flag setting as needed.
        let r_out = self.handle_saturated_result(mac1 / 16, ColourFifoR, lm, sf);
        let g_out = self.handle_saturated_result(mac2 / 16, ColourFifoG, lm, sf);
        let b_out = self.handle_saturated_result(mac3 / 16, ColourFifoB, lm, sf);

        // Calculate flag bit 31.
        if (self.control_registers[31] & 0x7F87E000) != 0 {
            self.control_registers[31] |= 0x80000000_u32 as i32;
        }

        // Store all values back.
        self.data_registers[25] = mac1 as i32; // MAC1.
        self.data_registers[26] = mac2 as i32; // MAC2.
        self.data_registers[27] = mac3 as i32; // MAC3.

        self.data_registers[9] = ir1 as i32;  // IR1.
        self.data_registers[10] = ir2 as i32; // IR2.
        self.data_registers[11] = ir3 as i32; // IR3.

        self.data_registers[20] = self.data_registers[21]; // RGB1 to RGB0.
        self.data_registers[21] = self.data_registers[22]; // RGB2 to RGB1.
        self.data_registers[22] = ((code << 24) | (b_out << 16) | (g_out << 8) | r_out) as i32; // RGB2.
    }

    /// This function handles the GPL GTE function.
    fn handle_gpl(&mut self, opcode: i32) {

        // Clear flag register.
        self.control_registers[31] = 0;

        // Filter out sf bit.
        let sf = opcode.bit_value(19);

        // Get lm bit status.
        let lm = opcode.bit_is_set(10);

        // Retrieve MAC1, MAC2 and MAC3, naturally sign extending and shifting left by sf * 12.
        let mut mac1 = (self.data_registers[25] as i64) << (sf * 12);
        let mut mac2 = (self.data_registers[26] as i64) << (sf * 12);
        let mut mac3 = (self.data_registers[27] as i64) << (sf * 12);

        // Handle MAC1, MAC2 and MAC3 flags.
        self.handle_unsaturated_result(mac1, MAC1);
        self.handle_unsaturated_result(mac2, MAC2);
        self.handle_unsaturated_result(mac3, MAC3);

        // Retrieve R0, IR1, IR2 and IR3, sign extending as necessary.
        let ir0 = ((self.data_registers[8] & 0xFFFF) as i64).sign_extend(15);
        let mut ir1 = ((self.data_registers[9] & 0xFFFF) as i64).sign_extend(15);
        let mut ir2 = ((self.data_registers[10] & 0xFFFF) as i64).sign_extend(15);
        let mut ir3 = ((self.data_registers[11] & 0xFFFF) as i64).sign_extend(15);

        // Perform calculations.
        mac1 = ((ir1 * ir0) + mac1) >> (sf * 12);
        mac2 = ((ir2 * ir0) + mac2) >> (sf * 12);
        mac3 = ((ir3 * ir0) + mac3) >> (sf * 12);

        // Check/set flags for MAC1, MAC2 and MAC3 again.
        self.handle_unsaturated_result(mac1, MAC1);
        self.handle_unsaturated_result(mac2, MAC2);
        self.handle_unsaturated_result(mac3, MAC3);

        // Store MAC1, MAC2 and MAC3 to IR1, IR2 and IR3.
        ir1 = self.handle_saturated_result(mac1, IR1, lm, sf);
        ir2 = self.handle_saturated_result(mac2, IR2, lm, sf);
        ir3 = self.handle_saturated_result(mac3, IR3, lm, sf);

        // Fetch code value from RGBC.
        let code = self.data_registers[6].logical_rshift(24) as i64;

        // Calculate colour FIFO entries, saturating and flag setting as needed.
        let r_out = self.handle_saturated_result(mac1 / 16, ColourFifoR, lm, sf);
        let g_out = self.handle_saturated_result(mac2 / 16, ColourFifoG, lm, sf);
        let b_out = self.handle_saturated_result(mac3 / 16, ColourFifoB, lm, sf);

        // Calculate flag bit 31.
        if (self.control_registers[31] & 0x7F87E000) != 0 {
            self.control_registers[31] |= 0x80000000_u32 as i32;
        }

        // Store all values back.
        self.data_registers[25] = mac1 as i32; // MAC1.
        self.data_registers[26] = mac2 as i32; // MAC2.
        self.data_registers[27] = mac3 as i32; // MAC3.

        self.data_registers[9] = ir1 as i32;  // IR1.
        self.data_registers[10] = ir2 as i32; // IR2.
        self.data_registers[11] = ir3 as i32; // IR3.

        self.data_registers[20] = self.data_registers[21]; // RGB1 to RGB0.
        self.data_registers[21] = self.data_registers[22]; // RGB2 to RGB1.
        self.data_registers[22] = ((code << 24) | (b_out << 16) | (g_out << 8) | r_out) as i32; // RGB2.
    }

    /// This function handles the NCCT GTE function.
    fn handle_ncct(&mut self, opcode: i32) {

    }

    /// This function handles overflow/underflow detection for given results in MAC0/1/2/3, which we
    /// do not saturate.
    #[inline(always)]
    fn handle_unsaturated_result(&mut self, result: i64, result_type: UnsaturatedFlagRegisterField) {

        let (lower_bound, upper_bound) = match result_type {

            // Result larger than 43 bits and negative, or larger than 43 bits and positive.
            // This corrects a misconception for the larger bound that I had in the original.
            MAC1 | MAC2 | MAC3 => (-0x80000000000_i64, 0x7FFFFFFFFFF_i64),

            // Result larger than 31 bits and negative, or larger than 31 bits and positive.
            // Again, this correct a misconception for the larger bound in the original version.
            MAC0 => (-0x80000000_i64, 0x7FFFFFFF_i64),
        };

        let (lower_bit_flag, upper_bit_flag) = match result_type {

            // For lower and upper bit flags, these will respectively be:

            // Bit 27, Bit 30.
            MAC1 => (0x8000000_i32, 0x40000000_i32),

            // Bit 26, Bit 29.
            MAC2 => (0x4000000_i32, 0x20000000_i32),

            // Bit 25, Bit 28.
            MAC3 => (0x2000000_i32, 0x10000000_i32),

            // Bit 15, Bit 16.
            MAC0 => (0x8000_i32, 0x10000_i32),
        };

        // Now we can set flag register flags as appropriate.
        if result < lower_bound {
            self.control_registers[31] |= lower_bit_flag;
        }
        else if result > upper_bound {
            self.control_registers[31] |= upper_bit_flag;
        }
    }

    /// This function handles overflow/underflow detection for given results in:
    /// IR0/IR1/IR2/IR3/Colour-FIFO-R/Colour-FIFO-G/Colour-FIFO-B/SX2/SY2/SZ3.
    /// We also return a value which is conditionally saturated.
    #[inline(always)]
    fn handle_saturated_result(&mut self, result: i64, result_type: SaturatedFlagRegisterField, lm: bool, sf: i32) -> i64 {

        let (lower_bound, upper_bound) = match result_type {

            IR0 => (0_i64, 0x1000_i64),

            // IR1/IR2/IR3 need different behaviour depending on lm bit.
            IR1 | IR2 | IR3 => (
                if lm { 0_i64 } else { -0x8000_i64 }, 0x7FFF_i64
            ),

            // Special case for IR3 quirk in RTPS/RTPT.
            IR3Quirk => (-0x8000_i64, 0x7FFF_i64),

            ColourFifoR | ColourFifoG | ColourFifoB => (0_i64, 0xFF_i64),

            SX2 | SY2 => (-0x400_i64, 0x3FF_i64),

            SZ3 => (0_i64, 0xFFFF_i64),
        };

        let bit_flag = match result_type {

            IR0 => 0x1000_i32,

            IR1 => 0x1000000_i32,

            IR2 => 0x800000_i32,

            IR3 | IR3Quirk => 0x400000_i32,

            ColourFifoR => 0x200000_i32,

            ColourFifoG => 0x100000_i32,

            ColourFifoB => 0x80000_i32,

            SX2 => 0x4000_i32,

            SY2 => 0x2000_i32,

            SZ3 => 0x40000_i32,
        };

        // Now we can set flag register flags as appropriate,
        // and return the value, saturated or otherwise.
        // Be sure to peform the IR3 quirk behaviour if specified.
        if result < lower_bound {

            match result_type {

                IR3Quirk => {
                    // Deal with quirk in IR3 flag handling.
                    if sf == 0 {
                        // Shift result (a 64-bit signed value) right by 12 bits,
                        // preserving sign automatically.
                        let temp = result >> 12;
                        if !(-0x8000..0x7FFF).contains(&temp) {
                            self.control_registers[31] |= 0x400000;
                        }
                    } else {
                        self.control_registers[31] |= 0x400000;
                    }
                },

                _ => {
                    self.control_registers[31] |= bit_flag;
                },
            };

            lower_bound
        }
        else if result > upper_bound {

            match result_type {

                IR3Quirk => {
                    // Deal with quirk in IR3 flag handling.
                    if sf == 0 {
                        // Shift result (a 64-bit signed value) right by 12 bits,
                        // preserving sign automatically.
                        let temp = result >> 12;
                        if !(-0x8000..0x7FFF).contains(&temp) {
                            self.control_registers[31] |= 0x400000;
                        }
                    } else {
                        self.control_registers[31] |= 0x400000;
                    }
                },

                _ => {
                    self.control_registers[31] |= bit_flag;
                },
            };

            upper_bound
        }
        else {
            result
        }
    }
}

#[cfg(test)]
mod tests;
mod math;
