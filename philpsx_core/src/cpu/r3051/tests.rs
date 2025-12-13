// SPDX-License-Identifier: GPL-3.0
// tests.rs - Copyright Phillip Potter, 2025, under GPLv3 only.

use philpsx_utility::CustomInteger;

use crate::{
    cpu::Cpu,
    cpu::CpuBridge,
    cpu::r3051::mips_exception::MIPSExceptionReason
};

use super::R3051;

// Implementation of CPU bridge specfically just for testing R3051 functionality.

const TEST_RAM_SIZE: usize = 2097152;

struct TestCpuBridge {
    ram: Vec<i8>
}

impl TestCpuBridge {
    fn new() -> Self {
        Self {
            ram: vec![0; TEST_RAM_SIZE],
        }
    }
}

impl CpuBridge for TestCpuBridge {

    fn append_sync_cycles(&mut self, _cpu: &mut dyn Cpu, _cycles: i32) {}

    fn how_how_many_stall_cycles(&self, _cpu: &mut dyn Cpu, _address: i32) -> i32 {
        0
    }

    fn ok_to_increment(&self, _cpu: &mut dyn Cpu, _address: i64) -> bool {
        true
    }

    fn scratchpad_enabled(&self, _cpu: &mut dyn Cpu) -> bool {
        true
    }

    fn instruction_cache_enabled(&self, _cpu: &mut dyn Cpu) -> bool {
        true
    }

    fn read_byte(&self, _cpu: &mut dyn Cpu, address: i32) -> i8 {
        self.ram[address as usize]
    }

    fn read_word(&self, _cpu: &mut dyn Cpu, address: i32) -> i32 {
        let temp_address = ((address as i64) & 0xFFFFFFFC) as usize;

        (((self.ram[temp_address] as i32) & 0xFF) << 24) |
        (((self.ram[temp_address + 1] as i32) & 0xFF) << 16) |
        (((self.ram[temp_address + 2] as i32) & 0xFF) << 8) |
        ((self.ram[temp_address + 3] as i32) & 0xFF)
    }

    fn write_byte(&mut self, _cpu: &mut dyn Cpu, address: i32, value: i8) {
        self.ram[address as usize] = value;
    }

    fn write_word(&mut self, cpu: &mut dyn Cpu, address: i32, value: i32) {
        let temp_address = ((address as i64) & 0xFFFFFFFC) as usize;

        self.ram[temp_address] = value.logical_rshift(24) as i8;
        self.ram[temp_address + 1] = value.logical_rshift(16) as i8;
        self.ram[temp_address + 2] = value.logical_rshift(8) as i8;
        self.ram[temp_address + 3] = value as i8;
    }

    fn increment_interrupt_counters(&mut self, cpu: &mut dyn Cpu) {}
}

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

#[test]
fn test_div_when_not_dividing_by_zero() {

    let mut r3051 = R3051::new();

    // Given dividend of -26 and divisor of 5, we should expect
    // a quotient of -5 and a remainder of -1.
    r3051.general_registers[1] = -26;
    r3051.general_registers[2] = 5;
    let instruction = 0x0022001A;
    r3051.div_instruction(instruction);

    assert_eq!(r3051.hi_reg, -1);
    assert_eq!(r3051.lo_reg, -5);
}

#[test]
fn test_div_when_dividing_by_zero() {

    let mut r3051 = R3051::new();

    // Given dividend of -26 and divisor of 0, we should expect
    // a quotient of -1 and a remainder of -26.
    r3051.general_registers[1] = -26;
    r3051.general_registers[2] = 0;
    let instruction = 0x0022001A;
    r3051.div_instruction(instruction);

    assert_eq!(r3051.hi_reg, -26);
    assert_eq!(r3051.lo_reg, -1);
}

#[test]
fn test_divu_when_not_dividing_by_zero() {

    let mut r3051 = R3051::new();

    // Given dividend of -26 and divisor of 5, we should expect
    // a quotient of 858,993,454 and a remainder of 0.
    r3051.general_registers[1] = -26;
    r3051.general_registers[2] = 5;
    let instruction = 0x0022001B;
    r3051.divu_instruction(instruction);

    assert_eq!(r3051.hi_reg, 0);
    assert_eq!(r3051.lo_reg, 858_993_454);
}

