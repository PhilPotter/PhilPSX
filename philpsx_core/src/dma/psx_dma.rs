// SPDX-License-Identifier: GPL-3.0
// psx_dma.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

use super::DmaArbiter;

/// This struct models orchestratiopn of DMA operations inside the PlayStation.
pub struct PsxDmaArbiter {
}

/// Implementation functions for the DMA arbiter component itself.
impl PsxDmaArbiter {

    /// Creates a new DMA arbiter object with the correct initial state.
    pub fn new() -> Self {
        PsxDmaArbiter {
        }
    }
}

/// Implementation functions to be called from anything that understands what
/// a DMA arbiter object is.
impl DmaArbiter for PsxDmaArbiter {
}