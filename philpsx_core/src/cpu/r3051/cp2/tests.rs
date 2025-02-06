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

    let mut cp2 = CP2::new();
    for i in 0..32 {
        cp2.write_control_reg(i, 0x8000, true);
        let output = cp2.control_registers[i as usize];
        assert_eq!(output, 0x8000);
    }
}

#[test]
fn write_data_reg_should_work() {

    let mut cp2 = CP2::new();
    for i in 0..32 {
        cp2.write_data_reg(i, 0x8000, true);
        let output = cp2.data_registers[i as usize];
        assert_eq!(output, 0x8000);
    }

    cp2 = CP2::new();
    for i in [7, 23, 29, 31] {
        cp2.write_data_reg(i, 0x8000, false);
        let output = cp2.data_registers[i as usize];
        assert_eq!(output, 0);
    }

    cp2 = CP2::new();
    cp2.write_data_reg(14, 0x8000, false);
    let output = cp2.data_registers[14];
    assert_eq!(output, 0x8000);
    let output = cp2.data_registers[15];
    assert_eq!(output, 0x8000);

    cp2 = CP2::new();
    cp2.data_registers[14] = 2;
    cp2.data_registers[13] = 1;
    cp2.write_data_reg(15, 0x8000, false);
    let output = cp2.data_registers[15];
    assert_eq!(output, 0x8000);
    let output = cp2.data_registers[14];
    assert_eq!(output, 0x8000);
    let output = cp2.data_registers[13];
    assert_eq!(output, 2);
    let output = cp2.data_registers[12];
    assert_eq!(output, 1);

    cp2 = CP2::new();
    cp2.write_data_reg(28, 0x7FFF, false);
    let output = cp2.data_registers[9];
    assert_eq!(output, 0xF80);
    let output = cp2.data_registers[10];
    assert_eq!(output, 0xF80);
    let output = cp2.data_registers[11];
    assert_eq!(output, 0xF80);

    cp2 = CP2::new();
    cp2.write_data_reg(8, 0x8000, false);
    let output = cp2.data_registers[8];
    assert_eq!(output, 0x8000);
}

#[test]
fn all_gte_functions_should_give_proper_cycle_count() {

    let mut cp2 = CP2::new();

    let cycles = cp2.gte_function(0x01);
    assert_eq!(cycles, 15);

    let cycles = cp2.gte_function(0x06);
    assert_eq!(cycles, 8);

    let cycles = cp2.gte_function(0x0C);
    assert_eq!(cycles, 6);

    let cycles = cp2.gte_function(0x10);
    assert_eq!(cycles, 8);

    let cycles = cp2.gte_function(0x11);
    assert_eq!(cycles, 8);

    let cycles = cp2.gte_function(0x12);
    assert_eq!(cycles, 8);

    let cycles = cp2.gte_function(0x13);
    assert_eq!(cycles, 19);

    let cycles = cp2.gte_function(0x14);
    assert_eq!(cycles, 13);

    let cycles = cp2.gte_function(0x16);
    assert_eq!(cycles, 44);

    let cycles = cp2.gte_function(0x1B);
    assert_eq!(cycles, 17);

    let cycles = cp2.gte_function(0x1C);
    assert_eq!(cycles, 11);

    let cycles = cp2.gte_function(0x1E);
    assert_eq!(cycles, 14);

    let cycles = cp2.gte_function(0x20);
    assert_eq!(cycles, 30);

    let cycles = cp2.gte_function(0x28);
    assert_eq!(cycles, 5);

    let cycles = cp2.gte_function(0x29);
    assert_eq!(cycles, 8);

    let cycles = cp2.gte_function(0x2A);
    assert_eq!(cycles, 17);

    let cycles = cp2.gte_function(0x2D);
    assert_eq!(cycles, 5);

    let cycles = cp2.gte_function(0x2E);
    assert_eq!(cycles, 6);

    let cycles = cp2.gte_function(0x30);
    assert_eq!(cycles, 23);

    let cycles = cp2.gte_function(0x3D);
    assert_eq!(cycles, 5);

    let cycles = cp2.gte_function(0x3E);
    assert_eq!(cycles, 5);

    let cycles = cp2.gte_function(0x3F);
    assert_eq!(cycles, 39);

    let cycles = cp2.gte_function(0);
    assert_eq!(cycles, 0);
}