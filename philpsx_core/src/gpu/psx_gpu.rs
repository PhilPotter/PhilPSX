// SPDX-License-Identifier: GPL-3.0
// psx_gpu.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

use crate::{
    gpu::{Gpu, GpuBridge},
};

/// Values for easier reading when dealing with GPU cycle math.
const GPU_CYCLES_PER_FRAME: i32 = 1069484;
const GPU_CYCLES_PER_SCANLINE: i32 = 3406;
const GPU_CYCLES_VBLANK: i32 = 817440;

/// The size of our VRAM (2 MiB).
const VRAM_SIZE_IN_BYTES: usize = 2097152;

/// Number of FIFO buffer slots.
const FIFO_BUFFER_SLOTS: usize = 16;

/// This struct models the GPU (graphics chip) of the PlayStation.
pub struct PsxGpu {

    // This lets us store values for DMA transfers.
    dma_buffer_index: i32,
    dma_needed_bytes: i32,
    dma_buffer: Vec<u8>,
    dma_read_in_progress: i32,
    dma_write_in_progress: i32,
    dma_width_in_pixels: i32,
    dma_height_in_pixels: i32,

    // Command FIFO buffer for GP0 commands.
    fifo_buffer: [u32; FIFO_BUFFER_SLOTS],
    commands_in_fifo: u32,

    // Status register.
    status_register: u32,

    // X and Y display start variables.
    x_start: u32,
    y_start: u32,

    // X1 and X2 horizontal display range variables.
    x1: u32,
    x2: u32,

    // Y1 and Y2 vertical display range variables.
    y1: u32,
    y2: u32,

    // Texture window.
    texture_window: u32,

    // Drawing area top-left and bottom-right variables.
    drawing_area_top_left: u32,
    drawing_area_bottom_right: u32,

    // Drawing offset.
    drawing_offset: u32,

    // Cycle stores.
    cpu_cycles: i32,
    gpu_cycles: i32,

    // This allows us to dynamically create the odd/even
    // line flag (bit 31) in the status register.
    odd_or_even: i32,

    // Variable caches to prevent constant recalculation and
    // improve performance.
    horizontal_res: i32,
    vertical_res: i32,
    dot_factor: i32,
    interlace_enabled: bool,

    // This lets us trigger only once per frame.
    vblank_triggered: bool,
}

/// Implementation functions for the GPU component itself.
impl PsxGpu {

    /// Creates a new GPU object with the correct initial state.
    pub fn new() -> Self {
        PsxGpu {

            // Setup DMA buffer.
            dma_buffer_index: -1,
            dma_needed_bytes: -1,
            dma_buffer: vec![0; VRAM_SIZE_IN_BYTES],
            dma_read_in_progress: -1,
            dma_write_in_progress: -1,
            dma_width_in_pixels: -1,
            dma_height_in_pixels: -1,

            // Setup FIFO buffer and counter.
            fifo_buffer: [0; FIFO_BUFFER_SLOTS],
            commands_in_fifo: 0,

            // Setup status register.
            status_register: 0x14902000,

            // Setup X and Y display start variables.
            x_start: 0,
            y_start: 0,

            // Setup display range variables.
            x1: 0x260,
            x2: 0xC60,
            y1: 0x1F,
            y2: 0x127,

            // Setup texture window variable.
            texture_window: 0,

            // Setup drawing area for top-left and bottom-right.
            drawing_area_top_left: 0,
            drawing_area_bottom_right: 0,

            // Setup drawing offset.
            drawing_offset: 0,

            // Setup cycle store counts.
            cpu_cycles: 0,
            gpu_cycles: 0,

            // Setup odd/even line variable.
            odd_or_even: 0,

            // Setup resolution and dot factor caches.
            horizontal_res: 256,
            vertical_res: 240,
            dot_factor: 10,
            interlace_enabled: false,

            // Setup vblank triggered flag.
            vblank_triggered: false,
        }
    }

