// SPDX-License-Identifier: GPL-3.0
// empty_cd.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

use philpsx_utility::error::PhilPSXError;

use super::Cdrom;

/// This struct is intended to represent a non-existent CD.
pub struct EmptyCd {}

/// Implementation functions for EmptyCd.
impl EmptyCd {

    /// This function creates a new empty CD.
    pub fn new() -> Self {

        log::info!("CD: Empty...");

        EmptyCd {}
    }
}

/// Functions to satisfy Cdrom trait.
impl Cdrom for EmptyCd {

    /// This function tells us the drive is loaded.
    fn is_loaded(&self) -> bool {
        false
    }

    /// This function just errors when we read from an address.
    fn read_byte(&mut self, _address: usize) -> Result<u8, Box<dyn std::error::Error>> {
        Err(PhilPSXError::error("CD: Can't read byte, no disc loaded"))
    }
}