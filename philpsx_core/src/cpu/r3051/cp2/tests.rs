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

    for i in [26, 27, 29, 30] {
        cp2.control_registers[i] = 0x8000;
        let output = cp2.read_control_reg(i as i32);
        assert_eq!(output, 0xFFFF8000_u32 as i32);

        cp2.control_registers[i] = 0x7000;
        let output = cp2.read_control_reg(i as i32);
        assert_eq!(output, 0x7000);
    }

    for i in [4, 12, 20] {
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

    // Should always return 0.
    cp2.write_data_reg(23, 1, false);
    let output = cp2.read_data_reg(23);
    assert_eq!(output, 0);
    
    // IRGB - writable and readable.
    cp2.write_data_reg(28, 0x1F, false);
    let output = cp2.read_data_reg(28);
    assert_eq!(output, 0x1F);

    // ORGB - readonly mirror of IRGB.
    cp2.write_data_reg(29, 0, false);
    let output = cp2.read_data_reg(29);
    assert_eq!(output, 0x1F);
    
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
// the NOPSX debugger, and is the next best thing from exhaustively testing every edge case
// which is too much for a passion project like this.
#[test]
fn rtps_should_produce_correct_result() {

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
fn nclip_should_produce_correct_result() {

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
fn op_should_produce_correct_result() {

    let mut cp2 = CP2::new();

    // Setup IR1, IR2 and IR3.
    cp2.write_data_reg(9, 0xD0C0, false);
    cp2.write_data_reg(10, 0xEFEF, false);
    cp2.write_data_reg(11, 0xDE40, false);

    // Setup RT11, RT22 and RT33.
    cp2.write_control_reg(0, 0x9CDC, false);
    cp2.write_control_reg(2, 0x41B3, false);
    cp2.write_control_reg(4, 0x0014, false);

    // Execute OP (with sf bit set to 0 and lm bit set to 1).
    cp2.handle_op(0x4BE0040C);

    // Now read registers.
    let irgb = cp2.read_data_reg(28);
    let orgb = cp2.read_data_reg(29);
    let flag = cp2.read_control_reg(31);
    let ir1 = cp2.read_data_reg(9);
    let ir2 = cp2.read_data_reg(10);
    let ir3 = cp2.read_data_reg(11);
    let mac1 = cp2.read_data_reg(25);
    let mac2 = cp2.read_data_reg(26);
    let mac3 = cp2.read_data_reg(27);

    // Assert results are correct.
    assert_eq!(irgb, 0x7C00);
    assert_eq!(orgb, 0x7C00);
    assert_eq!(flag, 0x81C00000_u32 as i32);
    assert_eq!(ir1, 0);
    assert_eq!(ir2, 0);
    assert_eq!(ir3, 0x7FFF);
    assert_eq!(mac1, 0xF757E814_u32 as i32);
    assert_eq!(mac2, 0xF2EA5000_u32 as i32);
    assert_eq!(mac3, 0x12591F24);
}

#[test]
fn dpcs_should_produce_correct_result() {

    let mut cp2 = CP2::new();

    // Setup IR0.
    cp2.write_data_reg(8, 0x551A, false);

    // Setup RFC, GFC and BFC.
    cp2.write_control_reg(21, 0x7A63EA20, false);
    cp2.write_control_reg(22, 0xBCF74DF3_u32 as i32, false);
    cp2.write_control_reg(23, 0x9AA79D4C_u32 as i32, false);

    // Write RGBC.
    cp2.write_data_reg(6, 0xDDF85B4F_u32 as i32, false);

    // Execute DPCS (with sf bit set to 1 and lm bit set to 0).
    cp2.handle_common_dpc(0x4BE80010, InstructionVariant::Single);

    // Now read registers.
    let irgb = cp2.read_data_reg(28);
    let orgb = cp2.read_data_reg(29);
    let flag = cp2.read_control_reg(31);
    let ir1 = cp2.read_data_reg(9);
    let ir2 = cp2.read_data_reg(10);
    let ir3 = cp2.read_data_reg(11);
    let mac1 = cp2.read_data_reg(25);
    let mac2 = cp2.read_data_reg(26);
    let mac3 = cp2.read_data_reg(27);

    // Assert results are correct.
    assert_eq!(irgb, 0x001F);
    assert_eq!(orgb, 0x001F);
    assert_eq!(flag, 0x81F80000_u32 as i32);
    assert_eq!(ir1, 0x7FFF);
    assert_eq!(ir2, 0xFFFF8000_u32 as i32);
    assert_eq!(ir3, 0xFFFF8000_u32 as i32);
    assert_eq!(mac1, 0x0002ADBA);
    assert_eq!(mac2, 0xFFFD5CE0_u32 as i32);
    assert_eq!(mac3, 0xFFFD66B0_u32 as i32);
}

#[test]
fn intpl_should_produce_correct_result() {

    let mut cp2 = CP2::new();

    // Setup IR0, IR1, IR2 and IR3.
    cp2.write_data_reg(8, 0x5DF2, false);
    cp2.write_data_reg(9, 0x291A, false);
    cp2.write_data_reg(10, 0x6D59, false);
    cp2.write_data_reg(11, 0xC84E, false);

    // Setup RFC, GFC and BFC.
    cp2.write_control_reg(21, 0x6AF00171, false);
    cp2.write_control_reg(22, 0x1D0974AA, false);
    cp2.write_control_reg(23, 0x1DB84F2E, false);

    // Write CODE.
    cp2.write_data_reg(6, 0xE4000000_u32 as i32, false);

    // Execute INTPL (with sf bit set to 1 and lm bit set to 0).
    cp2.handle_intpl(0x4BE80011);

    // Now read registers.
    let rgb2 = cp2.read_data_reg(22);
    let irgb = cp2.read_data_reg(28);
    let orgb = cp2.read_data_reg(29);
    let flag = cp2.read_control_reg(31);
    let ir1 = cp2.read_data_reg(9);
    let ir2 = cp2.read_data_reg(10);
    let ir3 = cp2.read_data_reg(11);
    let mac1 = cp2.read_data_reg(25);
    let mac2 = cp2.read_data_reg(26);
    let mac3 = cp2.read_data_reg(27);

    // Assert results are correct.
    assert_eq!(rgb2, 0xE4FFFFFF_u32 as i32);
    assert_eq!(irgb, 0x7FFF);
    assert_eq!(orgb, 0x7FFF);
    assert_eq!(flag, 0x81F80000_u32 as i32);
    assert_eq!(ir1, 0x7FFF);
    assert_eq!(ir2, 0x7FFF);
    assert_eq!(ir3, 0x7FFF);
    assert_eq!(mac1, 0x000318A4);
    assert_eq!(mac2, 0x00035CE3);
    assert_eq!(mac3, 0x0002B7D8);
}

#[test]
fn mvmva_should_produce_correct_result() {

    let mut cp2 = CP2::new();

    // Setup RBK, GBK and BBK.
    cp2.write_control_reg(13, 0x31D403CE, false);
    cp2.write_control_reg(14, 0xC38A24BD_u32 as i32, false);
    cp2.write_control_reg(15, 0x3E2C6816, false);

    // Setup VX0, VY0 and VZ0.
    cp2.write_data_reg(0, 0x07E7D758, false);
    cp2.write_data_reg(1, 0xD24A, false);

    // Setup IR0, RT13 and RT22.
    cp2.write_data_reg(8, 0x52E1, false);
    cp2.write_control_reg(1, 0xDEC2, false);
    cp2.write_control_reg(2, 0xF851, false);

    // Execute MVMVA (with sf bit set to 0 and lm bit set to 1).
    cp2.handle_mvmva(0x4BE62412);

    // Now read registers.
    let irgb = cp2.read_data_reg(28);
    let orgb = cp2.read_data_reg(29);
    let flag = cp2.read_control_reg(31);
    let ir1 = cp2.read_data_reg(9);
    let ir2 = cp2.read_data_reg(10);
    let ir3 = cp2.read_data_reg(11);
    let mac1 = cp2.read_data_reg(25);
    let mac2 = cp2.read_data_reg(26);
    let mac3 = cp2.read_data_reg(27);

    // Assert results are correct.
    assert_eq!(irgb, 0x001F);
    assert_eq!(orgb, 0x001F);
    assert_eq!(flag, 0x81C00000_u32 as i32);
    assert_eq!(ir1, 0x7FFF);
    assert_eq!(ir2, 0);
    assert_eq!(ir3, 0);
    assert_eq!(mac1, 0x31829CAA);
    assert_eq!(mac2, 0xAC7C27D2_u32 as i32);
    assert_eq!(mac3, 0xC8DC4459_u32 as i32);
}

#[test]
fn ncds_should_produce_correct_result() {

    let mut cp2 = CP2::new();

    // Setup light matrix.
    cp2.write_control_reg(8, 0x56B3749C, false);
    cp2.write_control_reg(9, 0x564D301E, false);
    cp2.write_control_reg(10, 0x555672AE, false);
    cp2.write_control_reg(11, 0x25DF0EC9, false);
    cp2.write_control_reg(12, 0xB812, false);

    // Setup light colour matrix.
    cp2.write_control_reg(16, 0x9F6D6DE1_u32 as i32, false);
    cp2.write_control_reg(17, 0x6E79C789, false);
    cp2.write_control_reg(18, 0xC333E5E1_u32 as i32, false);
    cp2.write_control_reg(19, 0x7CDC09A8, false);
    cp2.write_control_reg(20, 0xDE76, false);

    // Setup RBK, GBK and BBK.
    cp2.write_control_reg(13, 0xFE67018B_u32 as i32, false);
    cp2.write_control_reg(14, 0xE8A615E0_u32 as i32, false);
    cp2.write_control_reg(15, 0x14CA298A, false);

    // Setup RFC, GFC and BFC.
    cp2.write_control_reg(21, 0xCCFE41E8_u32 as i32, false);
    cp2.write_control_reg(22, 0x67945B2A, false);
    cp2.write_control_reg(23, 0xC13B1BBD_u32 as i32, false);

    // Write IR0.
    cp2.write_data_reg(8, 0x4A25, false);

    // Write RGBC.
    cp2.write_data_reg(6, 0x31A22561, false);

    // Setup VX0, VY0 and VZ0.
    cp2.write_data_reg(0, 0x1D0CE323, false);
    cp2.write_data_reg(1, 0x256A, false);

    // Execute NCDS (with sf bit set to 1 and lm bit set to 1).
    cp2.handle_common_ncd(0x4BE80413, InstructionVariant::Single);

    // Now read registers.
    let rgb2 = cp2.read_data_reg(22);
    let irgb = cp2.read_data_reg(28);
    let orgb = cp2.read_data_reg(29);
    let flag = cp2.read_control_reg(31);
    let ir1 = cp2.read_data_reg(9);
    let ir2 = cp2.read_data_reg(10);
    let ir3 = cp2.read_data_reg(11);
    let mac1 = cp2.read_data_reg(25);
    let mac2 = cp2.read_data_reg(26);
    let mac3 = cp2.read_data_reg(27);

    // Assert results are correct.
    assert_eq!(rgb2, 0x3100FF00);
    assert_eq!(irgb, 0x03E0);
    assert_eq!(orgb, 0x03E0);
    assert_eq!(flag, 0x81F80000_u32 as i32);
    assert_eq!(ir1, 0);
    assert_eq!(ir2, 0x7FFF);
    assert_eq!(ir3, 0);
    assert_eq!(mac1, 0xFFFDAED8_u32 as i32);
    assert_eq!(mac2, 0x00025123);
    assert_eq!(mac3, 0xFFFDFFD7_u32 as i32);
}

#[test]
fn cdp_should_produce_correct_result() {

    let mut cp2 = CP2::new();

    // Setup RBK, GBK and BBK.
    cp2.write_control_reg(13, 0x7D4D7981, false);
    cp2.write_control_reg(14, 0x4EDC826D, false);
    cp2.write_control_reg(15, 0x3E0C38CA, false);

    // Setup RFC, GFC and BFC.
    cp2.write_control_reg(21, 0xD9B39721_u32 as i32, false);
    cp2.write_control_reg(22, 0xECD4B243_u32 as i32, false);
    cp2.write_control_reg(23, 0x110C2255, false);

    // Write IR0, IR1, IR2 and IR3.
    cp2.write_data_reg(8, 0xC384, false);
    cp2.write_data_reg(9, 0x0F69, false);
    cp2.write_data_reg(10, 0xB7AC, false);
    cp2.write_data_reg(11, 0xF692, false);

    // Write RGBC.
    cp2.write_data_reg(6, 0xA511774A_u32 as i32, false);

    // Setup light colour matrix.
    cp2.write_control_reg(16, 0x8EE57615_u32 as i32, false);
    cp2.write_control_reg(17, 0xB8B4B74E_u32 as i32, false);
    cp2.write_control_reg(18, 0x35AC624B, false);
    cp2.write_control_reg(19, 0x8F2872D8_u32 as i32, false);
    cp2.write_control_reg(20, 0x2286, false);

    // Execute CDP (with sf bit set to 1 and lm bit set to 0).
    cp2.handle_cdp(0x4BE80014);

    // Now read registers.
    let rgb2 = cp2.read_data_reg(22);
    let irgb = cp2.read_data_reg(28);
    let orgb = cp2.read_data_reg(29);
    let flag = cp2.read_control_reg(31);
    let ir1 = cp2.read_data_reg(9);
    let ir2 = cp2.read_data_reg(10);
    let ir3 = cp2.read_data_reg(11);
    let mac1 = cp2.read_data_reg(25);
    let mac2 = cp2.read_data_reg(26);
    let mac3 = cp2.read_data_reg(27);

    // Assert results are correct.
    assert_eq!(rgb2, 0xA500FFFF_u32 as i32);
    assert_eq!(irgb, 0x03FF);
    assert_eq!(orgb, 0x03FF);
    assert_eq!(flag, 0x81F80000_u32 as i32);
    assert_eq!(ir1, 0x7FFF);
    assert_eq!(ir2, 0x7FFF);
    assert_eq!(ir3, 0xFFFF8000_u32 as i32);
    assert_eq!(mac1, 0x000208DF);
    assert_eq!(mac2, 0x00021F5F);
    assert_eq!(mac3, 0xFFFE24A3_u32 as i32);
}

#[test]
fn ncdt_should_produce_correct_result() {

    let mut cp2 = CP2::new();

    // Setup light matrix.
    cp2.write_control_reg(8, 0x45F14941, false);
    cp2.write_control_reg(9, 0x287577FE, false);
    cp2.write_control_reg(10, 0x63F3B255, false);
    cp2.write_control_reg(11, 0xB41A47A8_u32 as i32, false);
    cp2.write_control_reg(12, 0xA39D, false);

    // Setup light colour matrix.
    cp2.write_control_reg(16, 0xA2F1D4CD_u32 as i32, false);
    cp2.write_control_reg(17, 0x2CDE7E82, false);
    cp2.write_control_reg(18, 0x68C4537A, false);
    cp2.write_control_reg(19, 0xD19306B0_u32 as i32, false);
    cp2.write_control_reg(20, 0x5F8C, false);

    // Setup RBK, GBK and BBK.
    cp2.write_control_reg(13, 0x7CA6292C, false);
    cp2.write_control_reg(14, 0x9C6B02A5_u32 as i32, false);
    cp2.write_control_reg(15, 0x05E85B0F, false);

    // Setup RFC, GFC and BFC.
    cp2.write_control_reg(21, 0x243DA360, false);
    cp2.write_control_reg(22, 0xC089D527_u32 as i32, false);
    cp2.write_control_reg(23, 0xDA171A5F_u32 as i32, false);

    // Write IR0.
    cp2.write_data_reg(8, 0x249D, false);

    // Write RGBC.
    cp2.write_data_reg(6, 0xB745516E_u32 as i32, false);

    // Setup VXx, VYx and VZx.
    cp2.write_data_reg(0, 0xA59B584C_u32 as i32, false);
    cp2.write_data_reg(1, 0xDDA8, false);
    cp2.write_data_reg(2, 0xDB407286_u32 as i32, false);
    cp2.write_data_reg(3, 0x3ED7, false);
    cp2.write_data_reg(4, 0x72935978, false);
    cp2.write_data_reg(5, 0x1876, false);

    // Execute NCDT (with sf bit set to 0 and lm bit set to 1).
    cp2.handle_common_ncd(0x4BE00416, InstructionVariant::Triple);

    // Now read registers.
    let rgb0 = cp2.read_data_reg(20);
    let rgb1 = cp2.read_data_reg(21);
    let rgb2 = cp2.read_data_reg(22);
    let irgb = cp2.read_data_reg(28);
    let orgb = cp2.read_data_reg(29);
    let flag = cp2.read_control_reg(31);
    let ir1 = cp2.read_data_reg(9);
    let ir2 = cp2.read_data_reg(10);
    let ir3 = cp2.read_data_reg(11);
    let mac1 = cp2.read_data_reg(25);
    let mac2 = cp2.read_data_reg(26);
    let mac3 = cp2.read_data_reg(27);

    // Assert results are correct.
    assert_eq!(rgb0, 0xB7FF0000_u32 as i32);
    assert_eq!(rgb1, 0xB7FF0000_u32 as i32);
    assert_eq!(rgb2, 0xB7FF0000_u32 as i32);
    assert_eq!(irgb, 0x7C00);
    assert_eq!(orgb, 0x7C00);
    assert_eq!(flag, 0x81F80000_u32 as i32);
    assert_eq!(ir1, 0);
    assert_eq!(ir2, 0);
    assert_eq!(ir3, 0x7FFF);
    assert_eq!(mac1, 0xF1217920_u32 as i32);
    assert_eq!(mac2, 0xEDB18000_u32 as i32);
    assert_eq!(mac3, 0x124E5B63);
}

#[test]
fn ncds_with_same_inputs_as_ncdt_should_produce_correct_result() {

    let mut cp2 = CP2::new();

    // Setup light matrix.
    cp2.write_control_reg(8, 0x45F14941, false);
    cp2.write_control_reg(9, 0x287577FE, false);
    cp2.write_control_reg(10, 0x63F3B255, false);
    cp2.write_control_reg(11, 0xB41A47A8_u32 as i32, false);
    cp2.write_control_reg(12, 0xA39D, false);

    // Setup light colour matrix.
    cp2.write_control_reg(16, 0xA2F1D4CD_u32 as i32, false);
    cp2.write_control_reg(17, 0x2CDE7E82, false);
    cp2.write_control_reg(18, 0x68C4537A, false);
    cp2.write_control_reg(19, 0xD19306B0_u32 as i32, false);
    cp2.write_control_reg(20, 0x5F8C, false);

    // Setup RBK, GBK and BBK.
    cp2.write_control_reg(13, 0x7CA6292C, false);
    cp2.write_control_reg(14, 0x9C6B02A5_u32 as i32, false);
    cp2.write_control_reg(15, 0x05E85B0F, false);

    // Setup RFC, GFC and BFC.
    cp2.write_control_reg(21, 0x243DA360, false);
    cp2.write_control_reg(22, 0xC089D527_u32 as i32, false);
    cp2.write_control_reg(23, 0xDA171A5F_u32 as i32, false);

    // Write IR0.
    cp2.write_data_reg(8, 0x249D, false);

    // Write RGBC.
    cp2.write_data_reg(6, 0xB745516E_u32 as i32, false);

    // Setup VXx, VYx and VZx.
    cp2.write_data_reg(0, 0xA59B584C_u32 as i32, false);
    cp2.write_data_reg(1, 0xDDA8, false);
    cp2.write_data_reg(2, 0xDB407286_u32 as i32, false);
    cp2.write_data_reg(3, 0x3ED7, false);
    cp2.write_data_reg(4, 0x72935978, false);
    cp2.write_data_reg(5, 0x1876, false);

    // Execute NCDS (with sf bit set to 0 and lm bit set to 1).
    cp2.handle_common_ncd(0x4BE00413, InstructionVariant::Single);

    // Now read registers.
    let rgb0 = cp2.read_data_reg(20);
    let rgb1 = cp2.read_data_reg(21);
    let rgb2 = cp2.read_data_reg(22);
    let irgb = cp2.read_data_reg(28);
    let orgb = cp2.read_data_reg(29);
    let flag = cp2.read_control_reg(31);
    let ir1 = cp2.read_data_reg(9);
    let ir2 = cp2.read_data_reg(10);
    let ir3 = cp2.read_data_reg(11);
    let mac1 = cp2.read_data_reg(25);
    let mac2 = cp2.read_data_reg(26);
    let mac3 = cp2.read_data_reg(27);

    // Assert results are correct.
    assert_eq!(rgb0, 0);
    assert_eq!(rgb1, 0);
    assert_eq!(rgb2, 0xB7FF0000_u32 as i32);
    assert_eq!(irgb, 0x7C00);
    assert_eq!(orgb, 0x7C00);
    assert_eq!(flag, 0x81F80000_u32 as i32);
    assert_eq!(ir1, 0);
    assert_eq!(ir2, 0);
    assert_eq!(ir3, 0x7FFF);
    assert_eq!(mac1, 0xF1217920_u32 as i32);
    assert_eq!(mac2, 0xF0397AF0_u32 as i32);
    assert_eq!(mac3, 0x124E5B63);
}

#[test]
fn nccs_should_produce_correct_result() {

    let mut cp2 = CP2::new();

    // Setup light matrix.
    cp2.write_control_reg(8, 0x81059853_u32 as i32, false);
    cp2.write_control_reg(9, 0xDD1320C6_u32 as i32, false);
    cp2.write_control_reg(10, 0x115712DC, false);
    cp2.write_control_reg(11, 0x0686DAA6, false);
    cp2.write_control_reg(12, 0x31E3, false);

    // Setup light colour matrix.
    cp2.write_control_reg(16, 0xE71DAD51_u32 as i32, false);
    cp2.write_control_reg(17, 0xEFAFF0CE_u32 as i32, false);
    cp2.write_control_reg(18, 0xCC7B8EB3_u32 as i32, false);
    cp2.write_control_reg(19, 0x2845AE41, false);
    cp2.write_control_reg(20, 0xC8A2, false);

    // Setup RBK, GBK and BBK.
    cp2.write_control_reg(13, 0xB9BFE5E9_u32 as i32, false);
    cp2.write_control_reg(14, 0x8A717DC9_u32 as i32, false);
    cp2.write_control_reg(15, 0xEBEBDD4E_u32 as i32, false);

    // Write RGBC.
    cp2.write_data_reg(6, 0xECCC7792_u32 as i32, false);

    // Write VX0, VY0 and VZ0.
    cp2.write_data_reg(0, 0x7B348CA2, false);
    cp2.write_data_reg(1, 0xEF6B, false);

    // Execute NCCS (with sf bit set to 1 and lm bit set to 1).
    cp2.handle_common_ncc(0x4BE8041B, InstructionVariant::Single);

    // Now read registers.
    let rgb2 = cp2.read_data_reg(22);
    let irgb = cp2.read_data_reg(28);
    let orgb = cp2.read_data_reg(29);
    let flag = cp2.read_control_reg(31);
    let ir1 = cp2.read_data_reg(9);
    let ir2 = cp2.read_data_reg(10);
    let ir3 = cp2.read_data_reg(11);
    let mac1 = cp2.read_data_reg(25);
    let mac2 = cp2.read_data_reg(26);
    let mac3 = cp2.read_data_reg(27);

    // Assert results are correct.
    assert_eq!(rgb2, 0xEC000000_u32 as i32);
    assert_eq!(irgb, 0);
    assert_eq!(orgb, 0);
    assert_eq!(flag, 0x81C00000_u32 as i32);
    assert_eq!(ir1, 0);
    assert_eq!(ir2, 0);
    assert_eq!(ir3, 0);
    assert_eq!(mac1, 0);
    assert_eq!(mac2, 0);
    assert_eq!(mac3, 0);
}

#[test]
fn cc_should_produce_correct_result() {

    let mut cp2 = CP2::new();

    // Setup RBK, GBK and BBK.
    cp2.write_control_reg(13, 0x1E49F5FA, false);
    cp2.write_control_reg(14, 0x05BA5761, false);
    cp2.write_control_reg(15, 0x3FB42B2E, false);

    // Setup IR1, IR2 and IR3.
    cp2.write_data_reg(9, 0x93E7, false);
    cp2.write_data_reg(10, 0x8F63, false);
    cp2.write_data_reg(11, 0xFC8B, false);

    // Setup RGBC.
    cp2.write_data_reg(6, 0xB4258F0A_u32 as i32, false);

    // Setup light colour matrix.
    cp2.write_control_reg(16, 0xAC56AF73_u32 as i32, false);
    cp2.write_control_reg(17, 0xDD8B2ADF_u32 as i32, false);
    cp2.write_control_reg(18, 0xD4971805_u32 as i32, false);
    cp2.write_control_reg(19, 0xE8C068C7_u32 as i32, false);
    cp2.write_control_reg(20, 0x5ADE, false);

    // Execute CC (with sf bit set to 1 and lm bit set to 0).
    cp2.handle_cc(0x4BE8001C);

    // Now read registers.
    let rgb2 = cp2.read_data_reg(22);
    let irgb = cp2.read_data_reg(28);
    let orgb = cp2.read_data_reg(29);
    let flag = cp2.read_control_reg(31);
    let ir1 = cp2.read_data_reg(9);
    let ir2 = cp2.read_data_reg(10);
    let ir3 = cp2.read_data_reg(11);
    let mac1 = cp2.read_data_reg(25);
    let mac2 = cp2.read_data_reg(26);
    let mac3 = cp2.read_data_reg(27);

    // Assert results are correct.
    assert_eq!(rgb2, 0xB4FFFF4F_u32 as i32);
    assert_eq!(irgb, 0x7FE9);
    assert_eq!(orgb, 0x7FE9);
    assert_eq!(flag, 0x81D80000_u32 as i32);
    assert_eq!(ir1, 0x04FF);
    assert_eq!(ir2, 0x477F);
    assert_eq!(ir3, 0x127F);
    assert_eq!(mac1, 0x000004FF);
    assert_eq!(mac2, 0x0000477F);
    assert_eq!(mac3, 0x0000127F);
}

#[test]
fn ncs_should_produce_correct_result() {

    let mut cp2 = CP2::new();

    // Setup light matrix.
    cp2.write_control_reg(8, 0xCA097274_u32 as i32, false);
    cp2.write_control_reg(9, 0x5A0B3305, false);
    cp2.write_control_reg(10, 0x3047377D, false);
    cp2.write_control_reg(11, 0xB45166D9_u32 as i32, false);
    cp2.write_control_reg(12, 0x4507, false);

    // Setup light colour matrix.
    cp2.write_control_reg(16, 0xF08223B6_u32 as i32, false);
    cp2.write_control_reg(17, 0xA29269CA_u32 as i32, false);
    cp2.write_control_reg(18, 0x361E800A, false);
    cp2.write_control_reg(19, 0x70110788, false);
    cp2.write_control_reg(20, 0x8D76, false);

    // Setup RBK, GBK and BBK.
    cp2.write_control_reg(13, 0xCC31624D_u32 as i32, false);
    cp2.write_control_reg(14, 0xD1406D34_u32 as i32, false);
    cp2.write_control_reg(15, 0x88A04499_u32 as i32, false);

    // Setup CODE.
    cp2.write_data_reg(6, 0x96000000_u32 as i32, false);

    // Setup VX0, VY0 and VZ0.
    cp2.write_data_reg(0, 0xAF7564FB_u32 as i32, false);
    cp2.write_data_reg(1, 0x814D, false);

    // Execute NCS (with sf bit set to 1 and lm bit set to 0).
    cp2.handle_common_nc(0x4BE8001E, InstructionVariant::Single);

    // Now read registers.
    let rgb2 = cp2.read_data_reg(22);
    let irgb = cp2.read_data_reg(28);
    let orgb = cp2.read_data_reg(29);
    let flag = cp2.read_control_reg(31);
    let ir1 = cp2.read_data_reg(9);
    let ir2 = cp2.read_data_reg(10);
    let ir3 = cp2.read_data_reg(11);
    let mac1 = cp2.read_data_reg(25);
    let mac2 = cp2.read_data_reg(26);
    let mac3 = cp2.read_data_reg(27);

    // Assert results are correct.
    assert_eq!(rgb2, 0x96000000_u32 as i32);
    assert_eq!(irgb, 0);
    assert_eq!(orgb, 0);
    assert_eq!(flag, 0x81F80000_u32 as i32);
    assert_eq!(ir1, 0xFFFF8000_u32 as i32);
    assert_eq!(ir2, 0xFFFF8000_u32 as i32);
    assert_eq!(ir3, 0xFFFF8000_u32 as i32);
    assert_eq!(mac1, 0xCC3628A2_u32 as i32);
    assert_eq!(mac2, 0xD1421D24_u32 as i32);
    assert_eq!(mac3, 0x889A5ED9_u32 as i32);
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
    cp2.handle_sqr(0x4BE00028);

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