#[test]
fn test_divu_when_dividing_by_zero() {

    let mut r3051 = R3051::new();

    // Given dividend of -26 and divisor of 0, we should expect
    // a quotient of -1 and a remainder of -26 (as interpreted from i32).
    r3051.general_registers[1] = -26;
    r3051.general_registers[2] = 0;
    let instruction = 0x0022001B;
    r3051.divu_instruction(instruction);

    assert_eq!(r3051.hi_reg, -26);
    assert_eq!(r3051.lo_reg, -1);
}

#[test]
fn test_j_instruction_success() {

    let mut r3051 = R3051::new();

    // Given a target value of 0x2000000, we should end up with a jump address
    // of 0xB8000000 and a pending jump.
    let instruction = 0x0A000000;
    r3051.j_instruction(instruction);

    assert_eq!(r3051.jump_address, 0xB8000000_u32 as i32);
    assert!(r3051.jump_pending);
    assert!(r3051.is_branch);
}

#[test]
fn test_jal_instruction_success() {

    let mut r3051 = R3051::new();

    // Given an encoded target value of 0x2000000, we should end up with a jump address
    // of 0xB8000000 and a pending jump. Also, we should have address of instruction
    // after branch delay slot in register 31.
    let instruction = 0x0E000000;
    r3051.jal_instruction(instruction);

    assert_eq!(r3051.jump_address, 0xB8000000_u32 as i32);
    assert_eq!(r3051.general_registers[31], r3051.program_counter + 8);
    assert!(r3051.jump_pending);
    assert!(r3051.is_branch);
}

#[test]
fn test_jalr_instruction_success() {

    let mut r3051 = R3051::new();

    // Given a target value of 0xB8000000 in register 1, we should end up with a jump
    // address of 0xB8000000 and a pending jump. Also, we should have address of instruction
    // after branch delay slot in register 2.
    r3051.general_registers[1] = 0xB8000000_u32 as i32;
    let instruction = 0x00201009;
    r3051.jalr_instruction(instruction);

    assert_eq!(r3051.jump_address, 0xB8000000_u32 as i32);
    assert_eq!(r3051.general_registers[2], r3051.program_counter + 8);
    assert!(r3051.jump_pending);
    assert!(r3051.is_branch);
}

#[test]
fn test_jr_instruction_success() {

    let mut r3051 = R3051::new();

    // Given a target value of 0xB8000000 in register 1, we should end up with a jump
    // address of 0xB8000000 and a pending jump.
    r3051.general_registers[1] = 0xB8000000_u32 as i32;
    let instruction = 0x00200008;
    r3051.jr_instruction(instruction);

    assert_eq!(r3051.jump_address, 0xB8000000_u32 as i32);
    assert!(r3051.jump_pending);
    assert!(r3051.is_branch);
}

#[test]
fn test_lb_instruction_success() {

    let mut r3051 = R3051::new();
    let mut test_bridge = TestCpuBridge::new();

    // Given an initial address of 0xFFFF in register 1 and an offset of
    // -0x7FFF, we should attempt to read a byte from address 0x8000 and
    // store it to register 2, and that byte should be sign extended to
    // 0xFFFFFFFF.
    r3051.general_registers[1] = 0xFFFF;
    test_bridge.ram[0x8000] = 0xFF_u8 as i8;
    let instruction = 0x80228001_u32 as i32;
    r3051.lb_instruction(&mut test_bridge, instruction);

    assert_eq!(r3051.general_registers[2], 0xFFFFFFFF_u32 as i32);
}

#[test]
fn test_lb_instruction_banned_address() {

    let mut r3051 = R3051::new();
    let mut test_bridge = TestCpuBridge::new();

    // Given an initial address of 0x80000000 in register 1 and an offset of
    // 0, we should attempt to read a byte from address 0x80000000 and this
    // should trigger an exception as we are in 'user' mode.
    r3051.general_registers[1] = 0x80000000_u32 as i32;
    let instruction = 0x80220000_u32 as i32;
    let cp0_status_reg_with_user_mode = r3051.sccp.read_reg(12) | 0x2;
    r3051.sccp.write_reg(12, cp0_status_reg_with_user_mode, false);
    r3051.lb_instruction(&mut test_bridge, instruction);

    assert_eq!(r3051.general_registers[2], 0);
    assert_eq!(r3051.exception.bad_address, 0x80000000_u32 as i32);
    assert_eq!(r3051.exception.exception_reason, MIPSExceptionReason::ADEL);
    assert_eq!(r3051.exception.program_counter_origin, r3051.program_counter);
}

