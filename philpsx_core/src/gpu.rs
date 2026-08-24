// SPDX-License-Identifier: GPL-3.0
// gpu.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

/// This module contains the default GPU implementation. There may
/// be others in future.
pub mod psx_gpu;

/// This trait provides an implementation-opaque way of calling GPU
/// methods from elsewhere in the system.
pub trait Gpu {

    /// Implementations must use this to increment the GPU cycle count.
    fn append_sync_cycles(&mut self, cycles: i32);

    /// Implementations must use this to determine if the GPU is
    /// currently within the hblank phase of the scanline.
    fn is_in_hblank(&self) -> bool;

    /// Implementations must use this to determine if the GPU is
    /// currently within the vblank phase of screen drawing.
    fn is_in_vblank(&self) -> bool;

    /// Implementations must use this to determine how many GPU
    /// cycles will be left after a round of dotclock timer incrementation.
    fn how_many_dotclock_gpu_cycles_left(&self, gpu_cycles: i32) -> i32;

    /// Implementations must use this to determine how many dotclock
    /// timer increments are needed.
    fn how_many_dotclock_increments(&self, gpu_cycles: i32) -> i32;

    /// Implementations must use this to determine how many GPU
    /// cycles will be left after a round of hblank timer incrementation.
    fn how_many_hblank_gpu_cycles_left(&self, gpu_cycles: i32) -> i32;

    /// Implementations must use this to determine how many hblank
    /// timer increments are needed.
    fn how_many_hblank_increments(&self, gpu_cycles: i32) -> i32;
}

/// This trait provides an implementation-opaque way of the GPU
/// calling methods from elsewhere in the system via a 'bridge'.
pub trait GpuBridge {
    
    /// The GPU must call this to set the GPU interrupt delay.
    fn set_gpu_interrupt_delay(&mut self, gpu: &mut dyn Gpu, delay: i32);
}