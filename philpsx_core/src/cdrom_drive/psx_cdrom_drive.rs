// SPDX-License-Identifier: GPL-3.0
// psx_cdrom_drive.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

/// This module contains an implementation of the CD-ROM disc format itself, specific to
/// the primary PsxCdromDrive implementation below.
mod psx_bin_cue_cd;

/// This module contains a stub implementation, intended for use when the drive is empty.
mod empty_cd;

use empty_cd::EmptyCd;
use psx_bin_cue_cd::PsxBinCueCd;
use super::{CdromDrive, CdromDriveBridge};
use std::{
    error::Error,
    ffi::OsStr,
};

/// This trait provides a way of supporting different types of CD image in such a way
/// that the rest of the system doesn't have to care about the details.
trait Cdrom {

    /// Implementations must use this to signal whether the drive is loaded with a disc.
    fn is_loaded(&self) -> bool;

    /// Implementations must read a byte from the specified real CD address
    /// using the supplied address.
    fn read_byte(
        &mut self,
        address: usize,
    ) -> Result<u8, Box<dyn Error>>;
}

/// This struct models the CD-ROM drive of the PlayStation.
pub struct PsxCdromDrive {

    // This controls what we are reading/writing.
    port_index: u8,

    // This stores parameters for commands.
    parameter_fifo: [u8; 16],
    parameter_count: i32,

    // This stores command responses.
    response_fifo: [u8; 16],
    response_count: i32,
    response_index: i32,

    // This stores data from the CD.
    data_fifo: Vec<u8>,
    data_count: i32,
    data_index: i32,

    // This references the actual CD.
    cd: Box<dyn Cdrom>,

    // Interrupt registers.
    interrupt_enable_register: i32,
    interrupt_flag_register: i32,

    // Busy flag and current command.
    busy: bool,
    current_command: i32,
    needs_second_response: bool,

    // These flags allow the composition of a status byte.
    cdda_playing: bool,
    is_seeking: bool,
    is_reading: bool,
    shell_open: bool,
    id_error: bool,
    seek_error: bool,
    motor_status: bool,
    command_error: bool,

    // These flags control behaviour and are set via the Setmode command.
    double_speed: bool,
    xa_adpcm: bool,
    whole_sector: bool,
    ignore_bit: bool,
    xa_filter: bool,
    enable_report_interrupts: bool,
    auto_pause: bool,
    allow_cdda_read: bool,

    // This tells us if a response has been received.
    response_received: i32,

    // This stores the setloc position as a byte index, and whether
    // this sector has been read.
    setloc_position: i64,
    setloc_processed: bool,

    // This handles the read retry in ReadN.
    been_read: bool,
}

/// Implementation functions for the CD-ROM drive itself.
impl PsxCdromDrive {

    /// Creates a new CD-ROM drive object with the correct initial state.
    pub fn new() -> Self {
        PsxCdromDrive {

            // Set port index to 0.
            port_index: 0,

            // Setup parameter FIFO and count.
            parameter_fifo: [0; 16],
            parameter_count: 0,

            // Setup response FIFO and related variables.
            response_fifo: [0; 16],
            response_count: 0,
            response_index: 0,

            // Setup data FIFO and related variables.
            data_fifo: vec![0; 0x924],
            data_count: 0,
            data_index: 0,

            // Setup CD object itself.
            cd: Box::new(EmptyCd::new()),

            // Setup interrupt registers.
            interrupt_enable_register: 0,
            interrupt_flag_register: 0,

            // Set status to non-busy and current command to 0, as well as
            // second response flag.
            busy: false,
            current_command: 0,
            needs_second_response: false,

            // Setup status byte flags.
            cdda_playing: false,
            is_seeking: false,
            is_reading: false,
            shell_open: false,
            id_error: false,
            seek_error: false,
            motor_status: false,
            command_error: false,

            // Setup mode flags.
            double_speed: false,
            xa_adpcm: false,
            whole_sector: false,
            ignore_bit: false,
            xa_filter: false,
            enable_report_interrupts: false,
            auto_pause: false,
            allow_cdda_read: false,

            // Setup response received.
            response_received: 0,

            // Setup position.
            setloc_position: 0,
            setloc_processed: false,

            // Handle read retry in ReadN command.
            been_read: true,
        }
    }