#[test]
fn test_lbu_instruction_success() {

    let mut r3051 = R3051::new();
    let mut test_bridge = TestCpuBridge::new();

    // Given an initial address of 0xFFFF in register 1 and an offset of
    // -0x7FFF, we should attempt to read a byte from address 0x8000 and
    // store it to register 2, and that byte should not be sign extended.
    r3051.general_registers[1] = 0xFFFF;
    test_bridge.ram[0x8000] = 0xFF_u8 as i8;
    let instruction = 0x90228001_u32 as i32;
    r3051.lbu_instruction(&mut test_bridge, instruction);

    assert_eq!(r3051.general_registers[2], 0xFF);
}

#[test]
fn test_lbu_instruction_banned_address() {

    let mut r3051 = R3051::new();
    let mut test_bridge = TestCpuBridge::new();

    // Given an initial address of 0x80000000 in register 1 and an offset of
    // 0, we should attempt to read a byte from address 0x80000000 and this
    // should trigger an exception as we are in 'user' mode.
    r3051.general_registers[1] = 0x80000000_u32 as i32;
    let instruction = 0x90220000_u32 as i32;
    let cp0_status_reg_with_user_mode = r3051.sccp.read_reg(12) | 0x2;
    r3051.sccp.write_reg(12, cp0_status_reg_with_user_mode, false);
    r3051.lbu_instruction(&mut test_bridge, instruction);

    assert_eq!(r3051.general_registers[2], 0);
    assert_eq!(r3051.exception.bad_address, 0x80000000_u32 as i32);
    assert_eq!(r3051.exception.exception_reason, MIPSExceptionReason::ADEL);
    assert_eq!(r3051.exception.program_counter_origin, r3051.program_counter);
}

#[test]
fn test_lh_instruction_success() {

    let mut r3051 = R3051::new();
    let mut test_bridge = TestCpuBridge::new();

    // Given an initial address of 0xFFFF in register 1 and an offset of
    // -0x7FFF, we should attempt to read a half word from address 0x8000 and
    // store it to register 2, and it should be sign extended.
    r3051.general_registers[1] = 0xFFFF;
    test_bridge.ram[0x8000] = 0xFE_u8 as i8;
    test_bridge.ram[0x8001] = 0xFF_u8 as i8;
    let instruction = 0x84228001_u32 as i32;
    r3051.lh_instruction(&mut test_bridge, instruction);

    assert_eq!(r3051.general_registers[2], 0xFFFFFFFE_u32 as i32);
}

#[test]
fn test_lh_instruction_banned_address() {

    let mut r3051 = R3051::new();
    let mut test_bridge = TestCpuBridge::new();

    // Given an initial address of 0x80000000 in register 1 and an offset of
    // 0, we should attempt to read a half word from address 0x80000000 and this
    // should trigger an exception as we are in 'user' mode.
    r3051.general_registers[1] = 0x80000000_u32 as i32;
    let instruction = 0x84220000_u32 as i32;
    let cp0_status_reg_with_user_mode = r3051.sccp.read_reg(12) | 0x2;
    r3051.sccp.write_reg(12, cp0_status_reg_with_user_mode, false);
    r3051.lh_instruction(&mut test_bridge, instruction);

    assert_eq!(r3051.general_registers[2], 0);
    assert_eq!(r3051.exception.bad_address, 0x80000000_u32 as i32);
    assert_eq!(r3051.exception.exception_reason, MIPSExceptionReason::ADEL);
    assert_eq!(r3051.exception.program_counter_origin, r3051.program_counter);
}

#[test]
fn test_lhu_instruction_success() {

    let mut r3051 = R3051::new();
    let mut test_bridge = TestCpuBridge::new();

    // Given an initial address of 0xFFFF in register 1 and an offset of
    // -0x7FFF, we should attempt to read a half word from address 0x8000 and
    // store it to register 2, and it should not be sign extended.
    r3051.general_registers[1] = 0xFFFF;
    test_bridge.ram[0x8000] = 0xFE_u8 as i8;
    test_bridge.ram[0x8001] = 0xFF_u8 as i8;
    let instruction = 0x94228001_u32 as i32;
    r3051.lhu_instruction(&mut test_bridge, instruction);

    assert_eq!(r3051.general_registers[2], 0xFFFE);
}

