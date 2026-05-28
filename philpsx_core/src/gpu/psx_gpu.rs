// SPDX-License-Identifier: GPL-3.0
// psx_gpu.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

use super::Gpu;

/// This struct models the GPU (graphics chip) of the PlayStation.
pub struct PsxGpu {
}

/// Implementation functions for the GPU component itself.
impl PsxGpu {

    /// Creates a new GPU object with the correct initial state.
    pub fn new() -> Self {
        PsxGpu {
        }
    }
}

/// Implementation functions to be called from anything that understands what
/// a Gpu object is.
impl Gpu for PsxGpu {

    /// Increment the GPU cycle count.
    fn append_sync_cycles(&mut self, cycles: i32) {
    }
}