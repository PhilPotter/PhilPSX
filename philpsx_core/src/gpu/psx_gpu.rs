// SPDX-License-Identifier: GPL-3.0
// psx_gpu.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

use crate::gpu::Gpu;

/// Values for easier reading when dealing with GPU cycle math.
const GPU_CYCLES_PER_FRAME: i32 = 1069484;
const GPU_CYCLES_PER_SCANLINE: i32 = 3406;
const GPU_CYCLES_VBLANK: i32 = 817440;

/// This struct models the GPU (graphics chip) of the PlayStation.
pub struct PsxGpu {

    // Cycle stores.
    cpu_cycles: i32,
    gpu_cycles: i32,

    // Variable caches to prevent constant recalculation and
    // improve performance.
    horizontal_res: i32,
    vertical_res: i32,
    dot_factor: i32,
    interlace_enabled: bool,
}

/// Implementation functions for the GPU component itself.
impl PsxGpu {

    /// Creates a new GPU object with the correct initial state.
    pub fn new() -> Self {
        PsxGpu {

            // Setup cycle store counts.
            cpu_cycles: 0,
            gpu_cycles: 0,

            // Setup resolution and dot factor caches.
            horizontal_res: 256,
            vertical_res: 240,
            dot_factor: 10,
            interlace_enabled: false,
        }
    }
}

/// Implementation functions to be called from anything that understands what
/// a Gpu object is.
impl Gpu for PsxGpu {

    /// Increment the GPU cycle count.
    fn append_sync_cycles(&mut self, cycles: i32) {

        self.cpu_cycles += cycles;
    }

    /// Determine if the GPU is currently within the hblank phase of the scanline.
    fn is_in_hblank(&self) -> bool {

        // Get scanline.
        let scanline = self.gpu_cycles / GPU_CYCLES_PER_SCANLINE;

        // Get position in scanline.
        let position_in_scanline = self.gpu_cycles - scanline * GPU_CYCLES_PER_SCANLINE;

        // Get dot value we are on.
        let dot_value = (position_in_scanline as f32) / (self.dot_factor as f32);

        // Weird to do the casting this way, but gives equivalent behaviour
        // to the implicit casting rules in the C version this way.
        dot_value > self.horizontal_res as f32
    }

    /// This function is used to determine if the GPU is
    /// currently within the vblank phase of screen drawing.
    fn is_in_vblank(&self) -> bool {

        // If we are over GPU_CYCLES_VBLANK we must be in vblank area.
        self.gpu_cycles > GPU_CYCLES_VBLANK
    }

    /// This function is used to determine how many GPU cycles
    /// will be left after a round of dotclock timer incrementation.
    fn how_many_dotclock_gpu_cycles_left(&self, gpu_cycles: i32) -> i32 {
        gpu_cycles % self.dot_factor
    }

    /// This function is used to determine how many dotclock
    /// timer increments are needed.
    fn how_many_dotclock_increments(&self, gpu_cycles: i32) -> i32 {
        gpu_cycles / self.dot_factor
    }

    /// This function is used to determine how many GPU cycles
    /// will be left after a round of hblank timer incrementation.
    fn how_many_hblank_gpu_cycles_left(&self, gpu_cycles: i32) -> i32 {
        gpu_cycles % GPU_CYCLES_PER_SCANLINE
    }

    /// This function is used to determine how many hblank
    /// timer increments are needed.
    fn how_many_hblank_increments(&self, gpu_cycles: i32) -> i32 {
        gpu_cycles / GPU_CYCLES_PER_SCANLINE
    }
}