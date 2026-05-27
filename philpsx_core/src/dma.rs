// SPDX-License-Identifier: GPL-3.0
// dma.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

/// This module contains the default DMA arbiter implementation. There may
/// be others in future.
pub mod psx_dma;

/// This trait provides an implementation-opaque way of calling DMA arbiter
/// methods from elsewhere in the system.
pub trait DmaArbiter {
}