#[test]
fn test_lhu_instruction_banned_address() {

    let mut r3051 = R3051::new();
    let mut test_bridge = TestCpuBridge::new();

    // Given an initial address of 0x80000000 in register 1 and an offset of
    // 0, we should attempt to read a half word from address 0x80000000 and this
    // should trigger an exception as we are in 'user' mode.
    r3051.general_registers[1] = 0x80000000_u32 as i32;
    let instruction = 0x94220000_u32 as i32;
    let cp0_status_reg_with_user_mode = r3051.sccp.read_reg(12) | 0x2;
    r3051.sccp.write_reg(12, cp0_status_reg_with_user_mode, false);
    r3051.lhu_instruction(&mut test_bridge, instruction);

    assert_eq!(r3051.general_registers[2], 0);
    assert_eq!(r3051.exception.bad_address, 0x80000000_u32 as i32);
    assert_eq!(r3051.exception.exception_reason, MIPSExceptionReason::ADEL);
    assert_eq!(r3051.exception.program_counter_origin, r3051.program_counter);
}

#[test]
fn test_lui_instruction_success() {

    let mut r3051 = R3051::new();

    // Given an immediate value of 0x1234 and a target register of
    // 1, the value of the target register should become 0x12340000.
    let instruction = 0x3C011234;
    r3051.lui_instruction(instruction);

    assert_eq!(r3051.general_registers[1], 0x12340000);
}

#[test]
fn test_lw_instruction_success() {

    let mut r3051 = R3051::new();
    let mut test_bridge = TestCpuBridge::new();

    // Given an initial address of 0xFFFF in register 1 and an offset of
    // -0x7FFF, we should attempt to read a word from address 0x8000 and
    // store it to register 2.
    r3051.general_registers[1] = 0xFFFF;
    test_bridge.ram[0x8000] = 0x78;
    test_bridge.ram[0x8001] = 0x56;
    test_bridge.ram[0x8002] = 0x34;
    test_bridge.ram[0x8003] = 0x12;
    let instruction = 0x8C228001_u32 as i32;
    r3051.lw_instruction(&mut test_bridge, instruction);

    assert_eq!(r3051.general_registers[2], 0x12345678);
}

#[test]
fn test_lw_instruction_banned_address() {

    let mut r3051 = R3051::new();
    let mut test_bridge = TestCpuBridge::new();

    // Given an initial address of 0x80000000 in register 1 and an offset of
    // 0, we should attempt to read a word from address 0x80000000 and this
    // should trigger an exception as we are in 'user' mode.
    r3051.general_registers[1] = 0x80000000_u32 as i32;
    let instruction = 0x8C220000_u32 as i32;
    let cp0_status_reg_with_user_mode = r3051.sccp.read_reg(12) | 0x2;
    r3051.sccp.write_reg(12, cp0_status_reg_with_user_mode, false);
    r3051.lw_instruction(&mut test_bridge, instruction);

    assert_eq!(r3051.general_registers[2], 0);
    assert_eq!(r3051.exception.bad_address, 0x80000000_u32 as i32);
    assert_eq!(r3051.exception.exception_reason, MIPSExceptionReason::ADEL);
    assert_eq!(r3051.exception.program_counter_origin, r3051.program_counter);
}

#[test]
fn test_lwc2_instruction_success() {

    let mut r3051 = R3051::new();
    let mut test_bridge = TestCpuBridge::new();

    // Given an initial address of 0xFFFF in register 1 and an offset of
    // -0x7FFF, we should attempt to read a word from address 0x8000 and
    // store it to CP2 register 2.
    r3051.general_registers[1] = 0xFFFF;
    test_bridge.ram[0x8000] = 0x78;
    test_bridge.ram[0x8001] = 0x56;
    test_bridge.ram[0x8002] = 0x34;
    test_bridge.ram[0x8003] = 0x12;
    let instruction = 0xC82F8001_u32 as i32;
    r3051.lwc2_instruction(&mut test_bridge, instruction);
    let cp2_data_reg = r3051.gte.read_data_reg(15);

    assert_eq!(cp2_data_reg, 0x12345678);
}

