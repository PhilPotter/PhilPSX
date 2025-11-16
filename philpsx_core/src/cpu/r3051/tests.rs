// SPDX-License-Identifier: GPL-3.0
// tests.rs - Copyright Phillip Potter, 2025, under GPLv3 only.

use crate::cpu::r3051::mips_exception::MIPSExceptionReason;

use super::R3051;

// Tests for the R3051 CPU core.

#[test]
fn test_add_instruction_success() {

    let mut r3051 = R3051::new();

    // Given registers 1 and 2 contain 4 and 5,
    r3051.general_registers[1] = 4;
    r3051.general_registers[2] = 5;

    // Adding them together should produce no exception and 9.
    // Instruction should be passed in big-endian form.
    let instruction = 0x00221820;

    r3051.add_instruction(instruction);

    // Check exception and register 3 for result.
    assert_eq!(r3051.general_registers[3], 9);
    assert_eq!(r3051.exception.exception_reason, MIPSExceptionReason::NULL);
}

#[test]
fn test_add_instruction_overflow() {

    let mut r3051 = R3051::new();

    // Given registers 1 and 2 contain 2,147,483,647 and 1,
    r3051.general_registers[1] = 2_147_483_647;
    r3051.general_registers[2] = 1;

    // Adding them together should produce an exception.
    // Instruction should be passed in big-endian form.
    let instruction = 0x00221820;
    r3051.add_instruction(instruction);

    // Check exception.
    assert_eq!(r3051.exception.exception_reason, MIPSExceptionReason::OVF);
}

#[test]
fn test_addi_instruction_success() {

    let mut r3051 = R3051::new();

    // Given register 1 contains 4.
    r3051.general_registers[1] = 4;

    // Adding immediate value of 5 should produce no exception and 9.
    // Instruction should be passed in big-endian form.
    let instruction = 0x20220005;
    r3051.addi_instruction(instruction);

    // Check exception and register 2 for result.
    assert_eq!(r3051.general_registers[2], 9);
    assert_eq!(r3051.exception.exception_reason, MIPSExceptionReason::NULL);
}

#[test]
fn test_addi_instruction_overflow() {

    let mut r3051 = R3051::new();

    // Given register 1 contains -2,147,483,648.
    r3051.general_registers[1] = -2_147_483_648;

    // Adding immediate value of -1 should produce an exception.
    // Instruction should be passed in big-endian form.
    let instruction = 0x2022FFFF;
    r3051.addi_instruction(instruction);

    // Check exception.
    assert_eq!(r3051.exception.exception_reason, MIPSExceptionReason::OVF);
}

#[test]
fn test_addiu_instruction_success() {

    let mut r3051 = R3051::new();

    // Given register 1 contains 4,294,967,295.
    r3051.general_registers[1] = 4_294_967_295_u32 as i32;

    // Adding immediate value of 1 should cause wrap around to 0.
    // Instruction should be passed in big-endian form.
    let instruction = 0x24220001;
    r3051.addiu_instruction(instruction);

    // Check exception and register 2 for result.
    assert_eq!(r3051.general_registers[2], 0);
    assert_eq!(r3051.exception.exception_reason, MIPSExceptionReason::NULL);
}

#[test]
fn test_addu_instruction_success() {

    let mut r3051 = R3051::new();

    // Given registers 1 and 2 contain 4,294,967,295 and 1.
    r3051.general_registers[1] = 4_294_967_295_u32 as i32;
    r3051.general_registers[2] = 1;

    // Adding them together should cause wrap around to 0.
    // Instruction should be passed in big-endian form.
    let instruction = 0x00221821;
    r3051.addiu_instruction(instruction);

    // Check exception and register 3 for result.
    assert_eq!(r3051.general_registers[3], 0);
    assert_eq!(r3051.exception.exception_reason, MIPSExceptionReason::NULL);
}

#[test]
fn test_and_instruction_success() {

    let mut r3051 = R3051::new();

    // Given registers 1 and 2 contain 0xFFFFFFFF and 0xFFFF0000.
    r3051.general_registers[1] = 0xFFFFFFFF_u32 as i32;
    r3051.general_registers[2] = 0xFFFF0000_u32 as i32;

    // ANDing them together should produce 0xFFFF0000.
    let instruction = 0x00221824;
    r3051.and_instruction(instruction);

    // Check register 3 for result.
    assert_eq!(r3051.general_registers[3], 0xFFFF0000_u32 as i32);
}

#[test]
fn test_andi_instruction_success() {

    let mut r3051 = R3051::new();

    // Given register 1 contains 0xFFFFFFFF.
    r3051.general_registers[1] = 0xFFFFFFFF_u32 as i32;

    // ANDing with immediate of 0xFFFF should produce 0xFFFF.
    let instruction = 0x3022FFFF;
    r3051.andi_instruction(instruction);

    // Check register 2 for result.
    assert_eq!(r3051.general_registers[2], 0xFFFF);
}