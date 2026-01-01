// SPDX-License-Identifier: GPL-3.0
// spu.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

/// This module contains the default sound chip implementation. There
/// may be others in future.
pub mod psx_spu;

/// This trait provides an implementation-opaque way of calling SPU
/// methods from elsewhere in the system.
pub trait Spu {

    /// This must be called in order to read a byte from the SPU.
    fn read_byte(&self, address: usize) -> i8;

    /// This must be called in order to write a byte to the SPU.
    fn write_byte(&mut self, address: usize, value: i8);
}