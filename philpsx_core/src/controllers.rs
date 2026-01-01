// SPDX-License-Identifier: GPL-3.0
// controllers.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

/// This module contains the default controllers implementation. There
/// may be others in future.
pub mod psx_controllers;

/// This trait provides an implementation-opaque way of calling controllers
/// methods from elsewhere in the system.
pub trait Controllers {

    /// This must be called in order to read a byte from the controllers implementation.
    fn read_byte(&mut self, address: i8) -> i8;

    /// This must be called in order to write a byte to the controllers implementation.
    fn write_byte(&mut self, address: i8, value: i8);

    /// This must be called in order to append sync cycles for the controllers implementation.
    fn append_sync_cycles(&mut self, cycles: i32);
}