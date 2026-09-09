// SPDX-License-Identifier: GPL-3.0
// dma.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

use philpsx_utility::SystemBusHolder;

/// This module contains the default DMA arbiter implementation. There may
/// be others in future.
pub mod psx_dma;

/// This trait provides an implementation-opaque way of calling DMA arbiter
/// methods from elsewhere in the system.
pub trait DmaArbiter {
}

/// This trait provides an implementation-opaque way of the DMA arbiter
/// calling methods from elsewhere in the system via a 'bridge'.
pub trait DmaArbiterBridge {

    /// The DMA arbiter must call this to set the DMA interrupt delay.
    fn set_dma_interrupt_delay(&mut self, dma: &mut dyn DmaArbiter, delay: i32);

    /// The DMA arbiter must call this to set the system bus holder.
    fn set_system_bus_holder(&mut self, dma: &mut dyn DmaArbiter, holder: SystemBusHolder);

    /// The DMA arbiter must call this to convert a virtual address to a physical address.
    fn virtual_to_physical(&mut self, dma: &mut dyn DmaArbiter, address: u32) -> u32;
}