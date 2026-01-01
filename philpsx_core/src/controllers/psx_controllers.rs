// SPDX-License-Identifier: GPL-3.0
// psx_controllers.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

use super::Controllers;
use philpsx_utility::CustomInteger;

/// The RX fifo byte count.
const RX_FIFO_BYTES: usize = 4;

/// This struct encapsulates the state of the PlayStation's controllers.
pub struct PsxControllers {

    // RX fifo.
    rx_fifo: [i8; RX_FIFO_BYTES],
    rx_count: i32,

    // Controller related variables.
    joy_baud: i32,    // JOY_BAUD
    joy_tx_data: i32, // JOY_TX_DATA
    joy_stat: i32,    // JOY_STAT
    joy_mode: i32,    // JOY_MODE
    joy_ctrl: i32,    // JOY_CTRL

    // Store number of cycles to change timer by.
    cycles: i32,
}

/// Implementation functions for the Controllers component itself.
impl PsxControllers {

    /// Creates a new Controllers object with the correct initial state.
    pub fn new() -> Self {
        PsxControllers {

            // Setup RX fifo.
            rx_fifo: [0; RX_FIFO_BYTES],
            rx_count: 0,

            // Setup controller related variables.
            joy_baud: 0,
            joy_tx_data: 0,
            joy_stat: 0,
            joy_mode: 0,
            joy_ctrl: 0,

            // Setup cycle store.
            cycles: 0,
        }
    }

    /// This function updates the baudrate timer.
    fn update_baudrate_timer(&mut self) {

        let mut baudrate = self.joy_stat.logical_rshift(11) & 0x1FFFFF;
        baudrate -= self.cycles;
        self.cycles = 0;
        if baudrate < 0 {
            baudrate = self.joy_baud * (self.joy_mode & 0x3) / 2;
        }
        self.joy_stat = (baudrate << 11) | (self.joy_stat & 0x7FF);
    }

    /// This function updates joy_stat.
    fn update_joy_stat(&mut self) {

        self.joy_stat |= 0x7
    }
}

/// Implementation functions to be called from anything that understands what
/// a Controllers object is.
impl Controllers for PsxControllers {

    /// This reads a byte from the controllers implementation.
    fn read_byte(&mut self, address: i8) -> i8 {

        // Update baudrate timer.
        self.update_baudrate_timer();

        // Read the correct value based upon the passed in address byte.
        match address {

            // JOY_RX_DATA 1st fifo entry.
            0x40 => {
                if self.rx_count > 0 {
                    let fifo_value = self.rx_fifo[0];
                    self.rx_count -= 1;
                    fifo_value
                } else {
                    0
                }
            },

            // JOY_STAT 1st (lowest) byte.
            0x44 => {
                self.update_joy_stat();
                (self.joy_stat & 0xFF) as i8
            },

            // JOY_STAT 2nd byte.
            0x45 => (self.joy_stat.logical_rshift(8) & 0xFF) as i8,

            // JOY_STAT 3rd byte.
            0x46 => (self.joy_stat.logical_rshift(16) & 0xFF) as i8,

            // JOY_STAT 4th (highest) byte.
            0x47 => (self.joy_stat.logical_rshift(24) & 0xFF) as i8,

            // JOY_MODE lower byte.
            0x48 => (self.joy_mode & 0xFF) as i8,

            // JOY_MODE higher byte.
            0x49 => (self.joy_mode.logical_rshift(8) & 0xFF) as i8,

            // JOY_CTRL lower byte.
            0x4A => (self.joy_ctrl & 0xFF) as i8,

            // JOY_CTRL higher byte.
            0x4B => (self.joy_ctrl.logical_rshift(8) & 0xFF) as i8,

            // JOY_BAUD lower byte.
            0x4E => (self.joy_baud & 0xFF) as i8,

            // JOY_BAUD higher byte.
            0x4F => (self.joy_baud.logical_rshift(8) & 0xFF) as i8,

            _ => 0
        }
    }

    /// This writes a byte to the controllers implementation.
    fn write_byte(&mut self, address: i8, value: i8) {

        // Update baudrate timer.
        self.update_baudrate_timer();

        // Write the correct value based upon the passed in address byte.
        match address {

            // JOY_TX_DATA lower byte.
            0x40 => self.joy_tx_data = (value as i32) & 0xFF,

            // JOY_MODE lower byte.
            0x48 => self.joy_mode = (self.joy_mode & 0xFF00) | ((value as i32) & 0xFF),

            // JOY_MODE higher byte.
            0x49 => self.joy_mode = (((value as i32) & 0xFF) << 8) | (self.joy_mode & 0xFF),

            // JOY_CTRL lower byte.
            0x4A => self.joy_ctrl = (self.joy_ctrl & 0xFF00) | ((value as i32) & 0xFF),

            // JOY_CTRL higher byte.
            0x4B => self.joy_ctrl = (((value as i32) & 0xFF) << 8) | (self.joy_ctrl & 0xFF),

            // JOY_BAUD lower byte.
            0x4E => {
                self.joy_baud = (self.joy_baud & 0xFF00) | ((value as i32) & 0xFF);
                let baudrate = self.joy_baud * (self.joy_mode & 0x3) / 2;
                self.joy_stat = (baudrate << 11) | (self.joy_stat & 0x7FF);
            },

            // JOY_BAUD higher byte.
            0x4F => {
                self.joy_baud = (((value as i32) & 0xFF) << 8) | (self.joy_baud & 0xFF);
                let baudrate = self.joy_baud * (self.joy_mode & 0x3) / 2;
                self.joy_stat = (baudrate << 11) | (self.joy_stat & 0x7FF);
            },

            _ => ()
        };
    }

    /// This appends sync cycles for the controllers implementation.
    fn append_sync_cycles(&mut self, cycles: i32) {

        self.cycles += cycles;
    }
}