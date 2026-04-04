// SPDX-License-Identifier: GPL-3.0
// tests.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

// Tests for the PsxCdromDrive implementation.

use super::{
    PsxCdromDrive,
    super::CdromDrive,
};

#[test]
fn test_chunk_copy_length_small_enough_success() {

    // Given a PsxCdromDrive instance and a destination buffer
    // with the correct initial state.
    let mut cd_drive = PsxCdromDrive::new();
    let mut destination_buffer = vec![0u8; 5];
    cd_drive.data_count = 8;
    cd_drive.data_index = 2;
    cd_drive.data_fifo[2] = 0x1;
    cd_drive.data_fifo[3] = 0x2;
    cd_drive.data_fifo[4] = 0x3;
    cd_drive.data_fifo[5] = 0x5;
    cd_drive.data_fifo[6] = 0x6;
    cd_drive.data_fifo[7] = 0x7;

    // When we execute chunk_copy.
    cd_drive.chunk_copy(&mut destination_buffer, 1, 3);

    // Then the destination buffer should contain the copied bytes
    // in the right place.
    assert_eq!(destination_buffer, vec![0, 0x1, 0x2, 0x3, 0]);
}

#[test]
fn test_chunk_copy_length_bigger_than_remaining_fifo_items_whole_sector_success() {

    // Given a PsxCdromDrive instance and a destination buffer
    // with the correct initial state.
    let mut cd_drive = PsxCdromDrive::new();
    let mut destination_buffer = vec![0u8; 10];
    cd_drive.data_count = 5;
    cd_drive.data_index = 2;
    cd_drive.data_fifo[2] = 0x1;
    cd_drive.data_fifo[3] = 0x2;
    cd_drive.data_fifo[4] = 0x3;
    cd_drive.data_fifo[0x920] = 0x87;
    cd_drive.whole_sector = true;

    // When we execute chunk_copy.
    cd_drive.chunk_copy(&mut destination_buffer, 1, 9);

    // Then the destination buffer should contain the copied bytes
    // in the right place.
    assert_eq!(destination_buffer, vec![0, 0x1, 0x2, 0x3, 0x87, 0x87, 0x87, 0x87, 0x87, 0x87]);
}

#[test]
fn test_chunk_copy_length_bigger_than_remaining_fifo_items_not_whole_sector_success() {

    // Given a PsxCdromDrive instance and a destination buffer
    // with the correct initial state.
    let mut cd_drive = PsxCdromDrive::new();
    let mut destination_buffer = vec![0u8; 10];
    cd_drive.data_count = 5;
    cd_drive.data_index = 2;
    cd_drive.data_fifo[2] = 0x1;
    cd_drive.data_fifo[3] = 0x2;
    cd_drive.data_fifo[4] = 0x3;
    cd_drive.data_fifo[0x7F8] = 0x23;
    cd_drive.whole_sector = false;

    // When we execute chunk_copy.
    cd_drive.chunk_copy(&mut destination_buffer, 1, 9);

    // Then the destination buffer should contain the copied bytes
    // in the right place.
    assert_eq!(destination_buffer, vec![0, 0x1, 0x2, 0x3, 0x23, 0x23, 0x23, 0x23, 0x23, 0x23]);
}