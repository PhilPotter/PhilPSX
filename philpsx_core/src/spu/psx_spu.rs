// SPDX-License-Identifier: GPL-3.0
// psx_spu.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

use super::Spu;

/// The size of the fake register space.
const FAKE_REGISTER_SPACE_BYTES: usize = 1024;

/// This struct models the SPU (sound chip) of the PlayStation, and at present
/// is a stub. It is intended merely to store and return register values in
/// combination with the utility functions provided.
pub struct PsxSpu {

    // This stores all values written by the system to the SPU.
    fake_register_space: Vec<i8>,
}

/// Implementation functions for the SPU component itself.
impl PsxSpu {

    /// Creates a new SPU object with the correct initial state.
    pub fn new() -> Self {
        PsxSpu {

            // Initialise the fake register space to the correct size.
            fake_register_space: vec![0; FAKE_REGISTER_SPACE_BYTES],
        }
    }
}

/// Implementation functions to be called from anything that understands what
/// an Spu object is.
impl Spu for PsxSpu {

    /// Read a byte from the SPU.
    fn read_byte(&self, address: usize) -> i8 {

        self.fake_register_space[address]
    }

    /// Write a byte to the SPU.
    fn write_byte(&mut self, address: usize, value: i8) {

        self.fake_register_space[address] = value;
    }
}