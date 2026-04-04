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
    port_index: i32,

    // This stores parameters for commands.
    parameter_fifo: [i8; 16],
    parameter_count: i32,

    // This stores command responses.
    response_fifo: [i8; 16],
    response_count: i32,
    response_index: i32,

    // This stores data from the CD.
    data_fifo: Vec<i8>,
    data_count: i32,
    data_index: i32,

    // This references the actual CD.
    cd: Box<dyn Cdrom>,

    // Interrupt registers.
    interrupt_enable_register: i32,
    interrupt_flag_register: i32,

    // Busy flag and current command.
    busy: i32,
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
            busy: 0,
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
}

/// Implementation functions to be called from anything that understands what
/// a CdromDrive object is.
impl CdromDrive for PsxCdromDrive {

    /// Load the CD from the image file referenced by the supplied path.
    fn load_cd(
        &mut self,
        path: &OsStr,
    ) -> Result<(), Box<dyn Error>> {
        self.cd = Box::new(PsxBinCueCd::new(path)?);
        Ok(())
    }
}