    /// This function triggers a vblank interrupt, also updating the screen.
    fn trigger_vblank_interrupt(&mut self, bridge: &mut dyn GpuBridge) {

        // Trigger the interrupt.
        bridge.set_gpu_interrupt_delay(self, 0);

        // Update screen.
        self.display_screen();
    }

    /// This function did display the screen at the end of each frame in the C
    /// version - currently it's stubbed out here.
    fn display_screen(&mut self) {
        // This was originally a 'GpuCommand' struct block in the C version,
        // which would get set with the correct params and then added to a work
        // queue to update the screen on the GL worker thread.
    }

    /// This function lets us read the DMA buffer. Originally this was synchronised
    /// with a mutex in the C version, but for now we are going single-threaded.
    fn read_dma_buffer(&self, index: usize) -> u8 {
        self.dma_buffer[index]
    }

    /// This function lets us write to DMA buffer. Originally this was synchronised
    /// with a mutex in the C version, but for now we are going single-threaded.
    fn write_dma_buffer(&mut self, index: usize, value: u8) {
        self.dma_buffer[index] = value;
    }

    // Internal numbers GPn command functions are handled below.

    /// This function fills a rectangle in the VRAM.
    fn gp0_02(&mut self, command: u32, destination: u32, dimensions: u32) {
        // Just a stub for now.
    }

    /// This function triggers a GPU interrupt.
    fn gp0_1f(&mut self, bridge: &mut dyn GpuBridge) {

        // Set IRQ flag in status register.
        self.status_register |= 0x01000000;

        // Set flag in interrupt status register to trigger IRQ if enabled.
        bridge.set_gpu_interrupt_delay(self, 0);
    }

    /// This function copies a rectangle within VRAM.
    fn gp0_80(
        &mut self,
        command: u32,
        source_coord: u32,
        destination_coord: u32,
        width_and_height: u32
    ) {
        // Just a stub for now.
    }

    /// This function copies a rectangle into the VRAM.
    fn gp0_a0(&mut self, command: u32, destination: u32, dimensions: u32) {
        // Just a stub for now.
    }

    /// This function copies a rectangle from the VRAM.
    fn gp0_c0(&mut self, command: u32, destination: u32, dimensions: u32) {
        // Just a stub for now.
    }

    /// This function sets the draw mode ("texpage") setting.
    fn gp0_e1(&mut self, command: u32) {

        // Set bits 0-10 of status register.
        self.status_register &= 0xFFFFF800;
        self.status_register |= command & 0x7FF;

        // Set bit 15 of status register.
        self.status_register &= 0xFFFF7FFF;
        self.status_register |= (command << 4) & 0x8000;
    }

    /// This function sets the texture window setting.
    fn gp0_e2(&mut self, command: u32) {

        // Store in texture window variable.
        self.texture_window = command & 0xFFFFF;
    }

    /// This function sets the top-left drawing area.
    fn gp0_e3(&mut self, command: u32) {

        // Store in top-left drawing area variable.
        self.drawing_area_top_left = command & 0xFFFFF;
    }

    /// This function sets the bottom-right drawing area.
    fn gp0_e4(&mut self, command: u32) {

        // Store in bottom-right drawing area variable.
        self.drawing_area_bottom_right = command & 0xFFFFF;
    }

    /// This function sets the drawing offset.
    fn gp0_e5(&mut self, command: u32) {

        // Store in drawing offset variable.
        self.drawing_offset = command & 0x3FFFFF;
    }

    /// This function affects the mask bit settings in the status register.
    fn gp0_e6(&mut self, command: u32) {

        // Set the two bits (11 and 12) in the status register.
        self.status_register &= 0xFFFFE7FF;
        self.status_register |= (command & 0x3) << 11;
    }