#[test]
fn test_lwc2_instruction_banned_address() {

    let mut r3051 = R3051::new();
    let mut test_bridge = TestCpuBridge::new();

    // Given an initial address of 0x80000000 in register 1 and an offset of
    // 0, we should attempt to read a word from address 0x80000000 and this
    // should trigger an exception as we are in 'user' mode.
    r3051.general_registers[1] = 0x80000000_u32 as i32;
    let instruction = 0xC82F0000_u32 as i32;
    let cp0_status_reg_with_user_mode = r3051.sccp.read_reg(12) | 0x2;
    r3051.sccp.write_reg(12, cp0_status_reg_with_user_mode, false);
    r3051.lwc2_instruction(&mut test_bridge, instruction);
    let cp2_data_reg = r3051.gte.read_data_reg(15);

    assert_eq!(r3051.general_registers[2], 0);
    assert_eq!(r3051.exception.bad_address, 0x80000000_u32 as i32);
    assert_eq!(r3051.exception.exception_reason, MIPSExceptionReason::ADEL);
    assert_eq!(r3051.exception.program_counter_origin, r3051.program_counter);
    assert_eq!(cp2_data_reg, 0);
}

#[test]
fn test_lwl_instruction_success() {

    let mut r3051 = R3051::new();
    let mut test_bridge = TestCpuBridge::new();

    // Given an initial address of 0xFFFF in register 1 and an offset of
    // -0x7FFE, we should attempt to read a word from address 0x8000, shift
    // it left by 2 bytes, and then combine it with the lower two bytes of
    // register 2.
    r3051.general_registers[1] = 0xFFFF;
    r3051.general_registers[2] = 0xFFFF8765_u32 as i32;
    test_bridge.ram[0x8000] = 0x78;
    test_bridge.ram[0x8001] = 0x56;
    test_bridge.ram[0x8002] = 0x34;
    test_bridge.ram[0x8003] = 0x12;
    let instruction = 0x88228002_u32 as i32;
    r3051.lwl_instruction(&mut test_bridge, instruction);

    assert_eq!(r3051.general_registers[2], 0x56788765);
}

#[test]
fn test_lwl_instruction_banned_address() {

    let mut r3051 = R3051::new();
    let mut test_bridge = TestCpuBridge::new();

    // Given an initial address of 0x80000000 in register 1 and an offset of
    // 0, we should attempt to read a word from address 0x80000000 and this
    // should trigger an exception as we are in 'user' mode.
    r3051.general_registers[1] = 0x80000000_u32 as i32;
    let instruction = 0x88220000_u32 as i32;
    let cp0_status_reg_with_user_mode = r3051.sccp.read_reg(12) | 0x2;
    r3051.sccp.write_reg(12, cp0_status_reg_with_user_mode, false);
    r3051.lwl_instruction(&mut test_bridge, instruction);

    assert_eq!(r3051.general_registers[2], 0);
    assert_eq!(r3051.exception.bad_address, 0x80000000_u32 as i32);
    assert_eq!(r3051.exception.exception_reason, MIPSExceptionReason::ADEL);
    assert_eq!(r3051.exception.program_counter_origin, r3051.program_counter);
}

#[test]
fn test_lwr_instruction_success() {

    let mut r3051 = R3051::new();
    let mut test_bridge = TestCpuBridge::new();

    // Given an initial address of 0xFFFF in register 1 and an offset of
    // -0x7FFE, we should attempt to read a word from address 0x8000, shift
    // it left by 2 bytes, and then combine it with the lower two bytes of
    // register 2.
    r3051.general_registers[1] = 0xFFFF;
    r3051.general_registers[2] = 0x4321FFFF_u32 as i32;
    test_bridge.ram[0x8000] = 0x78;
    test_bridge.ram[0x8001] = 0x56;
    test_bridge.ram[0x8002] = 0x34;
    test_bridge.ram[0x8003] = 0x12;
    let instruction = 0x98228003_u32 as i32;
    r3051.lwr_instruction(&mut test_bridge, instruction);

    assert_eq!(r3051.general_registers[2], 0x43211234);
}

