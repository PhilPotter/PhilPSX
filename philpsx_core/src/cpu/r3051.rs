// SPDX-License-Identifier: GPL-3.0
// r3051.rs - Copyright Phillip Potter, 2025, under GPLv3 only.

use super::{Cpu, CpuBridge};
use philpsx_utility::SystemBusHolder;
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
        };

        r3051.reset();

        r3051
    }

    /// Set the R3051 object to its correct initial state.
    fn reset(&mut self) {

        // Patch in later with proper reset exception vector.
        self.program_counter = self.sccp.get_reset_exception_vector();
    }
}

/// Implementation functions to be called from anything that understands what
/// a Cpu object is.
impl Cpu for R3051 {

    /// Set the system bus holder.
    fn set_system_bus_holder(
        &mut self,
        holder: SystemBusHolder,
        _bridge: &mut dyn CpuBridge
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