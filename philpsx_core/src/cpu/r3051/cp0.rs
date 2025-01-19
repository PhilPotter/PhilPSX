// SPDX-License-Identifier: GPL-3.0
// cp0.rs - Copyright Phillip Potter, 2025, under GPLv3 only.

/// The CP0 structure models the System Control Co-Processor (CP0), which
/// is responsible for mememory management and exceptions.
pub struct CP0 {

    // Register definitions.
    cop_registers: [i32; 32],

    // Condition line.
    condition_line: bool,
}

impl CP0 {

    /// Creates a new CP0 object with the correct initial state.
    pub fn new() -> Self {

        let mut cp0 = CP0 {
            
            // Zero out all registers.
            cop_registers: [0; 32],

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
        self.cop_registers[1] = 63 << 8;

        // Set BEV and TS bits of status register to 0 and 0 (BEV should be 1 but
        // PSX doesn't run this way, TS should be 1 but other emulators don't seem
        // to do this).
        self.cop_registers[12] &= 0xFF9FFFFFu32 as i32;

        // Set SWc, KUc and IEc bits of status register to 0.
        self.cop_registers[12] &= 0xFFFDFFFCu32 as i32;

        // Set condition line to false.
        self.condition_line = false;
    }

    /// This function returns the reset exception vector's virtual address.
    pub fn get_reset_exception_vector(&self) -> i32 {
        0xBFC00000u32 as i32
    }
}