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

#[test]
fn test_bc2f_condition_line_false() {

    let mut r3051 = R3051::new();

    // Given a false condition line (default) and immediate of
    // -4 (when left shifted 2 bits and sign extended), jump
    // address should then be equal to program counter after
    // execution of BC2F, and jump should be pending.
    let instruction = 0x4900FFFF;
    r3051.bc2f_instruction(instruction);

    assert_eq!(r3051.jump_address, r3051.program_counter);
    assert!(r3051.jump_pending);
    assert!(!r3051.is_branch);
}

#[test]
fn test_bc2f_condition_line_true() {

    let mut r3051 = R3051::new();

    // Given a false condition line (default) and immediate of
    // -4 (when left shifted 2 bits and sign extended), jump
    // address should be unset and jump not pending.
    r3051.gte.set_condition_line_status(true);
    let instruction = 0x4900FFFF;
    r3051.bc2f_instruction(instruction);

    assert_eq!(r3051.jump_address, 0);
    assert!(!r3051.jump_pending);
    assert!(!r3051.is_branch);
}

#[test]
fn test_bc2t_condition_line_false() {

    let mut r3051 = R3051::new();

    // Given a false condition line (default) and immediate of
    // -4 (when left shifted 2 bits and sign extended), jump
    // address should be unset and jump not pending.
    let instruction = 0x4901FFFF;
    r3051.bc2t_instruction(instruction);

    assert_eq!(r3051.jump_address, 0);
    assert!(!r3051.jump_pending);
    assert!(!r3051.is_branch);
}

#[test]
fn test_bc2t_condition_line_true() {

    let mut r3051 = R3051::new();

    // Given a false condition line (default) and immediate of
    // -4 (when left shifted 2 bits and sign extended), jump
    // address should then be equal to program counter after
    // execution of BC2F, and jump should be pending.
    r3051.gte.set_condition_line_status(true);
    let instruction = 0x4901FFFF;
    r3051.bc2t_instruction(instruction);

    assert_eq!(r3051.jump_address, r3051.program_counter);
    assert!(r3051.jump_pending);
    assert!(!r3051.is_branch);
}

#[test]
fn test_beq_registers_not_equal() {

    let mut r3051 = R3051::new();

    // Given unequal registers and an immediate of -4
    // (when left shifted 2 bits and sign extended), jump
    // address should be unset and jump not pending.
    r3051.general_registers[1] = 1;
    let instruction = 0x1022FFFF;
    r3051.beq_instruction(instruction);

    assert_eq!(r3051.jump_address, 0);
    assert!(!r3051.jump_pending);
    assert!(r3051.is_branch);
}

#[test]
fn test_beq_registers_equal() {

    let mut r3051 = R3051::new();

    // Given equal registers (default) and an immediate of
    // -4 (when left shifted 2 bits and sign extended), jump
    // address should then be equal to program counter after
    // execution of BC2F, and jump should be pending.
    let instruction = 0x1022FFFF;
    r3051.beq_instruction(instruction);

    assert_eq!(r3051.jump_address, r3051.program_counter);
    assert!(r3051.jump_pending);
    assert!(r3051.is_branch);
}

#[test]
fn test_bgez_register_greater_than_or_equal_to_zero() {

    let mut r3051 = R3051::new();

    // Given register greater than or equal to 0 (register
    // 1, default) and an immediate of -4 (when left shifted 2 bits
    // and sign extended), jump address should be unset
    // and jump not pending.
    let instruction = 0x0421FFFF;
    r3051.bgez_instruction(instruction);

    assert_eq!(r3051.jump_address, r3051.program_counter);
    assert!(r3051.jump_pending);
    assert!(r3051.is_branch);
}

#[test]
fn test_bgez_register_less_than_zero() {

    let mut r3051 = R3051::new();

    // Given register less than 0 (register 1, default) and
    // an immediate of -4 (when left shifted 2 bits and sign
    // extended), jump address should be unset and jump not
    // pending.
    r3051.general_registers[1] = -1;
    let instruction = 0x0421FFFF;
    r3051.bgez_instruction(instruction);

    assert_eq!(r3051.jump_address, 0);
    assert!(!r3051.jump_pending);
    assert!(r3051.is_branch);
}