#[test]
fn test_lwr_instruction_banned_address() {

    let mut r3051 = R3051::new();
    let mut test_bridge = TestCpuBridge::new();

    // Given an initial address of 0x80000000 in register 1 and an offset of
    // 0, we should attempt to read a word from address 0x80000000 and this
    // should trigger an exception as we are in 'user' mode.
    r3051.general_registers[1] = 0x80000000_u32 as i32;
    let instruction = 0x98220000_u32 as i32;
    let cp0_status_reg_with_user_mode = r3051.sccp.read_reg(12) | 0x2;
    r3051.sccp.write_reg(12, cp0_status_reg_with_user_mode, false);
    r3051.lwr_instruction(&mut test_bridge, instruction);

    assert_eq!(r3051.general_registers[2], 0);
    assert_eq!(r3051.exception.bad_address, 0x80000000_u32 as i32);
    assert_eq!(r3051.exception.exception_reason, MIPSExceptionReason::ADEL);
    assert_eq!(r3051.exception.program_counter_origin, r3051.program_counter);
}

#[test]
fn test_mf0_instruction_success() {

    let mut r3051 = R3051::new();

    // Given a value in CP0 register 8, we should read it to CPU register 1.
    r3051.sccp.write_reg(8, 0x12345678, false);
    let instruction = 0x40014000;
    r3051.mf0_instruction(instruction);

    assert_eq!(r3051.general_registers[1], 0x12345678);
}

#[test]
fn test_mf0_instruction_banned_cp0_registers() {

    // Reading a value from CP0 registers 0, 1, 2, 4 or 10
    // should trigger an exception.
    for i in [0, 1, 2, 4, 10] {
        let mut r3051 = R3051::new();
    
        let masked_i = i & 0x1F;
        let instruction = 0x40010000 | (masked_i << 11);
        r3051.mf0_instruction(instruction);

        assert_eq!(r3051.exception.exception_reason, MIPSExceptionReason::RI);
        assert!(!r3051.exception.is_in_branch_delay_slot);
        assert_eq!(r3051.exception.program_counter_origin, r3051.program_counter);
    }
}

#[test]
fn test_mf2_instruction_success() {

    let mut r3051 = R3051::new();

    // Given a value in CP2 data register 15, we should read it to CPU register 1.
    r3051.gte.write_data_reg(15, 0x12345678, false);
    let instruction = 0x48017800;
    r3051.mf2_instruction(instruction);

    assert_eq!(r3051.general_registers[1], 0x12345678);
}

#[test]
fn test_mfhi_instruction_success() {

    let mut r3051 = R3051::new();

    // Given a value in CPU 'hi' register, we should read it to CPU register 1.
    r3051.hi_reg = 0x12345678;
    let instruction = 0x00000810;
    r3051.mfhi_instruction(instruction);

    assert_eq!(r3051.general_registers[1], 0x12345678);
}

#[test]
fn test_mflo_instruction_success() {

    let mut r3051 = R3051::new();

    // Given a value in CPU 'lo' register, we should read it to CPU register 1.
    r3051.lo_reg = 0x12345678;
    let instruction = 0x00000812;
    r3051.mflo_instruction(instruction);

    assert_eq!(r3051.general_registers[1], 0x12345678);
}

#[test]
fn test_mt0_instruction_success() {

    let mut r3051 = R3051::new();

    // Given a value in CPU register 1, we should write it to CP0 register 8.
    r3051.general_registers[1] = 0x12345678;
    let instruction = 0x40814000;
    r3051.mt0_instruction(instruction);

    assert_eq!(r3051.sccp.read_reg(8), 0x12345678);
}

#[test]
fn test_mt2_instruction_success() {

    let mut r3051 = R3051::new();

    // Given a value in CPU register 1, we should write it to CP2 data register 12.
    r3051.general_registers[1] = 0x12345678;
    let instruction = 0x48816000;
    r3051.mt2_instruction(instruction);

    assert_eq!(r3051.gte.read_data_reg(12), 0x12345678);
}

#[test]
fn test_mthi_instruction_success() {

    let mut r3051 = R3051::new();

    // Given a value in CPU register 1, we should read it to CPU 'hi' register.
    r3051.general_registers[1] = 0x12345678;

    let instruction = 0x00200011;
    r3051.mthi_instruction(instruction);

    assert_eq!(r3051.hi_reg, 0x12345678);
}

#[test]
fn test_mtlo_instruction_success() {

    let mut r3051 = R3051::new();

    // Given a value in CPU 'lo' register, we should read it to CPU register 1.
    r3051.general_registers[1] = 0x12345678;
    let instruction = 0x00200013;
    r3051.mtlo_instruction(instruction);

    assert_eq!(r3051.lo_reg, 0x12345678);
}

