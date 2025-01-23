// SPDX-License-Identifier: GPL-3.0
// tests.rs - Copyright Phillip Potter, 2025, under GPLv3 only.

use super::CP2;

// Tests for the CP2 / Geometry Transformation Engine.

#[test]
fn read_control_reg_should_work() {

    let mut cp2 = CP2::new();
    for i in [25,31] {
        cp2.control_registers[i] = 0x8000;
        let output = cp2.read_control_reg(i as i32);
        assert_eq!(output, 0x8000);
    }

    for i in 26..=30 {
        cp2.control_registers[i] = 0x8000;
        let output = cp2.read_control_reg(i as i32);
        assert_eq!(output, 0xFFFF8000_u32 as i32);

        cp2.control_registers[i] = 0x7000;
        let output = cp2.read_control_reg(i as i32);
        assert_eq!(output, 0x7000);
    }
}

#[test]
fn read_data_reg_should_work() {

    let mut cp2 = CP2::new();
    for i in [1, 3, 5, 8, 9, 10, 11] {
        cp2.data_registers[i] = 0x8000;
        let output = cp2.read_data_reg(i as i32);
        assert_eq!(output, 0xFFFF8000_u32 as i32);

        cp2.data_registers[i] = 0x7000;
        let output = cp2.read_data_reg(i as i32);
        assert_eq!(output, 0x7000);
    }

    for i in [23, 28] {
        cp2.data_registers[i] = 1;
        let output = cp2.read_data_reg(i as i32);
        assert_eq!(output, 0);
    }

    cp2.data_registers[29] = 123;
    cp2.data_registers[11] = 0xF80;
    cp2.data_registers[10] = 0xF80;
    cp2.data_registers[9] = 0xF80;
    let output = cp2.read_data_reg(29);
    assert_eq!(output, 0x7FFF);

    cp2.data_registers[30] = 0xFFFE7FFF_u32 as i32;
    let output = cp2.read_data_reg(31);
    assert_eq!(output, 15);

    cp2.data_registers[30] = 0x7FFF_u32 as i32;
    let output = cp2.read_data_reg(31);
    assert_eq!(output, 17);

    cp2.data_registers[2] = 0x8000;
    let output = cp2.read_data_reg(2);
    assert_eq!(output, 0x8000);
}

#[test]
fn write_control_reg_should_work() {

    let mut cp2 = CP2::new();
    for i in 0..32 {
        cp2.write_control_reg(i, 0x8000, false);
        let output = cp2.control_registers[i as usize];
        assert_eq!(output, 0x8000);
    }
}