#[test]
fn test_bgezal_register_greater_than_or_equal_to_zero() {

    let mut r3051 = R3051::new();

    // Given register greater than or equal to 0 (register
    // 1, default) and an immediate of -4 (when left shifted 2 bits
    // and sign extended), jump address should be unset
    // and jump not pending. Return address should be program
    // counter + 8.
    let instruction = 0x0431FFFF;
    r3051.bgezal_instruction(instruction);

    assert_eq!(r3051.general_registers[31], r3051.program_counter + 8);
    assert_eq!(r3051.jump_address, r3051.program_counter);
    assert!(r3051.jump_pending);
    assert!(r3051.is_branch);
}

#[test]
fn test_bgezal_register_less_than_zero() {

    let mut r3051 = R3051::new();

    // Given register less than 0 (register 1) and an
    // immediate of -4 (when left shifted 2 bits and sign
    // extended), jump address should be unset and jump not
    // pending. Return address should be zero.
    r3051.general_registers[1] = -1;
    let instruction = 0x0431FFFF;
    r3051.bgezal_instruction(instruction);

    assert_eq!(r3051.jump_address, 0);
    assert_eq!(r3051.general_registers[31], 0);
    assert!(!r3051.jump_pending);
    assert!(r3051.is_branch);
}

#[test]
fn test_bgtz_register_greater_than_zero() {

    let mut r3051 = R3051::new();

    // Given register greater than 0 (register 1) and an
    // immediate of -4 (when left shifted 2 bits and sign
    // extended), jump address should be set and jump
    // pending.
    r3051.general_registers[1] = 1;
    let instruction = 0x1C20FFFF;
    r3051.bgtz_instruction(instruction);

    assert_eq!(r3051.jump_address, r3051.program_counter);
    assert!(r3051.jump_pending);
    assert!(r3051.is_branch);
}

#[test]
fn test_bgtz_register_less_than_or_equal_to_zero() {

    let mut r3051 = R3051::new();

    // Given register less than or equal to 0 (register 1, default)
    // and an immediate of -4 (when left shifted 2 bits and sign
    // extended), jump address should be equal to PC and jump
    // pending.
    let instruction = 0x1C20FFFF;
    r3051.bgtz_instruction(instruction);

    assert_ne!(r3051.jump_address, r3051.program_counter);
    assert!(!r3051.jump_pending);
    assert!(r3051.is_branch);
}

#[test]
fn test_blez_register_greater_than_zero() {

    let mut r3051 = R3051::new();

    // Given register greater than 0 (register 1) and an
    // immediate of -4 (when left shifted 2 bits and sign
    // extended), jump address should be unset and jump not
    // pending.
    r3051.general_registers[1] = 1;
    let instruction = 0x1820FFFF;
    r3051.blez_instruction(instruction);

    assert_ne!(r3051.jump_address, r3051.program_counter);
    assert!(!r3051.jump_pending);
    assert!(r3051.is_branch);
}

#[test]
fn test_blez_register_less_than_or_equal_to_zero() {

    let mut r3051 = R3051::new();

    // Given register less than or equal to 0 (register 1, default)
    // and an immediate of -4 (when left shifted 2 bits and sign
    // extended), jump address should be equal to PC and jump
    // pending.
    let instruction = 0x1820FFFF;
    r3051.blez_instruction(instruction);

    assert_eq!(r3051.jump_address, r3051.program_counter);
    assert!(r3051.jump_pending);
    assert!(r3051.is_branch);
}

#[test]
fn test_bltz_register_greater_than_or_equal_to_zero() {

    let mut r3051 = R3051::new();

    // Given register greater than or equal 0 (register 1, default)
    // and an immediate of -4 (when left shifted 2 bits and sign
    // extended), jump address should be unset and jump not
    // pending.
    let instruction = 0x0420FFFF;
    r3051.bltz_instruction(instruction);

    assert_ne!(r3051.jump_address, r3051.program_counter);
    assert!(!r3051.jump_pending);
    assert!(r3051.is_branch);
}

#[test]
fn test_bltz_register_less_than_zero() {

    let mut r3051 = R3051::new();

    // Given register less than 0 (register 1) and an
    // immediate of -4 (when left shifted 2 bits and sign
    // extended), jump address should be equal to PC and jump
    // pending.
    r3051.general_registers[1] = -1;
    let instruction = 0x0420FFFF;
    r3051.bltz_instruction(instruction);

    assert_eq!(r3051.jump_address, r3051.program_counter);
    assert!(r3051.jump_pending);
    assert!(r3051.is_branch);
}