#[test]
fn test_mult_instruction_success() {

    let mut r3051 = R3051::new();

    // Given two signed values in registers 1 and 2, we should store
    // the multiplied 64-bit result into the 'hi' and 'lo' registers.
    r3051.general_registers[1] = 1337;
    r3051.general_registers[2] = -1;
    let instruction = 0x00220018;
    r3051.mult_instruction(instruction);

    assert_eq!(r3051.hi_reg, 0xFFFFFFFF_u32 as i32);
    assert_eq!(r3051.lo_reg, 0xFFFFFAC7_u32 as i32);
}

#[test]
fn test_multu_instruction_success() {

    let mut r3051 = R3051::new();

    // Given two signed values in registers 1 and 2, both interpreted
    // as unsigned, we should store the multiplied 64-bit result into
    // the 'hi' and 'lo' registers.
    r3051.general_registers[1] = 1337;
    r3051.general_registers[2] = -1;
    let instruction = 0x00220019;
    r3051.multu_instruction(instruction);

    assert_eq!(r3051.hi_reg, 0x00000538);
    assert_eq!(r3051.lo_reg, 0xFFFFFAC7_u32 as i32);
}

#[test]
fn test_nor_instruction_success() {

    let mut r3051 = R3051::new();

    // Given two values in register 1 and 2, we should store the result
    // of NOR-ing them in register 3.
    r3051.general_registers[1] = 0xAAAA0000_u32 as i32;
    r3051.general_registers[2] = 0x55550000;
    let instruction = 0x00221827;
    r3051.nor_instruction(instruction);

    assert_eq!(r3051.general_registers[3], 0x0000FFFF);
}

#[test]
fn test_or_instruction_success() {

    let mut r3051 = R3051::new();

    // Given two values in register 1 and 2, we should store the result
    // of OR-ing them in register 3.
    r3051.general_registers[1] = 0xAAAAAAAA_u32 as i32;
    r3051.general_registers[2] = 0x55555555;
    let instruction = 0x00221825;
    r3051.or_instruction(instruction);

    assert_eq!(r3051.general_registers[3], 0xFFFFFFFF_u32 as i32);
}

#[test]
fn test_ori_instruction_success() {

    let mut r3051 = R3051::new();

    // Given a value in register 1 and the immediate value, we should
    // store the result of OR-ing them in register 2.
    r3051.general_registers[1] = 0xFFFF0000_u32 as i32;
    let instruction = 0x3422FFFF;
    r3051.ori_instruction(instruction);

    assert_eq!(r3051.general_registers[2], 0xFFFFFFFF_u32 as i32);
}

#[test]
fn test_rfe_instruction_success() {

    let mut r3051 = R3051::new();

    // Given that the bits 5-2 of the CP0 status register are set and
    // bits 1-0 unset, after RFE we should shuffle right two bits, such
    // that bits 3-0 are then set and bits 5-4 are also still.
    let new_status_reg = (r3051.sccp.read_reg(12) & (0xFFFFFFC0_u32 as i32)) | 0x3C;
    r3051.sccp.write_reg(12, new_status_reg, false);
    r3051.rfe_instruction();

    assert_eq!(r3051.sccp.read_reg(12) & 0x3F, 0x3F);
}

#[test]
fn test_sb_instruction_success() {

    let mut r3051 = R3051::new();
    let mut test_bridge = TestCpuBridge::new();

    // Given an initial address of 0xFFFF in register 1 and an offset of
    // -0x7FFF, we should attempt to store a byte to address 0x8000 from
    // the lowest byte of register 2.
    r3051.general_registers[1] = 0xFFFF;
    r3051.general_registers[2] = 0x12345678;
    let instruction = 0xA0228001_u32 as i32;
    r3051.sb_instruction(&mut test_bridge, instruction);

    assert_eq!(test_bridge.ram[0x8000], 0x78);
}

