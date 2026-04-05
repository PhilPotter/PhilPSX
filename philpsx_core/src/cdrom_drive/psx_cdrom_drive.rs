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
    interrupt_enable_register: u8,
    interrupt_flag_register: u8,

    // Busy flag and current command.
    busy: bool,
    current_command: u8,
    needs_second_response: bool,

    // These flags allow the composition of a status byte.
    cdda_playing: bool,
    is_seeking: bool,
    is_reading: bool,
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

    /// Retrieve the interrupt enable register.
    fn get_interrupt_enable_register(&self) -> u8 {
        0xE0 | self.interrupt_enable_register
    }

    /// Retrieve the interrupt flag register.
    fn get_interrupt_flag_register(&self) -> u8 {
        0xE0 | (self.interrupt_flag_register & 0x7)
    }

    /// This function clears the response fifo.
    fn clear_response_fifo(&mut self) {
        
        // Set to all zeroes and reset count and index too.
        self.response_fifo.fill(0);
        self.response_count = 0;
        self.response_index = 0;
    }

    /// This function clears the parameter fifo.
    fn clear_parameter_fifo(&mut self) {

        // Set to all zeroes and reset count too.
        self.parameter_fifo.fill(0);
        self.parameter_count = 0;
    }

    /// This function clears the data fifo.
    fn clear_data_fifo(&mut self) {

        // Set to all zeroes and reset count and index too.
        self.data_fifo.fill(0);
        self.data_count = 0;
        self.data_index = 0;
    }

    /// This function handles the Getstat command.
    fn command_getstat(
        &mut self,
        bridge: &mut dyn CdromDriveBridge
    ) {

        // Store state byte to response fifo.
        self.response_fifo[self.response_count as usize] = self.get_status_code();
        self.response_count += 1;
        self.busy = false;
        self.response_received = 3;
        self.trigger_interrupt(bridge, 3, 16000);
    }

    /// This function handles the Setloc command.
    fn command_setloc(
        &mut self,
        bridge: &mut dyn CdromDriveBridge
    ) {

        // Get location from parameters.
        let mut minutes = (self.parameter_fifo[0] as i64) & 0xFF;
        let mut seconds = (self.parameter_fifo[1] as i64) & 0xFF;
        let mut frames = (self.parameter_fifo[2] as i64) & 0xFF;

        // Convert from BCD to actual numbers.
        minutes = (minutes & 0xF) + (((minutes >> 4) & 0xF) * 10);
        seconds = (seconds & 0xF) + (((seconds >> 4) & 0xF) * 10);
        frames = (frames & 0xF) + (((frames >> 4) & 0xF) * 10);

        // Get byte position of the above.
        self.setloc_position =
            (frames * 2352) + (seconds * 176400) + (minutes * 10584000);
        self.setloc_processed = false;

        // Deal with response code etc.
        self.response_fifo[self.response_count as usize] = self.get_status_code();
        self.response_count += 1;
        self.busy = false;
        self.response_received = 3;
        self.trigger_interrupt(bridge, 3, 16000);
    }

    /// This function handles the ReadN command.
    fn command_readn(
        &mut self,
        bridge: &mut dyn CdromDriveBridge,
        second_response: bool
    ) {

        // Determine what to do.
        // First response, just send state byte.
        if !second_response {
            self.response_fifo[self.response_count as usize] = self.get_status_code();
            self.response_count += 1;
            self.is_reading = true;
            self.needs_second_response = true;
            self.been_read = true;
            self.response_received = 3;
            self.trigger_interrupt(bridge, 3, 16000);
        }
        // Second response, read sector into fifo and send stat byte.
        else {
            if self.been_read {
                self.clear_data_fifo();

                // Modify Setloc position if needed.
                if self.setloc_processed {
                    self.setloc_position += 2352;
                } else {
                    self.setloc_processed = true;
                }

                let mut start_address = self.setloc_position;
                start_address += if self.whole_sector { 12 } else { 24 };
                let sector_size = if self.whole_sector { 0x924 } else { 0x800 };

                for i in 0..sector_size {
                    let byte = match self.cd.read_byte(start_address as usize) {
                        Ok(byte) => byte,

                        // Panic for now, passing this up is not really worthit,
                        // as not much we can do about it anyway. Perhaps worth
                        // reconsidering this later when I actually optimise/
                        // make this fast.
                        Err(error) => panic!(
                            "CD-ROM Drive: Unable to read byte from CD at {:#8X}: {}", start_address, error
                        ),
                    };

                    self.data_fifo[self.data_count as usize] = byte;
                    self.data_count += 1;
                    start_address += 1;
                }

                self.been_read = false;
            }

            // Send stat byte.
            self.response_fifo[self.response_count as usize] = self.get_status_code();
            self.response_count += 1;

            // TODO: This should probably be false - leaving it this way for now as that's how
            // my original version from 10 years ago does it, and I can go back and figure out
            // why later on.
            self.needs_second_response = true;
            self.response_received = 0;
            self.trigger_interrupt(bridge, 1, 16000);
        }
    }

    /// This function handles the Pause command.
    fn command_pause(
        &mut self,
        bridge: &mut dyn CdromDriveBridge,
        second_response: bool
    ) {

        // Determine what to do.
        // First response, send stat byte.
        if !second_response {
            self.response_fifo[self.response_count as usize] = self.get_status_code();
            self.response_count += 1;
            self.needs_second_response = true;
            self.response_received = 3;
            self.trigger_interrupt(bridge, 3, 16000);
        }
        // Second response, send stat byte.
        else {
            self.is_reading = false;
            self.cdda_playing = false;
            self.response_fifo[self.response_count as usize] = self.get_status_code();
            self.response_count += 1;
            self.busy = true;
            self.needs_second_response = false;
            self.response_received = 2;
            self.trigger_interrupt(bridge, 2, 16000);
        }
    }

    /// This function handles the Init command.
    fn command_init(
        &mut self,
        bridge: &mut dyn CdromDriveBridge,
        second_response: bool
    ) {

        // Determine what to do.
        // First response, send stat byte.
        if !second_response {

            // Nuke all mode bits.
            self.double_speed = false;
            self.xa_adpcm = false;
            self.whole_sector = false;
            self.ignore_bit = false;
            self.xa_filter = false;
            self.enable_report_interrupts = false;
            self.auto_pause = false;
            self.allow_cdda_read = false;

            // Store status byte to response fifo.
            self.response_fifo[self.response_count as usize] = self.get_status_code();
            self.response_count += 1;
            self.response_received = 3;
            self.trigger_interrupt(bridge, 3, 16000);
            self.needs_second_response = true;
        }
        // Second response, send stat byte.
        else {

            // Store status byte to response fifo.
            self.response_fifo[self.response_count as usize] = self.get_status_code();
            self.response_count += 1;
            self.response_received = 2;
            self.trigger_interrupt(bridge, 2, 16000);
            self.needs_second_response = false;
            self.busy = false;
        }
    }

    /// This function handles the Demute command.
    fn command_demute(
        &mut self,
        bridge: &mut dyn CdromDriveBridge
    ) {

        // Do nothing other than return status at the moment.

        // Store status byte to response fifo.
        self.response_fifo[self.response_count as usize] = self.get_status_code();
        self.response_count += 1;
        self.busy = false;
        self.response_received = 3;
        self.trigger_interrupt(bridge, 3, 16000);
    }

    /// This function handles the Setmode command.
    fn command_setmode(
        &mut self,
        bridge: &mut dyn CdromDriveBridge
    ) {

        // Set flags using parameter byte.
        let mode_flags = self.parameter_fifo[0];

        self.double_speed = (mode_flags & 0x80) != 0;
        self.xa_adpcm = (mode_flags & 0x40) != 0;
        self.whole_sector = (mode_flags & 0x20) != 0;
        self.ignore_bit = (mode_flags & 0x10) != 0;
        self.xa_filter = (mode_flags & 0x8) != 0;
        self.enable_report_interrupts = (mode_flags & 0x4) != 0;
        self.auto_pause = (mode_flags & 0x2) != 0;
        self.allow_cdda_read = (mode_flags & 0x1) != 0;

        // Store status byte to response fifo.
        self.response_fifo[self.response_count as usize] = self.get_status_code();
        self.response_count += 1;
        self.busy = false;
        self.response_received = 3;
        self.trigger_interrupt(bridge, 3, 16000);
    }

    /// This function handles the SeekL command.
    fn command_seekl(
        &mut self,
        bridge: &mut dyn CdromDriveBridge,
        second_response: bool
    ) {

        // Determine what to do.
        // First response, send stat byte.
        if !second_response {
            self.response_fifo[self.response_count as usize] = self.get_status_code();
            self.response_count += 1;
            self.is_seeking = true;
            self.needs_second_response = true;
            self.response_received = 3;
            self.trigger_interrupt(bridge, 3, 16000);
        }
        // Second response, send stat byte.
        else {
            self.response_fifo[self.response_count as usize] = self.get_status_code();
            self.response_count += 1;
            self.is_seeking = false;
            self.needs_second_response = false;
            self.response_received = 2;
            self.trigger_interrupt(bridge, 2, 16000);
            self.busy = false;
        }
    }

    /// This function handles the Test command.
    fn command_test(
        &mut self,
        bridge: &mut dyn CdromDriveBridge
    ) {

        // Get parameter from fifo.
        let parameter1 = self.parameter_fifo[0];
        self.clear_parameter_fifo();

        match parameter1 {

            // Get the date (y/m/d) in BCD and also the version of
            // the CD-ROM controller BIOS. Use the fake version of
            // PSX/PSone (PU-23, PM-41).
            0x20 => {
                self.response_fifo[self.response_count as usize] = 0x99; // (1999).
                self.response_count += 1;
                self.response_fifo[self.response_count as usize] = 0x02; // (February).
                self.response_count += 1;
                self.response_fifo[self.response_count as usize] = 0x01; // (1st).
                self.response_count += 1;
                self.response_fifo[self.response_count as usize] = 0xC3; // (version vC3).
                self.response_count += 1;
                self.busy = false;
                self.response_received = 3;
                self.trigger_interrupt(bridge, 3, 16000);
            },

            _ => (),
        };
    }

    /// This function handles the GetID command.
    fn command_getid(
        &mut self,
        bridge: &mut dyn CdromDriveBridge,
        second_response: bool
    ) {

        // Determine what to do.
        // First response, send stat byte.
        if !second_response {
            self.response_fifo[self.response_count as usize] = self.get_status_code();
            self.response_count += 1;
            self.needs_second_response = true;
            self.response_received = 3;
            self.trigger_interrupt(bridge, 3, 16000);
        }
        // Second response, send licensed mode 2 response.
        else {
            self.response_fifo[self.response_count as usize] = 0x00;
            self.response_count += 1;
		    self.response_fifo[self.response_count as usize] = 0x20;
            self.response_count += 1;
            self.response_fifo[self.response_count as usize] = 0x02;
            self.response_count += 1;
		    self.response_fifo[self.response_count as usize] = 0x00;
            self.response_count += 1;
		    self.response_fifo[self.response_count as usize] = 0x53;
            self.response_count += 1;
		    self.response_fifo[self.response_count as usize] = 0x43;
            self.response_count += 1;
		    self.response_fifo[self.response_count as usize] = 0x45;
            self.response_count += 1;
		    self.response_fifo[self.response_count as usize] = 0x45;
            self.response_count += 1;
            self.busy = false;
            self.needs_second_response = false;
            self.response_received = 2;
            self.trigger_interrupt(bridge, 2, 16000);
        }
    }

    /// This handles the ReadTOC command.
    fn command_readtoc(
        &mut self,
        bridge: &mut dyn CdromDriveBridge,
        second_response: bool
    ) {

        // Determine what to do.
        // First response, send stat byte.
        if !second_response {
            self.response_fifo[self.response_count as usize] = self.get_status_code();
            self.response_count += 1;
            self.needs_second_response = true;
            self.response_received = 3;
            self.trigger_interrupt(bridge, 3, 16000);
        }
        // Second response, send stat byte.
        else {
            self.response_fifo[self.response_count as usize] = self.get_status_code();
            self.response_count += 1;
            self.response_received = 2;
            self.trigger_interrupt(bridge, 2, 16000);
            self.needs_second_response = false;
            self.busy = false;
        }
    }

    /// This function returns the status code.
    fn get_status_code(&self) -> u8 {

        // Declare return value.
        let mut ret_val = 0;

        // Test each bit.
        if self.cdda_playing {
            ret_val |= 0x80;
        }
        if self.is_seeking {
            ret_val |= 0x40;
        }
        if self.is_reading {
            ret_val |= 0x20;
        }
        if !self.cd.is_loaded() {
            ret_val |= 0x10;
        }
        if self.id_error {
            ret_val |= 0x8;
        }
        if self.seek_error {
            ret_val |= 0x4;
        }
        if self.motor_status {
            ret_val |= 0x2;
        }
        if self.command_error {
            ret_val |= 0x1;
        }

        ret_val
    }

    /// This function triggers a CD-ROM interrupt.
    fn trigger_interrupt(
        &mut self,
        bridge: &mut dyn CdromDriveBridge,
        interrupt_num: u8,
        delay: i32
    ) {

        self.clear_parameter_fifo();

        if interrupt_num != 0 && (self.interrupt_enable_register & interrupt_num) == interrupt_num {
            bridge.set_cdrom_interrupt_enabled(self, true);
        }
        else {
            bridge.set_cdrom_interrupt_enabled(self, false);
        }

        // Set interrupt delay and number.
        bridge.set_cdrom_interrupt_delay(self, delay);
        bridge.set_cdrom_interrupt_number(self, interrupt_num);
    }

    /// This function executes a CD-ROM command.
    fn execute_command(
        &mut self,
        bridge: &mut dyn CdromDriveBridge,
        command_num: u8,
        second_response: bool
    ) {

        // Execute command or deal with second response.
        if !second_response {
            match command_num {

                // Getstat.
                0x01 => self.command_getstat(bridge),

                // Setloc.
                0x02 => self.command_setloc(bridge),

                // ReadN.
                0x06 => self.command_readn(bridge, second_response),

                // Pause.
                0x09 => self.command_pause(bridge, second_response),

                // Init.
                0x0A => self.command_init(bridge, second_response),

                // Demute.
                0x0C => self.command_demute(bridge),

                // Setmode.
                0x0E => self.command_setmode(bridge),

                // SeekL.
                0x15 => self.command_seekl(bridge, second_response),

                // Test.
                0x19 => self.command_test(bridge),

                // GetID.
                0x1A => self.command_getid(bridge, second_response),

                // ReadTOC.
                0x1E => self.command_readtoc(bridge, second_response),

                // Unimplemented command.
                _ => log::error!("CD-ROM Drive: Unimplemented command: {:#02X}", command_num),
            };
        }
        else {
            match command_num {

                // ReadN.
                0x06 => self.command_readn(bridge, second_response),

                // Pause.
                0x09 => self.command_pause(bridge, second_response),

                // Init.
                0x0A => self.command_init(bridge, second_response),

                // SeekL.
                0x15 => self.command_seekl(bridge, second_response),

                // GetID.
                0x1A => self.command_getid(bridge, second_response),

                // ReadTOC.
                0x1E => self.command_readtoc(bridge, second_response),

                _ => (),
            };
        }
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

    /// This function lets us set the interrupt flag register contents manually.
    fn set_interrupt_number(&mut self, interrupt_num: u8) {
        self.interrupt_flag_register = interrupt_num;
    }

    /// This function writes a byte to the index/status register.
    fn write_1800(&mut self, value: u8) {
        self.port_index = value & 0x3;
    }

    /// This function writes a byte to port 0x1F801801.
    fn write_1801(
        &mut self,
        bridge: &mut dyn CdromDriveBridge,
        value: u8
    ) {

        // Act depending on port index.
        if self.port_index == 0 {

            // Execute command byte.
            if self.busy {
                self.clear_response_fifo();
                self.busy = true;
                self.current_command = value;
                self.execute_command(bridge, value, self.needs_second_response);
            }
            else if value == 0x9 {
                self.current_command = value;
                self.needs_second_response = false;
                self.execute_command(bridge, value, self.needs_second_response);
            }
        }
    }

    /// This function writes a byte to port 0x1F801802.
    fn write_1802(&mut self, value: u8) {

        // Act depending on port index.
        match self.port_index {

            // Add byte to parameter fifo.
            0 => {
                self.parameter_fifo[self.parameter_count as usize] = value;
                self.parameter_count += 1;
            },

            // Write interrupt enable flags.
            1 => {
                self.interrupt_enable_register = value & 0x1F;
            },

            _ => (),
        };
    }

    /// This function writes a byte to port 0x1F801803.
    fn write_1803(
        &mut self,
        bridge: &mut dyn CdromDriveBridge,
        value: u8
    ) {

        // Act depending on port index.
        match self.port_index {

            // This is the request register port.
            0 => {
                match value & 0x80 {

                    // Reset data fifo.
                    0 => {
                        self.data_index = 0;
                    },

                    // Wants data.
                    // We have already filled fifo so do nothing.
                    _ => (),
                };
            },

            // Deal with interrupt acknowledgement.
            1 => {

                // Clear parameter fifop if specified.
                if value & 0x40 == 0x40 {
                    self.clear_parameter_fifo();
                }

                // Define reset mask.
                let mut interrupt_reset_mask = value;
                interrupt_reset_mask = !interrupt_reset_mask & 0x1F;
                self.interrupt_flag_register &= interrupt_reset_mask;

                // Check if command has to issue a second response or not.
                self.response_received = 0;
                if self.needs_second_response {
                    self.clear_response_fifo();
                    self.execute_command(
                        bridge,
                        self.current_command,
                        self.needs_second_response
                    );
                }
            },

            _ => (),
        };
    }
}

#[cfg(test)]
mod tests;