#[test]
fn test_bltzal_register_greater_than_or_equal_to_zero() {

    let mut r3051 = R3051::new();

    // Given register greater than or equal 0 (register 1, default)
    // and an immediate of -4 (when left shifted 2 bits and sign
    // extended), jump address should be unset and jump not
    // pending. Return address should be zero.
    let instruction = 0x0430FFFF;
    r3051.bltzal_instruction(instruction);

    assert_ne!(r3051.jump_address, r3051.program_counter);
    assert_eq!(r3051.general_registers[31], 0);
    assert!(!r3051.jump_pending);
    assert!(r3051.is_branch);
}

#[test]
fn test_bltzal_register_less_than_zero() {

    let mut r3051 = R3051::new();

    // Given register less than 0 (register 1) and an
    // immediate of -4 (when left shifted 2 bits and sign
    // extended), jump address should be equal to PC and jump
    // pending. Return address should be PC + 8.
    r3051.general_registers[1] = -1;
    let instruction = 0x0430FFFF;
    r3051.bltzal_instruction(instruction);

    assert_eq!(r3051.jump_address, r3051.program_counter);
    assert_eq!(r3051.general_registers[31], r3051.program_counter + 8);
    assert!(r3051.jump_pending);
    assert!(r3051.is_branch);
}

#[test]
fn test_bne_registers_equal() {

    let mut r3051 = R3051::new();

    // Given registers equal (registers 1 and 2, default) and
    // an immediate of -4 (when left shifted 2 bits and sign
    // extended), jump address should be unset and jump not
    // pending.
    let instruction = 0x1422FFFF;
    r3051.bne_instruction(instruction);

    assert_eq!(r3051.jump_address, 0);
    assert_ne!(r3051.jump_address, r3051.program_counter);
    assert!(!r3051.jump_pending);
    assert!(r3051.is_branch);
}

#[test]
fn test_bne_registers_not_equal() {

    let mut r3051 = R3051::new();

    // Given register registers not equal (registers 1 and 2) and
    // an immediate of -4 (when left shifted 2 bits and sign
    // extended), jump address should be equal to PC and jump
    // pending.
    r3051.general_registers[1] = -1;
    let instruction = 0x1422FFFF;
    r3051.bne_instruction(instruction);

    assert_eq!(r3051.jump_address, r3051.program_counter);
    assert!(r3051.jump_pending);
    assert!(r3051.is_branch);
}

#[test]
fn test_break_in_branch_delay_slot() {

    let mut r3051 = R3051::new();

    // Given we are in a branch delay slot, this will be set
    // in exception, and origin will be PC - 4, with cause being
    // BP.
    r3051.prev_was_branch = true;
    r3051.break_instruction();

    assert_eq!(r3051.exception.exception_reason, MIPSExceptionReason::BP);
    assert_eq!(r3051.exception.program_counter_origin, r3051.program_counter - 4);
    assert!(r3051.exception.is_in_branch_delay_slot);
}

#[test]
fn test_break_not_in_branch_delay_slot() {

    let mut r3051 = R3051::new();

    // Given we are not in a branch delay slot, this will be unset
    // in exception, and origin will be PC, with cause being BP.
    r3051.break_instruction();

    assert_eq!(r3051.exception.exception_reason, MIPSExceptionReason::BP);
    assert_eq!(r3051.exception.program_counter_origin, r3051.program_counter);
    assert!(!r3051.exception.is_in_branch_delay_slot);
}

#[test]
fn test_cf2_reads_from_cp2_properly() {

    let mut r3051 = R3051::new();

    // Given we set regster 15 of CP2 to a value, that value should
    // be successfully read into register 1 of the CPU.
    r3051.gte.write_control_reg(15, 1337, false);
    let instruction = 0x48417800;
    r3051.cf2_instruction(instruction);

    assert_eq!(r3051.general_registers[1], 1337);
}

#[test]
fn test_ct2_writes_to_cp2_properly() {

    let mut r3051 = R3051::new();

    // Given we set regster 15 of CP2 to a value, that value should
    // be successfully read into register 1 of the CPU.
    r3051.general_registers[1] = 1337;
    let instruction = 0x48C17800;
    r3051.ct2_instruction(instruction);

    assert_eq!(r3051.gte.read_control_reg(15), 1337);
}