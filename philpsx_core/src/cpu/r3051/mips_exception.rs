// SPDX-License-Identifier: GPL-3.0
// mips_exception.rs - Copyright Phillip Potter, 2025, under GPLv3 only.

/// This structure models exceptions within the R3051 CPU.
pub struct MIPSException {
    pub exception_reason: MIPSExceptionReason,
    pub program_counter_origin: i32,
    pub bad_address: i32,
    pub co_processor_num: i32,
    pub is_in_branch_delay_slot: bool
}

/// This enum represents all possible reasons for an exception.
/// Integer codes are listed explicitly for clarity.
#[derive(Copy, Clone, PartialEq)]
#[repr(i32)]
pub enum MIPSExceptionReason {
    INT = 0,
    ADEL = 4,
    ADES = 5,
    IBE = 6,
    DBE = 7,
    SYS = 8,
    BP = 9,
    RI = 10,
    CPU = 11,
    OVF = 12,
    RESET = 13,
    NULL = 14,
}

impl MIPSException {

    /// Creates a new MIPSException object with the correct initial state.
    pub fn new() -> Self {
        MIPSException {
            exception_reason: MIPSExceptionReason::NULL,
            program_counter_origin: 0,
            bad_address: 0,
            co_processor_num: 0,
            is_in_branch_delay_slot: false,
        }
    }

    /// Resets a MIPSException object to its initial empty state.
    pub fn reset(&mut self) {
        self.exception_reason = MIPSExceptionReason::NULL;
        self.program_counter_origin = 0;
        self.bad_address = 0;
        self.co_processor_num = 0;
        self.is_in_branch_delay_slot = false;
    }
}