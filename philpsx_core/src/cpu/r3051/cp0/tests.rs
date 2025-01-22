// SPDX-License-Identifier: GPL-3.0
// tests.rs - Copyright Phillip Potter, 2025, under GPLv3 only.

use super::CP0;

// Tests for the CP0 / System Control Co-processor (CP0).

#[test]
fn status_register_read_should_work() {

    let mut cp0 = CP0::new();
    cp0.cp_registers[12] = 0xFFFFFFFF_u32 as i32;
    let output = cp0.read_reg(12);

    assert_eq!(output, 0xF27FFF3F_u32 as i32);
}

#[test]
fn cause_register_read_should_work() {

    let mut cp0 = CP0::new();
    cp0.cp_registers[13] = 0xFFFFFFFF_u32 as i32;
    let output = cp0.read_reg(13);

    assert_eq!(output, 0xB000FF7C_u32 as i32);
}

#[test]
fn prid_register_read_should_work() {

    let cp0 = CP0::new();
    let output = cp0.read_reg(15);

    assert_eq!(output, 0x00000002);
}

#[test]
fn registers_1_8_and_14_reads_should_return_as_is() {

    let mut cp0 = CP0::new();
    cp0.cp_registers[1] = 1;
    cp0.cp_registers[8] = 8;
    cp0.cp_registers[14] = 14;
    let output1 = cp0.read_reg(1);
    let output2 = cp0.read_reg(8);
    let output3 = cp0.read_reg(14);

    assert_eq!(output1, 1);
    assert_eq!(output2, 8);
    assert_eq!(output3, 14);
}

#[test]
fn any_other_register_read_should_return_0() {

    let mut cp0 = CP0::new();
    cp0.cp_registers[2] = 1;
    let output = cp0.read_reg(2);

    assert_eq!(output, 0);
}

#[test]
fn status_and_cause_registers_write_with_override() {

    let mut cp0 = CP0::new();
    cp0.cp_registers[12] = 0xFFFFFFFF_u32 as i32;
    cp0.cp_registers[13] = 0xFFFFFFFF_u32 as i32;
    cp0.write_reg(12, 0, true);
    cp0.write_reg(13, 0, true);
    let output1 = cp0.cp_registers[12];
    let output2 = cp0.cp_registers[13];

    assert_eq!(output1, 0);
    assert_eq!(output2, 0);
}

#[test]
fn status_and_cause_registers_write_without_override() {

    let mut cp0 = CP0::new();
    cp0.cp_registers[12] = 0xFFFFFFFF_u32 as i32;
    cp0.cp_registers[13] = 0xFFFFFFFF_u32 as i32;
    cp0.write_reg(12, 0, false);
    cp0.write_reg(13, 0, false);
    let output1 = cp0.cp_registers[12];
    let output2 = cp0.cp_registers[13];

    assert_eq!(output1, 0x0DB400C0);
    assert_eq!(output2, 0xFFFFFCFF_u32 as i32);
}

#[test]
fn status_and_cause_registers_write_merge() {

    let mut cp0 = CP0::new();
    cp0.cp_registers[12] = 0x0DB400C0;
    cp0.cp_registers[13] = 0xFFFFFCFF_u32 as i32;
    cp0.write_reg(12, 0xF24BFF3F_u32 as i32, false);
    cp0.write_reg(13, 0x00000300, false);
    let output1 = cp0.cp_registers[12];
    let output2 = cp0.cp_registers[13];

    assert_eq!(output1, 0xFFFFFFFF_u32 as i32);
    assert_eq!(output2, 0xFFFFFFFF_u32 as i32);
}

#[test]
fn status_and_cause_registers_write_only_writable_bits() {

    let mut cp0 = CP0::new();
    cp0.cp_registers[12] = 0x00000000;
    cp0.cp_registers[13] = 0x00000000;
    cp0.write_reg(12, 0xFF000000_u32 as i32, false);
    cp0.write_reg(13, 0x0000FF00, false);
    let output1 = cp0.cp_registers[12];
    let output2 = cp0.cp_registers[13];

    assert_eq!(output1, 0xF2000000_u32 as i32);
    assert_eq!(output2, 0x00000300);
}

#[test]
fn arbitrary_register_write_without_override() {

    let mut cp0 = CP0::new();
    cp0.cp_registers[1] = 0x00000000;
    cp0.write_reg(1, 0xFFFFFFFF_u32 as i32, false);
    let output = cp0.cp_registers[1];

    assert_eq!(output, 0xFFFFFFFF_u32 as i32);
}

#[test]
fn rfe_should_shift_status_bits_correctly() {

    let mut cp0 = CP0::new();
    cp0.cp_registers[12] = 0xF24BFF3C_u32 as i32;
    cp0.rfe();
    let output = cp0.cp_registers[12];

    assert_eq!(output, 0xF24BFF3F_u32 as i32);
}

#[test]
fn general_exception_vector_correct_when_bev_set() {

    let mut cp0 = CP0::new();
    cp0.cp_registers[12] = 0x00400000;
    let output = cp0.get_general_exception_vector();

    assert_eq!(output, 0xBFC00180_u32 as i32);
}