    fn get_interrupt_enable_register(&self) -> u8 {
        (self.interrupt_enable_register | 0xE0) as u8
    }

    fn get_interrupt_flag_register(&self) -> u8 {
        (0xE0 | (self.interrupt_flag_register & 0x7)) as u8
    }
}

/// Implementation functions to be called from anything that understands what
/// a CdromDrive object is.
impl CdromDrive for PsxCdromDrive {

    /// This function reads chunks at a time from the data fifo into the supplied buffer.
    fn chunk_copy(
        &mut self,
        mut destination: &mut [u8],
        start_index: i32,
        mut length: i32
    ) {

        // Setup the destination buffer and data fifo to the correct offsets.
        let temp_data_fifo = &mut self.data_fifo[(self.data_index as usize)..];
        destination = &mut destination[(start_index as usize)..];

        // Check if length would bring us over the data fifo bounds.
        if self.data_index + length > self.data_count {

            // Copy as much as we can.
            let copyable_amount = (self.data_count - self.data_index) as usize;
            destination[..copyable_amount].copy_from_slice(&temp_data_fifo[..copyable_amount]);

            // Now copy the rest as specified.
            destination = &mut destination[copyable_amount..];
            length -= copyable_amount as i32;
            self.data_index = self.data_count;
            let fill_value = if self.whole_sector {
                self.data_fifo[0x920]
            } else {
                self.data_fifo[0x7F8]
            };
            destination[..(length as usize)].fill(fill_value);
        }
        // We are fine, just do the copy.
        else {
            destination[..(length as usize)].copy_from_slice(&temp_data_fifo[..(length as usize)]);
            self.data_index += length;
        }

        self.been_read = true;
    }

    /// Load the CD from the image file referenced by the supplied path.
    fn load_cd(
        &mut self,
        path: &OsStr,
    ) -> Result<(), Box<dyn Error>> {

        // For now, just assume bin/cue as we don't support anything else.
        self.cd = Box::new(PsxBinCueCd::new(path)?);
        Ok(())
    }

    /// This function reads a byte from the index/status register.
    fn read_1800(&self) -> u8 {

        let mut ret_val = self.port_index & 0x3;

        let bit3 = if self.parameter_count == 0 { 1u8 } else { 0 };
        let bit4 = if self.parameter_count == 16 { 0u8 } else { 1 };
        let bit5 = if self.response_count == 0 { 0u8 } else { 1 };
        let bit6 = if self.data_count == 0 { 0u8 } else { 1 };

        ret_val |= bit3 << 3;
        ret_val |= bit4 << 4;
        ret_val |= bit5 << 5;
        ret_val |= bit6 << 6;
        ret_val |= if self.busy { 1u8 } else { 0 } << 7;

        ret_val
    }

    /// This function reads a byte from port 0x1F801801.
    fn read_1801(&mut self) -> u8 {

        // Return fifo byte, regardless of port index, as this is
        // mirrored on all four ports.
        let ret_val = self.response_fifo[self.response_index as usize];
        self.response_index += 1;

        if self.response_index == self.response_count {
            self.response_count = 0;
        }
        if self.response_index > 15 {
            self.response_index = 0;
        }

        ret_val
    }

    /// This function reads a byte from port 0x1F801802.
    fn read_1802(&mut self) -> u8 {

        // All port indexes to this port read from data fifo.
        let ret_val = if self.data_index < self.data_count {
            let temp = self.data_fifo[self.data_index as usize];
            self.data_index += 1;
            temp
        }
        else {
            if self.whole_sector {
                self.data_fifo[0x920]
            } else {
                self.data_fifo[0x7F8]
            }
        };

        self.been_read = true;

        ret_val
    }

    /// This function reads a byte from port 0x1F801803.
    fn read_1803(&self) -> u8 {

        // Act depending on port index.
        match self.port_index {
            0 | 2 => self.get_interrupt_enable_register(),

            1 | 3 => self.get_interrupt_flag_register(),

            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests;
