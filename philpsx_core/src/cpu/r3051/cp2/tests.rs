// SPDX-License-Identifier: GPL-3.0
// tests.rs - Copyright Phillip Potter, 2025, under GPLv3 only.

use super::CP2;
use super::InstructionVariant;

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

#[test]
fn casting_should_extend_sign_too() {

    let input: i32 = -1;
    let output = input as i64;    

    assert_eq!(output, -1_i64)
}

// Below tests are for opcodes themselves and thus use public functionality only, such
// that register manipulation via read/write routines works as expected. Also, the input
// data is mostly picked at random. This is more likely to uncover bugs when testing against
// the NOPSX debugger, and is the next best thing from exhaustively testing ever edge case
// which is too much for a passion project like this.
#[test]
fn rtps_should_product_correct_result() {

    let mut cp2 = CP2::new();

    // Setup TRX, TRY and TRZ.
    cp2.write_control_reg(5, 0x0C7ECDFC, false);
    cp2.write_control_reg(6, 0x1E844F9B, false);
    cp2.write_control_reg(7, 0x61153630, false);

    // Setup rotation matrix.
    cp2.write_control_reg(0, 0x48C52508, false); // RT12 | RT11.
    cp2.write_control_reg(1, 0xD41E63BA_u32 as i32, false); // RT21 | RT13.
    cp2.write_control_reg(2, 0xB6331DC8_u32 as i32, false); // RT23 | RT22.
    cp2.write_control_reg(3, 0xF7F7FD3A_u32 as i32, false); // RT32 | RT31.
    cp2.write_control_reg(4, 0x0000260E, false); // RT33.

    // Write ofx, ofy, h, dqa and dqb.
    cp2.write_control_reg(24, 0x094427BF, false); // ofx.
    cp2.write_control_reg(25, 0xF83A9D3C_u32 as i32, false); // ofy.
    cp2.write_control_reg(26, 0x00008CC7, false); // h.
    cp2.write_control_reg(27, 0x0000F6A1, false); // dqa.
    cp2.write_control_reg(28, 0xF87ECDA2_u32 as i32, false); // dqb.

    // Write VX0, VY0 and VZ0.
    cp2.write_data_reg(0, 0xB54BC06A_u32 as i32, false); // VY0 | VX0.
    cp2.write_data_reg(1, 0x00004C87, false); // VZ0.

    // Execute RTPS (with sf bit set to 0).
    cp2.handle_common_rtp(0x4BE00001, InstructionVariant::Single);

    // Now read registers.
    let ir1 = cp2.read_data_reg(9);
    let ir2 = cp2.read_data_reg(10);
    let ir3 = cp2.read_data_reg(11);
    let mac0 = cp2.read_data_reg(24);
    let mac1 = cp2.read_data_reg(25);
    let mac2 = cp2.read_data_reg(26);
    let mac3 = cp2.read_data_reg(27);
    let flag = cp2.read_control_reg(31);
    let irgb = cp2.read_data_reg(28);
    let orgb = cp2.read_data_reg(29);
    let sxy2 = cp2.read_data_reg(14);
    let sxyp = cp2.read_data_reg(15);
    let sz3 = cp2.read_data_reg(19);

    // Assert results are correct.
    assert_eq!(ir1, 0xFFFF8000_u32 as i32);
    assert_eq!(ir2, 0x7FFF);
    assert_eq!(ir3, 0x7FFF);
    assert_eq!(mac0, 0xF35790C9_u32 as i32);
    assert_eq!(mac1, 0xEC407F1D_u32 as i32);
    assert_eq!(mac2, 0x311F5EE9);
    assert_eq!(mac3, 0x61CBDBC3);
    assert_eq!(flag, 0x81C47000_u32 as i32);
    assert_eq!(irgb, 0x7FE0);
    assert_eq!(orgb, 0x7FE0);
    assert_eq!(sxy2, 0x03FFFC00);
    assert_eq!(sxyp, 0x03FFFC00);
    assert_eq!(sz3, 0xFFFF);
}

#[test]
fn nclip_should_product_correct_result() {

    let mut cp2 = CP2::new();

    // Setup SXY0, SXY1 and SXY2.
    cp2.write_data_reg(12, 0x29F498C6, false);
    cp2.write_data_reg(13, 0x1ACE8EBE, false);
    cp2.write_data_reg(14, 0x99A9E1F1_u32 as i32, false);

    // Execute NCLIP.
    cp2.handle_nclip(0x4BE00006);

    // Now read registers.
    let sxyp = cp2.read_data_reg(15);
    let mac0 = cp2.read_data_reg(24);
    let flag = cp2.read_control_reg(31);

    // Assert results are correct.
    assert_eq!(sxyp, 0x99A9E1F1_u32 as i32);
    assert_eq!(mac0, 0x09FBD1BA);
    assert_eq!(flag, 0);
}

#[test]
fn sqr_should_produce_correct_result() {

    let mut cp2 = CP2::new();

    // Set IR1, IR2 and IR3 to 0xB5, 0xB5 and 0xFFF, thus
    // triggering saturation only on IR3.
    cp2.write_data_reg(9, 0xB5, false);
    cp2.write_data_reg(10, 0xB5, false);
    cp2.write_data_reg(11, 0xFFF, false);

    // Execute SQR.
    cp2.handle_sqr(0x2800E04B);

    // Now read registers.
    let ir1 = cp2.read_data_reg(9);
    let ir2 = cp2.read_data_reg(10);
    let ir3 = cp2.read_data_reg(11);
    let mac1 = cp2.read_data_reg(25);
    let mac2 = cp2.read_data_reg(26);
    let mac3 = cp2.read_data_reg(27);
    let flag = cp2.read_control_reg(31);
    let irgb = cp2.read_data_reg(28);
    let orgb = cp2.read_data_reg(29);

    // Assert results are correct.
    assert_eq!(ir1, 0x7FF9);
    assert_eq!(ir2, 0x7FF9);
    assert_eq!(ir3, 0x7FFF);
    assert_eq!(mac1, 0x7FF9);
    assert_eq!(mac2, 0x7FF9);
    assert_eq!(mac3, 0xFFE001);
    assert_eq!(flag, 0x400000);
    assert_eq!(irgb, 0x7FFF);
    assert_eq!(orgb, 0x7FFF);
}