#[test]
fn general_exception_vector_correct_when_bev_unset() {

    let mut cp0 = CP0::new();
    cp0.cp_registers[12] = 0x00000000;
    let output = cp0.get_general_exception_vector();

    assert_eq!(output, 0x80000080_u32 as i32);
}

#[test]
fn setting_cache_miss_works() {

    let mut cp0 = CP0::new();
    cp0.cp_registers[12] = 0x00000000;
    cp0.set_cache_miss(true);
    let output = cp0.cp_registers[12];

    assert_eq!(output, 0x00080000);
}

#[test]
fn unsetting_cache_miss_works() {

    let mut cp0 = CP0::new();
    cp0.cp_registers[12] = 0x00080000;
    cp0.set_cache_miss(false);
    let output = cp0.cp_registers[12];

    assert_eq!(output, 0x00000000);
}

#[test]
fn virtual_to_physical_gives_expected_addresses() {

    let cp0 = CP0::new();
    let input1 = 0x00000000;
    let input2 = 0x7FFFFFFF;
    let input3 = 0x80000000_u32 as i32;
    let input4 = 0x9FFFFFFF_u32 as i32;
    let input5 = 0xA0000000_u32 as i32;
    let input6 = 0xBFFFFFFF_u32 as i32;
    let input7 = 0xC0000000_u32 as i32;
    let input8 = 0xFFFFFFFF_u32 as i32;

    let output1 = cp0.virtual_to_physical(input1);
    let output2 = cp0.virtual_to_physical(input2);
    let output3 = cp0.virtual_to_physical(input3);
    let output4 = cp0.virtual_to_physical(input4);
    let output5 = cp0.virtual_to_physical(input5);
    let output6 = cp0.virtual_to_physical(input6);
    let output7 = cp0.virtual_to_physical(input7);
    let output8 = cp0.virtual_to_physical(input8);

    assert_eq!(output1, 0x00000000);
    assert_eq!(output2, 0x7FFFFFFF);
    assert_eq!(output3, 0x00000000);
    assert_eq!(output4, 0x1FFFFFFF);
    assert_eq!(output5, 0x00000000);
    assert_eq!(output6, 0x1FFFFFFF);
    assert_eq!(output7, 0xC0000000_u32 as i32);
    assert_eq!(output8, 0xFFFFFFFF_u32 as i32);
}

#[test]
fn is_cacheable_gives_expected_results() {

    let cp0 = CP0::new();
    let input1 = 0x00000000;
    let input2 = 0x9FFFFFFFu32 as i32;
    let input3 = 0xA0000000_u32 as i32;
    let input4 = 0xFFFFFFFF_u32 as i32;

    let output1 = cp0.is_cacheable(input1);
    let output2 = cp0.is_cacheable(input2);
    let output3 = cp0.is_cacheable(input3);
    let output4 = cp0.is_cacheable(input4);

    assert!(output1);
    assert!(output2);
    assert!(!output3);
    assert!(!output4);
}

#[test]
fn kernel_mode_properly_detected() {

    let mut cp0 = CP0::new();
    let output1 = cp0.are_we_in_kernel_mode();
    cp0.cp_registers[12] |= 0x2;
    let output2 = cp0.are_we_in_kernel_mode();

    assert!(output1);
    assert!(!output2);
}

#[test]
fn user_mode_opposite_byte_ordering_properly_detected() {

    let mut cp0 = CP0::new();
    let output1 = cp0.user_mode_opposite_byte_ordering();
    cp0.cp_registers[12] |= 0x02000000;
    let output2 = cp0.user_mode_opposite_byte_ordering();

    assert!(!output1);
    assert!(output2);
}

#[test]
fn allowed_addresses_properly_detected() {

    let mut cp0 = CP0::new();
    let output1 = cp0.is_address_allowed(0xFFFFFFFF_u32 as i32);
    cp0.cp_registers[12] |= 0x2;
    let output2 = cp0.is_address_allowed(0xFFFFFFFF_u32 as i32);

    assert!(output1);
    assert!(!output2);
}

#[test]
fn data_cache_isolation_properly_detected() {

    let mut cp0 = CP0::new();
    let output1 = cp0.is_data_cache_isolated();
    cp0.cp_registers[12] |= 0x00010000;
    let output2 = cp0.is_data_cache_isolated();

    assert!(!output1);
    assert!(output2);
}

#[test]
fn co_processor_usability_properly_detected() {

    let mut cp0 = CP0::new();
    let output1 = cp0.is_co_processor_usable(0);
    let output2 = cp0.is_co_processor_usable(1);
    let output3 = cp0.is_co_processor_usable(2);
    let output4 = cp0.is_co_processor_usable(3);

    cp0.cp_registers[12] |= 0xF0000000_u32 as i32;
    let output5 = cp0.is_co_processor_usable(0);
    let output6 = cp0.is_co_processor_usable(1);
    let output7 = cp0.is_co_processor_usable(2);
    let output8 = cp0.is_co_processor_usable(3);

    assert!(!output1);
    assert!(!output2);
    assert!(!output3);
    assert!(!output4);
    assert!(output5);
    assert!(output6);
    assert!(output7);
    assert!(output8);
}