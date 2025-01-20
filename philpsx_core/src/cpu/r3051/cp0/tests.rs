use super::CP0;

#[test]
fn status_register_read_should_work() {

    let mut cp0 = CP0::new();
    cp0.cop_registers[12] = 0xFFFFFFFF_u32 as i32;
    let output = cp0.read_reg(12);

    assert_eq!(output, 0xF27FFF3F_u32 as i32);
}

#[test]
fn cause_register_read_should_work() {

    let mut cp0 = CP0::new();
    cp0.cop_registers[13] = 0xFFFFFFFF_u32 as i32;
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
    cp0.cop_registers[1] = 1;
    cp0.cop_registers[8] = 8;
    cp0.cop_registers[14] = 14;
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
    cp0.cop_registers[2] = 1;
    let output = cp0.read_reg(2);

    assert_eq!(output, 0);
}

#[test]
fn status_and_cause_registers_write_with_override() {

    let mut cp0 = CP0::new();
    cp0.cop_registers[12] = 0xFFFFFFFF_u32 as i32;
    cp0.cop_registers[13] = 0xFFFFFFFF_u32 as i32;
    cp0.write_reg(12, 0, true);
    cp0.write_reg(13, 0, true);
    let output1 = cp0.cop_registers[12];
    let output2 = cp0.cop_registers[13];

    assert_eq!(output1, 0);
    assert_eq!(output2, 0);
}

#[test]
fn status_and_cause_registers_write_without_override() {

    let mut cp0 = CP0::new();
    cp0.cop_registers[12] = 0xFFFFFFFF_u32 as i32;
    cp0.cop_registers[13] = 0xFFFFFFFF_u32 as i32;
    cp0.write_reg(12, 0, false);
    cp0.write_reg(13, 0, false);
    let output1 = cp0.cop_registers[12];
    let output2 = cp0.cop_registers[13];

    assert_eq!(output1, 0x0DB400C0);
    assert_eq!(output2, 0xFFFFFCFF_u32 as i32);
}

#[test]
fn status_and_cause_registers_write_merge() {

    let mut cp0 = CP0::new();
    cp0.cop_registers[12] = 0x0DB400C0;
    cp0.cop_registers[13] = 0xFFFFFCFF_u32 as i32;
    cp0.write_reg(12, 0xF24BFF3F_u32 as i32, false);
    cp0.write_reg(13, 0x00000300, false);
    let output1 = cp0.cop_registers[12];
    let output2 = cp0.cop_registers[13];

    assert_eq!(output1, 0xFFFFFFFF_u32 as i32);
    assert_eq!(output2, 0xFFFFFFFF_u32 as i32);
}

#[test]
fn status_and_cause_registers_write_only_writable_bits() {

    let mut cp0 = CP0::new();
    cp0.cop_registers[12] = 0x00000000;
    cp0.cop_registers[13] = 0x00000000;
    cp0.write_reg(12, 0xFF000000_u32 as i32, false);
    cp0.write_reg(13, 0x0000FF00, false);
    let output1 = cp0.cop_registers[12];
    let output2 = cp0.cop_registers[13];

    assert_eq!(output1, 0xF2000000_u32 as i32);
    assert_eq!(output2, 0x00000300);
}

#[test]
fn arbitrary_register_write_without_override() {

    let mut cp0 = CP0::new();
    cp0.cop_registers[1] = 0x00000000;
    cp0.write_reg(1, 0xFFFFFFFF_u32 as i32, false);
    let output = cp0.cop_registers[1];

    assert_eq!(output, 0xFFFFFFFF_u32 as i32);
}