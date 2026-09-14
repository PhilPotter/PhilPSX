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

    /// The DMA arbiter must call this to invoke a chunk copy from the
    /// CD-ROM drive to main memory.
    fn cdrom_drive_chunk_copy(
        &mut self,
        dma: &mut dyn DmaArbiter,
        starting_byte_address: u32,
        num_of_bytes: u32
    );

    /// The DMA arbiter must call this to set the DMA interrupt delay.
    fn set_dma_interrupt_delay(&mut self, dma: &mut dyn DmaArbiter, delay: i32);

    /// The DMA arbiter must call this to set the system bus holder.
    fn set_system_bus_holder(&mut self, dma: &mut dyn DmaArbiter, holder: SystemBusHolder);

    /// The DMA arbiter must call this to convert a virtual address to a physical address.
    fn virtual_to_physical(&mut self, dma: &mut dyn DmaArbiter, address: u32) -> u32;

    /// The DMA arbiter must call this to read a word from the system address space.
    fn read_word(&mut self, dma: &mut dyn DmaArbiter, address: u32) -> u32;

    /// The DMA arbiter must call this to read a byte from the system address space.
    fn read_byte(&mut self, dma: &mut dyn DmaArbiter, address: u32) -> u8;

    /// The DMA arbiter must call this to write a word to the system address space.
    fn write_word(&mut self, dma: &mut dyn DmaArbiter, address: u32, value: u32);

    /// The DMA arbiter must call this to write a byte to the system address space.
    fn write_byte(&mut self, dma: &mut dyn DmaArbiter, address: u32, value: u8);

    /// The DMA arbiter must use this to submit GP0 commands to the GPU.
    fn gpu_submit_to_gp0(&mut self, dma: &mut dyn DmaArbiter, word: u32);

    /// The DMA arbiter must use this to read GPU responses.
    fn gpu_read_response(&mut self, dma: &mut dyn DmaArbiter) -> u32;
}