    /// This function resets the GPU.
    fn gp1_00(&mut self, command: u32) {

        // Clear fifo.
        self.gp1_01(command);

        // Reset interrupt flag in status register.
        self.gp1_02(command);

        // Turn display off.
        self.gp1_03(0x1);

        // Set DMA direction to off.
        self.gp1_04(0);

        // Set start of display area.
        self.gp1_05(0);

        // Set horizontal display range variables.
        self.gp1_06(0xC60260);

        // Set vertical display range variables.
        self.gp1_07(0x49C1F);

        // Set display mode to 256x240 PAL.
        self.gp1_08(0x8);

        // Set display attributes.
        self.gp0_e1(0);
        self.gp0_e2(0);
        self.gp0_e3(0);
        self.gp0_e4(0);
        self.gp0_e5(0);
        self.gp0_e6(0);
    }

    /// This function emulates GP1_01, which resets the command buffer.
    fn gp1_01(&mut self) {

        // Empty buffer and set commands_in_fifo to 0.
        self.fifo_buffer.fill(0);
        self.commands_in_fifo = 0;
    }

    /// This function acknowledges a GPU interrupt (not a vblank interrupt,
    /// but one actually requested by the GPU with GP0_1F).
    fn gp1_02(&mut self) {

        // Reset IRQ flag in status register.
        self.status_register &= 0xFEFFFFFF;
    }

    /// This function enables or disables the display.
    fn gp1_03(&mut self, command: u32) {

        // Enable or disable display based on command word.
        let command = (command & 0x1) << 23;
        self.status_register &= 0xFF7FFFFF;
        self.status_register |= command;
    }

    /// This function sets the DMA direction / data request.
    fn gp1_04(&mut self, command: u32) {

        // Set direction in status register.
        let command = (command & 0x3) << 29;
        self.status_register &= 0x9FFFFFFF;
        self.status_register |= command;
    }

    /// This function sets the start of the display area in VRAM.
    fn gp1_05(&mut self, command: u32) {

        // Set X half-word address (0-1023).
        self.x_start = command & 0x3FF;
        self.y_start = (command >> 10) & 0x1FF;
    }

    /// This function sets the horizontal display range.
    fn gp1_06(&mut self, command: u32) {

        // Set X1 and X2.
        self.x1 = 0xFFF & command;
        self.x2 = 0xFFF & (command >> 12);
    }

    /// This function sets the vertical display range.
    fn gp1_07(&mut self, command: u32) {

        // Set Y1 and Y2.
        self.y1 = 0x3FF & command;
        self.y2 = 0x3FF & (command >> 10);
    }
}

/// Implementation functions to be called from anything that understands what
/// a Gpu object is.
impl Gpu for PsxGpu {

    /// Increment the GPU cycle count.
    fn append_sync_cycles(&mut self, cycles: i32) {

        self.cpu_cycles += cycles;
    }

    /// Keeps counters and such like up to date.
    fn execute_gpu_cycles(&mut self, bridge: &mut dyn GpuBridge) {

        // Convert CPU cycles to GPU cycles.
        let mut new_gpu_cycles = self.gpu_cycles + self.cpu_cycles * 11 / 7;

        // Test if we need to trigger a vblank interrupt.
        if new_gpu_cycles > GPU_CYCLES_VBLANK && !self.vblank_triggered {
            self.vblank_triggered = true;
            self.trigger_vblank_interrupt(bridge);
        }

        if new_gpu_cycles > GPU_CYCLES_PER_FRAME {
            // Reset vblank interrupt status.
            self.vblank_triggered = false;

            // Check if we are on odd or even frame.
            let frame_traversals = new_gpu_cycles / GPU_CYCLES_PER_FRAME;
            self.odd_or_even = if frame_traversals % 2 == 1 {
                !self.odd_or_even & 0x1
            } else {
                self.odd_or_even
            };

            // Modify new_gpu_cycles to reflect we are in a subsequent frame.
            new_gpu_cycles %= GPU_CYCLES_PER_FRAME;
        }

        // Store state.
        self.gpu_cycles = new_gpu_cycles;
        self.cpu_cycles = 0;
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