#[test]
fn test_sb_instruction_banned_address() {

    let mut r3051 = R3051::new();
    let mut test_bridge = TestCpuBridge::new();

    // Given an initial address of 0x80000000 in register 1 and an offset of
    // 0, we should attempt to read a byte from address 0x80000000 and this
    // should trigger an exception as we are in 'user' mode.
    r3051.general_registers[1] = 0x80000000_u32 as i32;
    let instruction = 0xA0220000_u32 as i32;
    let cp0_status_reg_with_user_mode = r3051.sccp.read_reg(12) | 0x2;
    r3051.sccp.write_reg(12, cp0_status_reg_with_user_mode, false);
    r3051.sb_instruction(&mut test_bridge, instruction);

    assert_eq!(r3051.general_registers[2], 0);
    assert_eq!(r3051.exception.bad_address, 0x80000000_u32 as i32);
    assert_eq!(r3051.exception.exception_reason, MIPSExceptionReason::ADES);
    assert_eq!(r3051.exception.program_counter_origin, r3051.program_counter);
}

#[test]
fn test_sh_instruction_success() {

    let mut r3051 = R3051::new();
    let mut test_bridge = TestCpuBridge::new();

    // Given an initial address of 0xFFFF in register 1 and an offset of
    // -0x7FFF, we should attempt to store a half word to address 0x8000
    // from the lower half word of register 2.
    r3051.general_registers[1] = 0xFFFF;
    r3051.general_registers[2] = 0x12345678;
    let instruction = 0xA4228001_u32 as i32;
    r3051.sh_instruction(&mut test_bridge, instruction);

    assert_eq!(test_bridge.ram[0x8000], 0x78);
    assert_eq!(test_bridge.ram[0x8001], 0x56);
}

#[test]
fn test_sh_instruction_banned_address() {

    let mut r3051 = R3051::new();
    let mut test_bridge = TestCpuBridge::new();

    // Given an initial address of 0x80000000 in register 1 and an offset of
    // 0, we should attempt to read a byte from address 0x80000000 and this
    // should trigger an exception as we are in 'user' mode.
    r3051.general_registers[1] = 0x80000000_u32 as i32;
    let instruction = 0xA4220000_u32 as i32;
    let cp0_status_reg_with_user_mode = r3051.sccp.read_reg(12) | 0x2;
    r3051.sccp.write_reg(12, cp0_status_reg_with_user_mode, false);
    r3051.sh_instruction(&mut test_bridge, instruction);

    assert_eq!(r3051.general_registers[2], 0);
    assert_eq!(r3051.exception.bad_address, 0x80000000_u32 as i32);
    assert_eq!(r3051.exception.exception_reason, MIPSExceptionReason::ADES);
    assert_eq!(r3051.exception.program_counter_origin, r3051.program_counter);
}

#[test]
fn test_sll_instruction_success() {

    let mut r3051 = R3051::new();

    // Given an initial value in register 1 and shift value encoded in
    // the instruction, we should shift this value left by the required
    // number of bits and store it in register 2.
    r3051.general_registers[1] = 0x12345678;
    let instruction = 0x00011100;
    r3051.sll_instruction(instruction);

    assert_eq!(r3051.general_registers[2], 0x23456780);
}

#[test]
fn test_sllv_instruction_success() {

    let mut r3051 = R3051::new();

    // Given an initial value in register 2 and shift value encoded in
    // register 1, we should shift this value left by the required
    // number of bits and store it in register 3.
    r3051.general_registers[1] = 4;
    r3051.general_registers[2] = 0x12345678;
    let instruction = 0x00221804;
    r3051.sllv_instruction(instruction);

    assert_eq!(r3051.general_registers[3], 0x23456780);
}

#[test]
fn test_slt_instruction_less_than() {

    let mut r3051 = R3051::new();

    // Given two values in registers 1 and 2 and a burneer value in
    // register 3, if the first is lower than the second then we
    // should set register 3 to 1.
    r3051.general_registers[1] = 4;
    r3051.general_registers[2] = 5;
    r3051.general_registers[3] = 6;
    let instruction = 0x0022182A;
    r3051.slt_instruction(instruction);

    assert_eq!(r3051.general_registers[3], 1);
}

#[test]
fn test_slt_instruction_not_less_than() {

    let mut r3051 = R3051::new();

    // Given two values in registers 1 and 2 and a burneer value in
    // register 3, if the first is not lower than the second then we
    // should set register 3 to 0.
    r3051.general_registers[1] = 5;
    r3051.general_registers[2] = 5;
    r3051.general_registers[3] = 6;
    let instruction = 0x0022182A;
    r3051.slt_instruction(instruction);

    assert_eq!(r3051.general_registers[3], 0);
}