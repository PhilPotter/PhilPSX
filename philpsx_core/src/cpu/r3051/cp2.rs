// SPDX-License-Identifier: GPL-3.0
// cp2.rs - Copyright Phillip Potter, 2025, under GPLv3 only.

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
    fn reset(&mut self) {
        self.condition_line = false;
    }
}