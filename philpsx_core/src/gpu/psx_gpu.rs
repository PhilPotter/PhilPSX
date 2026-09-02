// SPDX-License-Identifier: GPL-3.0
// psx_gpu.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

use philpsx_utility::EndiannessSwapper;
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

    // This vector stores line parameters for the line rendering commands.
    line_parameters: Vec<u32>,

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
    commands_in_fifo: usize,

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

    // GPUREAD latch value.
    gpuread_latch_value: u32,
    gpuread_latched: bool,

    // Cycle stores.
    cpu_cycles: i32,
    gpu_cycles: i32,

    // This allows us to dynamically create the odd/even
    // line flag (bit 31) in the status register. We don't
    // appear to actually ever change this at all, so this
    // is likely a bug in the original version that I'll
    // need to dig into at some point.
    odd_or_even: u32,

    // Variable caches to prevent constant recalculation and
    // improve performance.
    horizontal_res: u32,
    vertical_res: u32,
    dot_factor: u32,
    interlace_enabled: bool,

    // This lets us trigger only once per frame.
    vblank_triggered: bool,
}

/// Implementation functions for the GPU component itself.
impl PsxGpu {

    /// Creates a new GPU object with the correct initial state.
    pub fn new() -> Self {
        PsxGpu {

            // Setup line parameters vector.
            line_parameters: vec![],

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

            // Setup GPUREAD latch.
            gpuread_latch_value: 0,
            gpuread_latched: false,

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
    fn gp1_00(&mut self) {

        // Clear fifo.
        self.gp1_01();

        // Reset interrupt flag in status register.
        self.gp1_02();

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

    /// This function sets the display mode.
    fn gp1_08(&mut self, command: u32) {

        // Mask out the relevant bits in status register.
        self.status_register &= 0xFF80BFFF;

        // Horizontal resolution 1.
        let temp_horiz_1 = command & 0x3;
        self.status_register |= (command & 0x3) << 17;

        // Vertical resolution.
        let temp_vert = (command & 0x4) >> 2;
        self.status_register |= (command & 0x4) << 17;

        // Video mode.
        self.status_register |= (command & 0x8) << 17;

        // Display area colour depth.
        self.status_register |= (command & 0x10) << 17;

        // Vertical interlace (also set cache value).
        self.interlace_enabled = (command & 0x20) > 0;
        self.status_register |= (command & 0x20) << 17;

        // Horizontal resolution 2.
        let temp_horiz_2 = (command & 0x40) >> 6;
        self.status_register |= (command & 0x40) << 10;

        // Reverse flag.
        self.status_register |= (command & 0x80) << 7;

        // Now set cache values for horizontal resolution and dot factor.
        match temp_horiz_2 {

            1 => {
                self.horizontal_res = 368;
                self.dot_factor = 7;
            },

            // Default but represents 0 as it's the only other possibility.
            _ => {
                match temp_horiz_1 {

                    0 => {
                        self.horizontal_res = 256;
                        self.dot_factor = 10;
                    },

                    1 => {
                        self.horizontal_res = 320;
                        self.dot_factor = 8;
                    },

                    2 => {
                        self.horizontal_res = 512;
                        self.dot_factor = 5;
                    },

                    // Represents 3 as the only other possibility.
                    _ => {
                        self.horizontal_res = 640;
                        self.dot_factor = 4;
                    },
                }
            },
        }

        // Now set cache value for vertical resolution.
        match temp_vert {

            0 => {
                self.vertical_res = 240;
            },

            // Represents 1 as the only other possibility.
            _ => {
                self.vertical_res = if self.interlace_enabled {
                    480
                } else {
                    240
                };
            },
        }
    }

    /// This function allows disabling of texturing by other commands.
    fn gp1_09(&mut self, command: u32) {

        // Mask bit 15 of status register.
        self.status_register &= 0xFFFF7FFF;

        // Merge in command bit.
        self.status_register |= (command & 0x1) << 15;
    }

    /// This function allows GPU information to be read.
    fn gp1_10(&mut self, command: u32) {

        // Determine what to put in latch value.
        match command & 0xF {

            // Old value.
            0 | 0x1 | 0x6 | 0x9..=0xF => {
                self.gpuread_latched = true;
            },

            // Texture window.
            0x2 => {
                self.gpuread_latch_value = self.texture_window & 0xFFFFF;
                self.gpuread_latched = true;
            },

            // Drawing area top-left.
            0x3 => {
                self.gpuread_latch_value = self.drawing_area_top_left & 0xFFFFF;
                self.gpuread_latched = true;
            },

            // Drawing area bottom-right.
            0x4 => {
                self.gpuread_latch_value = self.drawing_area_bottom_right & 0xFFFFF;
                self.gpuread_latched = true;
            },

            // Drawing offset.
            0x5 => {
                self.gpuread_latch_value = self.drawing_offset & 0x3FFFFF;
                self.gpuread_latched = true;
            },

            // GPU type.
            0x7 => {
                self.gpuread_latch_value = 2;
                self.gpuread_latched = true;
            },

            // Unknown (0).
            0x8 => {
                self.gpuread_latch_value = 0;
                self.gpuread_latched = true;
            },

            _ => (),
        }
    }

    /// This function renders all forms of line primitive supported by the
    /// PlayStation hardware.
    fn any_line(
        &mut self,
        command: u32,
        params_list: &[u32]
    ) {
        // Just a stub for now.
    }

    /// This function draws a monochrome three or four point polygon.
    fn monochrome_polygon(
        &mut self,
        command: u32,
        vertex1: u32,
        vertex2: u32,
        vertex3: u32,
        vertex4: u32
    ) {
        // Just a stub for now.
    }

    /// This function draws a shaded three or four point polygon.
    fn shaded_polygon(
        &mut self,
        command: u32,
        vertex1: u32,
        colour2: u32,
        vertex2: u32,
        colour3: u32,
        vertex3: u32,
        colour4: u32,
        vertex4: u32
    ) {
        // Just a stub for now.
    }

    /// This function draws a textured three or four point polygon.
    fn shaded_textured_polygon(
        &mut self,
        command: u32,
        vertex1: u32,
        tex_coord1_and_palette: u32,
        colour2: u32,
        vertex2: u32,
        tex_coord2_and_tex_page: u32,
        colour3: u32,
        vertex3: u32,
        tex_coord3: u32,
        colour4: u32,
        vertex4: u32,
        tex_coord4: u32
    ) {
        // Just a stub for now.
    }

    /// This function draws a textured three or four point polygon.
    fn textured_polygon(
        &mut self,
        command: u32,
        vertex1: u32,
        tex_coord1_and_palette: u32,
        vertex2: u32,
        text_coord2_and_palette: u32,
        vertex3: u32,
        tex_coord3: u32,
        vertex4: u32,
        tex_coord4: u32
    ) {
        // Just a stub for now.
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
        gpu_cycles % (self.dot_factor as i32)
    }

    /// This function is used to determine how many dotclock
    /// timer increments are needed.
    fn how_many_dotclock_increments(&self, gpu_cycles: i32) -> i32 {
        gpu_cycles / (self.dot_factor as i32)
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

    /// This function is used to submit GP0 commands.
    fn submit_to_gp0(&mut self, bridge: &mut dyn GpuBridge, word: u32) {

        // Sync up to CPU.
        self.execute_gpu_cycles(bridge);

        // Exit if DMA read is in progress.
        if self.dma_read_in_progress != -1 {
            return;
        }

        // Switch endianness.
        let command_byte = word & 0xFF;
        let mut word = word.swap_endianness();

        // Write in progress, handle appropriately.
        if self.dma_write_in_progress != -1 {

            // Swap word order back as we are dumping this data straight to VRAM.
            word = word.swap_endianness();

            // Split first pixel into buffer.
            self.write_dma_buffer(self.dma_buffer_index as usize, (word >> 24) as u8);
            self.dma_buffer_index += 1;
            self.write_dma_buffer(self.dma_buffer_index as usize, ((word >> 16) & 0xFF) as u8);
            self.dma_buffer_index += 1;
            self.write_dma_buffer(self.dma_buffer_index as usize, 0);
            self.dma_buffer_index += 1;
            self.write_dma_buffer(self.dma_buffer_index as usize, 0);
            self.dma_buffer_index += 1;

            // Test for second pixel and split as well if needed.
            if self.dma_buffer_index != self.dma_needed_bytes {
                self.write_dma_buffer(self.dma_buffer_index as usize, ((word >> 8) & 0xFF) as u8);
                self.dma_buffer_index += 1;
                self.write_dma_buffer(self.dma_buffer_index as usize, (word & 0xFF) as u8);
                self.dma_buffer_index += 1;
                self.write_dma_buffer(self.dma_buffer_index as usize, 0);
                self.dma_buffer_index += 1;
                self.write_dma_buffer(self.dma_buffer_index as usize, 0);
                self.dma_buffer_index += 1;
            }

            if self.dma_buffer_index == self.dma_needed_bytes {

                // GP0(0xA0): copy rectangle (CPU to VRAM).
                if self.dma_write_in_progress == 0xA0 {
                    self.gp0_a0(self.fifo_buffer[0], self.fifo_buffer[1], self.fifo_buffer[2]);
                    self.gp1_01();
                }
                self.dma_write_in_progress = -1;

                // Set bit 26 (ready to receive command word) of status register.
                self.status_register |= 0x04000000;

                // Clear bit 27 (VRAM to CPU ready) of status register.
                self.status_register &= 0xF7FFFFFF;

                // Set bit 28 (DMA ready) of status register.
                self.status_register |= 0x10000000;
            }
        }
        // No write in progress, deal with normally.
        else {
            // Deal with command.
            if self.commands_in_fifo == 0 {

                // A lot of these blocks do the same thing, but they
                // are itemised and grouped as they are to keep style
                // roughly the same as in the C version, and because
                // the groupings represent logical GPU functionality.
                match command_byte {

                    // Trigger interrupt.
                    0x1F => self.gp0_1f(bridge),

                    // NOP (do nothing).
                    0x00 | 0x03 | 0x04 | 0x05 | 0x06 | 0x07 |
                    0x08 | 0x09 | 0x0A | 0x0B | 0x0C | 0x0D |
                    0x0E | 0x0F | 0x10 | 0x11 | 0x12 | 0x13 |
                    0x14 | 0x15 | 0x16 | 0x17 | 0x18 | 0x19 |
                    0x1A | 0x1B | 0x1C | 0x1D | 0x1E | 0xE0 |
                    0xE7 | 0xE8 | 0xE9 | 0xEA | 0xEB | 0xEC |
                    0xED | 0xEE | 0xEF => (),

                    // Clear cache, but not clear (it actually
                    // does nothing).
                    0x01 => (),

                    // GP0(0x02): fill rectangle in VRAM.
                    0x02 => {
                        self.fifo_buffer[self.commands_in_fifo] = word;
                        self.commands_in_fifo += 1;
                    },

                    // GP0(0x24): textured three-point polygon,
                    // opaque, texture-blending.
                    // GP0(0x25): textured three-point polygon,
                    // opaque, raw-texture.
                    // GP0(0x26): textured three-point polygon,
                    // semi-transparent, texture-blending.
                    // GP0(0x27): textured three-point polygon,
                    // semi-transparent, raw-texture.
                    // GP0(0x2C): textured four-point polygon,
                    // opaque, texture-blending.
                    // GP0(0x2D): textured four-point polygon,
                    // opaque, raw-texture.
                    // GP0(0x2E): textured four-point polygon,
                    // semi-transparent, texture-blending.
                    // GP0(0x2F): textured four-point polygon,
                    // semi-transparent, raw-texture.
                    0x24 | 0x25 | 0x26 | 0x27 | 0x2C | 0x2D |
                    0x2E | 0x2F => {
                        self.fifo_buffer[self.commands_in_fifo] = word;
                        self.commands_in_fifo += 1;
                    },

                    // GP0(0x34): shaded textured three-point
                    // polygon, opaque, texture-blending.
                    // GP0(0x35): undocumented, textured
                    // three-point polygon, opaque, no blending.
                    // GP0(0x36): shaded textured three-point
                    // polygon, semi-transparent,
                    // texture-blending.
                    // GP0(0x37): undocumented, textured
                    // three-point polygon, semi-transparent,
                    // no blending.
                    // GP0(0x3C): shaded textured four-point
                    // polygon, opaque, texture-blending.
                    // GP0(0x3D): undocumented, textured
                    // four-point polygon, opaque, no blending.
                    // GP0(0x3E): shaded textured four-point
                    // polygon, semi-transparent,
                    // texture-blending.
                    // GP0(0x3F): undocumented, textured
                    // four-point polygon, semi-transparent,
                    // no blending.
                    0x34 | 0x35 | 0x36 | 0x37 | 0x3C | 0x3D |
                    0x3E | 0x3F => {
                        self.fifo_buffer[self.commands_in_fifo] = word;
                        self.commands_in_fifo += 1;
                    },

                    // GP0(0x20): monochrome three-point polygon,
                    // opaque.
                    // GP0(0x21): undocumented command, same as
                    // GP0(0x20).
                    // GP0(0x22): monochrome three-point polygon,
                    // semi-transparent.
                    // GP0(0x23): undocumented command, same as
                    // GP0(0x22).
                    // GP0(0x28): monochrome four-point polygon,
                    // opaque.
                    // GP0(0x29): undocumented command, same as
                    // GP0(0x28).
                    // GP0(0x2A): monochrome four-point polygon,
                    // semi-transparent.
                    // GP0(0x2B): undocumented command, same as
                    // GP0(0x2A).
                    0x20 | 0x21 | 0x22 | 0x23 | 0x28 | 0x29 |
                    0x2A | 0x2B => {
                        self.fifo_buffer[self.commands_in_fifo] = word;
                        self.commands_in_fifo += 1;
                    },

                    // GP0(0x30): shaded three-point polygon,
                    // opaque.
                    // GP0(0x31): undocumented command, same as
                    // GP0(0x30).
                    // GP0(0x32): shaded three-point polygon,
                    // semi-transparent.
                    // GP0(0x33): undocumented command, same as
                    // GP0(0x32).
                    // GP0(0x38): shaded four-point polygon,
                    // opaque.
                    // GP0(0x39): undocumented command, same as
                    // GP0(0x38).
                    // GP0(0x3A): shaded four-point polygon,
                    // semi-transparent.
                    // GP0(0x3B): undocumented command, same as
                    // GP0(0x3A).
                    0x30 | 0x31 | 0x32 | 0x33 | 0x38 | 0x39 |
                    0x3A | 0x3B => {
                        self.fifo_buffer[self.commands_in_fifo] = word;
                        self.commands_in_fifo += 1;
                    },

                    // GP0(0x40): monochrome line, opaque.
                    // GP0(0x42): monochrome line,
                    // semi-transparent.
                    // GP0(0x48): monochrome poly-line, opaque.
                    // GP0(0x4A): monochrome poly-line,
                    // semi-transparent.
                    // GP0(0x50): shaded line, opaque.
                    // GP0(0x52): shaded line, semi-transparent.
                    // GP0(0x58): shaded poly-line, opaque.
                    // GP0(0x5A): shaded poly-line,
                    // semi-transparent.
                    0x40 | 0x42 | 0x48 | 0x4A | 0x50 | 0x52 |
                    0x58 | 0x5A => {
                        self.fifo_buffer[self.commands_in_fifo] = word;
                        self.commands_in_fifo += 1;
                    },

                    // GP0(0x60): monochrome rectangle, variable
                    // size, opaque.
                    // GP0(0x62): monochrome rectangle, variable
                    // size, semi-transparent.
                    // GP0(0x68): monochrome rectangle, 1x1,
                    // opaque.
                    // GP0(0x6A): monochrome rectangle, 1x1,
                    // semi-transparent.
                    // GP0(0x70): monochrome rectangle, 8x8,
                    // opaque.
                    // GP0(0x72): monochrome rectangle, 8x8,
                    // semi-transparent.
                    // GP0(0x78): monochrome rectangle, 16x16,
                    // opaque.
                    // GP0(0x7A): monochrome rectangle, 16x16,
                    // semi-transparent.
                    0x60 | 0x62 | 0x68 | 0x6A | 0x70 | 0x72 |
                    0x78 | 0x7A => {
                        self.fifo_buffer[self.commands_in_fifo] = word;
                        self.commands_in_fifo += 1;
                    },

                    // GP0(0x64): textured rectangle, variable
                    // size, opaque, texture-blending.
                    // GP0(0x65): textured rectangle, variable
                    // size, opaque, raw-texture.
                    // GP0(0x66): textured rectangle, variable
                    // size, semi-transparent, texture-blending.
                    // GP0(0x67): textured rectangle, variable
                    // size, semi-transparent, raw-texture.
                    // GP0(0x6C): textured rectangle, 1x1
                    // (nonsense), opaque, texture-blending.
                    // GP0(0x6D): textured rectangle, 1x1
                    // (nonsense), opaque, raw-texture.
                    // GP0(0x6E): textured rectangle, 1x1
                    // (nonsense), semi-transparent,
                    // texture-blending.
                    // GP0(0x6F): textured rectangle,
                    // 1x1 (nonsense), semi-transparent,
                    // raw-texture.
                    // GP0(0x74): textured rectangle, 8x8,
                    // opaque, texture-blending.
                    // GP0(0x75): textured rectangle, 8x8,
                    // opaque, raw-texture.
                    // GP0(0x76): textured rectangle, 8x8,
                    // semi-transparent, texture-blending.
                    // GP0(0x77): textured rectangle, 8x8,
                    // semi-transparent, raw-texture.
                    // GP0(0x7C): textured rectangle, 16x16,
                    // opaque, texture-blending.
                    // GP0(0x7D): textured rectangle, 16x16,
                    // opaque, raw-texture.
                    // GP0(0x7E): textured rectangle, 16x16,
                    // semi-transparent, texture-blending.
                    // GP0(0x7F): textured rectangle, 16x16,
                    // semi-transparent, raw-texture.
                    0x64 | 0x65 | 0x66 | 0x67 | 0x6C | 0x6D |
                    0x6E | 0x6F | 0x74 | 0x75 | 0x76 | 0x77 |
                    0x7C | 0x7D | 0x7E | 0x7F => {
                        self.fifo_buffer[self.commands_in_fifo] = word;
                        self.commands_in_fifo += 1;
                    },

                    // GP0(0x80): copy rectangle (VRAM to VRAM).
                    0x80 | 0x81 | 0x82 | 0x83 | 0x84 | 0x85 |
                    0x86 | 0x87 | 0x88 | 0x89 | 0x8A | 0x8B |
                    0x8C | 0x8D | 0x8E | 0x8F | 0x90 | 0x91 |
                    0x92 | 0x93 | 0x94 | 0x95 | 0x96 | 0x97 |
                    0x98 | 0x99 | 0x9A | 0x9B | 0x9C | 0x9D |
                    0x9E | 0x9F => {
                        self.fifo_buffer[self.commands_in_fifo] = word;
                        self.commands_in_fifo += 1;
                    },

                    // GP0(0xA0): copy rectangle (CPU to VRAM).
                    0xA0 | 0xA1 | 0xA2 | 0xA3 | 0xA4 | 0xA5 |
                    0xA6 | 0xA7 | 0xA8 | 0xA9 | 0xAA | 0xAB |
                    0xAC | 0xAD | 0xAE | 0xAF | 0xB0 | 0xB1 |
                    0xB2 | 0xB3 | 0xB4 | 0xB5 | 0xB6 | 0xB7 |
                    0xB8 | 0xB9 | 0xBA | 0xBB | 0xBC | 0xBD |
                    0xBE | 0xBF => {
                        self.fifo_buffer[self.commands_in_fifo] = word;
                        self.commands_in_fifo += 1;
                    },

                    // GP0(0xC0): copy rectangle (VRAM to CPU).
                    0xC0 | 0xC1 | 0xC2 | 0xC3 | 0xC4 | 0xC5 |
                    0xC6 | 0xC7 | 0xC8 | 0xC9 | 0xCA | 0xCB |
                    0xCC | 0xCD | 0xCE | 0xCF | 0xD0 | 0xD1 |
                    0xD2 | 0xD3 | 0xD4 | 0xD5 | 0xD6 | 0xD7 |
                    0xD8 | 0xD9 | 0xDA | 0xDB | 0xDC | 0xDD |
                    0xDE | 0xDF => {
                        self.fifo_buffer[self.commands_in_fifo] = word;
                        self.commands_in_fifo += 1;
                    },

                    // GP0(0xE1): draw mode ("texpage") setting.
                    0xE1 => self.gp0_e1(word),

                    // GP0(0xE2): texture window setting.
                    0xE2 => self.gp0_e2(word),

                    // GP0(0xE3): set drawing area (top-left).
                    0xE3 => self.gp0_e3(word),

                    // GP0(0xE4): set drawing area (bottom-right).
                    0xE4 => self.gp0_e4(word),

                    // GP0(0xE5): set drawing offset.
                    0xE5 => self.gp0_e5(word),

                    // GP0(0xE6): set mask setting bits.
                    0xE6 => self.gp0_e6(word),

                    // Unmapped as of now:
                    _ => log::warn!("GPU: GP0 submit unhandled: {:#010x}", word),
                }
            }
            // Command at index 0 in FIFO requires more parameters.
            else {

                match self.fifo_buffer[0] >> 24 {

                    // GP0(0x02): fill rectangle in VRAM.
                    0x02 => {
                        if self.commands_in_fifo < 3 {
                            self.fifo_buffer[self.commands_in_fifo] = word;
                            self.commands_in_fifo += 1;
                            if self.commands_in_fifo == 3 {
                                self.gp0_02(
                                    self.fifo_buffer[0],
                                    self.fifo_buffer[1],
                                    self.fifo_buffer[2]
                                );
                                self.gp1_01();
                            }
                        }
                    },

                    // GP0(0x24): textured three-point polygon,
                    // opaque, texture-blending.
                    // GP0(0x25): textured three-point polygon,
                    // opaque, raw-texture.
                    // GP0(0x26): textured three-point polygon,
                    // semi-transparent, texture-blending.
                    // GP0(0x27): textured three-point polygon,
                    // semi-transparent, raw-texture.
                    0x24 | 0x25 | 0x26 | 0x27 => {
                        if self.commands_in_fifo < 7 {
                            self.fifo_buffer[self.commands_in_fifo] = word;
                            self.commands_in_fifo += 1;
                            if self.commands_in_fifo == 7 {
                                self.textured_polygon(
                                    self.fifo_buffer[0],
                                    self.fifo_buffer[1],
                                    self.fifo_buffer[2],
                                    self.fifo_buffer[3],
                                    self.fifo_buffer[4],
                                    self.fifo_buffer[5],
                                    self.fifo_buffer[6],
                                    0,
                                    0
                                );
                                self.gp1_01();
                            }
                        }
                    },

                    // GP0(0x2C): textured four-point polygon,
                    // opaque, texture-blending.
                    // GP0(0x2D): textured four-point polygon,
                    // opaque, raw-texture.
                    // GP0(0x2E): textured four-point polygon,
                    // semi-transparent, texture-blending.
                    // GP0(0x2F): textured four-point polygon,
                    // semi-transparent, raw-texture.
                    0x2C | 0x2D | 0x2E | 0x2F => {
                        if self.commands_in_fifo < 9 {
                            self.fifo_buffer[self.commands_in_fifo] = word;
                            self.commands_in_fifo += 1;
                            if self.commands_in_fifo == 9 {
                                self.textured_polygon(
                                    self.fifo_buffer[0],
                                    self.fifo_buffer[1],
                                    self.fifo_buffer[2],
                                    self.fifo_buffer[3],
                                    self.fifo_buffer[4],
                                    self.fifo_buffer[5],
                                    self.fifo_buffer[6],
                                    self.fifo_buffer[7],
                                    self.fifo_buffer[8]
                                );
                                self.gp1_01();
                            }
                        }
                    },

                    // Shaded textured three-point polygon,
                    // opaque, texture-blending.
                    // Undocumented, textured three-point
                    // polygon, opaque, no blending.
                    // Shaded textured three-point polygon,
                    // semi-transparent, texture-blending.
                    // Undocumented, textured three-point
                    // polygon, semi-transparent, no blending.
                    0x34 | 0x35 | 0x36 | 0x37 => {
                        if self.commands_in_fifo < 9 {
                            self.fifo_buffer[self.commands_in_fifo] = word;
                            self.commands_in_fifo += 1;
                            if self.commands_in_fifo == 9 {
                                self.shaded_textured_polygon(
                                    self.fifo_buffer[0],
                                    self.fifo_buffer[1],
                                    self.fifo_buffer[2],
                                    self.fifo_buffer[3],
                                    self.fifo_buffer[4],
                                    self.fifo_buffer[5],
                                    self.fifo_buffer[6],
                                    self.fifo_buffer[7],
                                    self.fifo_buffer[8],
                                    0,
                                    0,
                                    0
                                );
                                self.gp1_01();
                            }
                        }
                    },

                    // Shaded textured four-point polygon,
                    // opaque, texture-blending.
                    // Undocumented, textured four-point polygon,
                    // opaque, no blending.
                    // Shaded textured four-point polygon,
                    // semi-transparent, texture-blending.
                    // Undocumented, textured four-point polygon,
                    // semi-transparent, no blending.
                    0x3C | 0x3D | 0x3E | 0x3F => {
                        if self.commands_in_fifo < 12 {
                            self.fifo_buffer[self.commands_in_fifo] = word;
                            self.commands_in_fifo += 1;
                            if self.commands_in_fifo == 12 {
                                self.shaded_textured_polygon(
                                    self.fifo_buffer[0],
                                    self.fifo_buffer[1],
                                    self.fifo_buffer[2],
                                    self.fifo_buffer[3],
                                    self.fifo_buffer[4],
                                    self.fifo_buffer[5],
                                    self.fifo_buffer[6],
                                    self.fifo_buffer[7],
                                    self.fifo_buffer[8],
                                    self.fifo_buffer[9],
                                    self.fifo_buffer[10],
                                    self.fifo_buffer[11]
                                );
                                self.gp1_01();
                            }
                        }
                    },

                    // GP0(0x20): monochrome three-point polygon,
                    // opaque.
                    // GP0(0x21): undocumented command, same as
                    // GP0(0x20).
                    // GP0(0x22): monochrome three-point polygon,
                    // semi-transparent.
                    // GP0(0x23): undocumented command, same as
                    // GP0(0x22).
                    0x20 | 0x21 | 0x22 | 0x23 => {
                        if self.commands_in_fifo < 4 {
                            self.fifo_buffer[self.commands_in_fifo] = word;
                            self.commands_in_fifo += 1;
                            if self.commands_in_fifo == 4 {
                                self.monochrome_polygon(
                                    self.fifo_buffer[0],
                                    self.fifo_buffer[1],
                                    self.fifo_buffer[2],
                                    self.fifo_buffer[3],
                                    0
                                );
                                self.gp1_01();
                            }
                        }
                    },

                    // GP0(0x28): monochrome four-point polygon,
                    // opaque.
                    // GP0(0x29): undocumented command, same as
                    // GP0(0x28).
                    // GP0(0x2A): monochrome four-point polygon,
                    // semi-transparent.
                    // GP0(0x2B): undocumented command, same as
                    // GP0(0x2B).
                    0x28 | 0x29 | 0x2A | 0x2B => {
                        if self.commands_in_fifo < 5 {
                            self.fifo_buffer[self.commands_in_fifo] = word;
                            self.commands_in_fifo += 1;
                            if self.commands_in_fifo == 5 {
                                self.monochrome_polygon(
                                    self.fifo_buffer[0],
                                    self.fifo_buffer[1],
                                    self.fifo_buffer[2],
                                    self.fifo_buffer[3],
                                    self.fifo_buffer[4]
                                );
                                self.gp1_01();
                            }
                        }
                    },

                    // GP0(0x30): shaded three-point polygon,
                    // opaque.
                    // GP0(0x31): undocumented command, same as
                    // GP0(0x30).
                    // GP0(0x32): shaded three-point polygon,
                    // semi-transparent.
                    // GP0(0x33): undocumented command, same as
                    // GP0(0x32).
                    0x30 | 0x31 | 0x32 | 0x33 => {
                        if self.commands_in_fifo < 6 {
                            self.fifo_buffer[self.commands_in_fifo] = word;
                            self.commands_in_fifo += 1;
                            if self.commands_in_fifo == 6 {
                                self.shaded_polygon(
                                    self.fifo_buffer[0],
                                    self.fifo_buffer[1],
                                    self.fifo_buffer[2],
                                    self.fifo_buffer[3],
                                    self.fifo_buffer[4],
                                    self.fifo_buffer[5],
                                    0,
                                    0
                                );
                                self.gp1_01();
                            }
                        }
                    },

                    // GP0(0x38): shaded four-point polygon,
                    // opaque.
                    // GP0(0x39): undocumented command, same as
                    // GP0(0x38).
                    // GP0(0x3A): shaded four-point polygon,
                    // semi-transparent.
                    // GP0(0x3B): undocumented command, same as
                    // GP0(0x3A).
                    0x38 | 0x39 | 0x3A | 0x3B => {
                        if self.commands_in_fifo < 8 {
                            self.fifo_buffer[self.commands_in_fifo] = word;
                            self.commands_in_fifo += 1;
                            if self.commands_in_fifo == 8 {
                                self.shaded_polygon(
                                    self.fifo_buffer[0],
                                    self.fifo_buffer[1],
                                    self.fifo_buffer[2],
                                    self.fifo_buffer[3],
                                    self.fifo_buffer[4],
                                    self.fifo_buffer[5],
                                    self.fifo_buffer[6],
                                    self.fifo_buffer[7]
                                );
                                self.gp1_01();
                            }
                        }
                    },

                    // GP0(0x40): monochrome line, opaque.
                    // GP0(0x42): monochrome line,
                    // semi-transparent.
                    0x40 | 0x42 => {
                        if self.line_parameters.len()  < 4 {
                            self.line_parameters.push(self.fifo_buffer[0] & 0xFFFFFF);
                            self.line_parameters.push(word);
                            if self.line_parameters.len() == 4 {
                                self.any_line(
                                    self.fifo_buffer[0],
                                    &self.line_parameters
                                );
                                self.line_parameters.clear();
                                self.gp1_01();
                            }
                        }
                    },
                }
            }
        }

        /*
		case -1: // No write in progress, deal with normally
			switch (gpu->commandsInFifo) {

				default: // Command at index 0 in FIFO requires more parameters
					switch (logical_rshift(gpu->fifoBuffer[0], 24) & 0xFF) {

						case 0x50: // GP0(0x50): shaded line, opaque
						case 0x52: // GP0(0x52): shaded line, semi-transparent
							if (ArrayList_getSize(gpu->lineParameters) < 4) {
								if (ArrayList_getSize(gpu->lineParameters)
										== 0) {
									ArrayList_addObject(gpu->lineParameters,
											(void *)(intptr_t)(
											gpu->fifoBuffer[0] & 0xFFFFFF));
								}
								ArrayList_addObject(gpu->lineParameters,
										(void *)(intptr_t)word);
								if (ArrayList_getSize(gpu->lineParameters)
										== 4) {
									GPU_anyLine(gpu,
											gpu->fifoBuffer[0],
											gpu->lineParameters);
									ArrayList_wipeAllObjects(
											gpu->lineParameters);
									GPU_GP1_01(gpu, 0);
								}
							}
							break;
						case 0x48: // GP0(0x48): monochrome poly-line, opaque
						case 0x4A: // GP0(0x4A): monochrome poly-line,
								   // semi-transparent
							// Variable number of components, so stop at
							// termination code
							if (word != 0x55555555 && word != 0x50005000) {
								ArrayList_addObject(gpu->lineParameters,
										(void *)(intptr_t)(gpu->fifoBuffer[0]
										& 0xFFFFFF));
								ArrayList_addObject(gpu->lineParameters,
										(void *)(intptr_t)word);
							} else {
								GPU_anyLine(gpu,
										gpu->fifoBuffer[0],
										gpu->lineParameters);
								ArrayList_wipeAllObjects(gpu->lineParameters);
								GPU_GP1_01(gpu, 0);
							}
							break;
						case 0x58: // GP0(0x58): shaded poly-line, opaque
						case 0x5A: // GP0(0x5A): shaded poly-line,
								   // semi-transparent
							// Variable number of components, so stop at
							// termination code
							if (word != 0x55555555 && word != 0x50005000) {
								if (ArrayList_getSize(gpu->lineParameters)
										== 0) {
									ArrayList_addObject(gpu->lineParameters,
											(void *)(intptr_t)(
											gpu->fifoBuffer[0] & 0xFFFFFF));
								}
								ArrayList_addObject(gpu->lineParameters,
										(void *)(intptr_t)word);
							} else {
								GPU_anyLine(gpu,
										gpu->fifoBuffer[0],
										gpu->lineParameters);
								ArrayList_wipeAllObjects(gpu->lineParameters);
								GPU_GP1_01(gpu, 0);
							}
							break;
						case 0x60: // GP0(0x60): monochrome rectangle, variable
								   // size, opaque
						case 0x62: // GP0(0x62): monochrome rectangle, variable
								   // size, semi-transparent
							if (gpu->commandsInFifo < 3) {
								gpu->fifoBuffer[gpu->commandsInFifo++] = word;
								if (gpu->commandsInFifo == 3) {
									GPU_monochromeRectangle(gpu,
											gpu->fifoBuffer[0],
											gpu->fifoBuffer[1],
											gpu->fifoBuffer[2]);
									GPU_GP1_01(gpu, 0);
								}
							}
							break;
						case 0x68: // GP0(0x68): monochrome rectangle, 1x1,
								   // opaque
						case 0x6A: // GP0(0x6A): monochrome rectangle, 1x1,
								   // semi-transparent
						case 0x70: // GP0(0x70): monochrome rectangle, 8x8,
								   // opaque
						case 0x72: // GP0(0x72): monochrome rectangle, 8x8,
								   // semi-transparent
						case 0x78: // GP0(0x78): monochrome rectangle, 16x16,
								   // opaque
						case 0x7A: // GP0(0x7A): monochrome rectangle, 16x16,
								   // semi-transparent
							if (gpu->commandsInFifo < 2) {
								gpu->fifoBuffer[gpu->commandsInFifo++] = word;
								if (gpu->commandsInFifo == 2) {
									int32_t rectangleSize = 0;
									switch (logical_rshift(
											gpu->fifoBuffer[0], 24) & 0xFF) {
										case 0x68:
										case 0x6A: // 1x1 size
											rectangleSize = 0x00010001;
											break;
										case 0x70:
										case 0x72: // 8x8 size
											rectangleSize = 0x00080008;
											break;
										case 0x78:
										case 0x7A: // 16x16 size
											rectangleSize = 0x00100010;
											break;
									}
									GPU_monochromeRectangle(gpu,
											gpu->fifoBuffer[0],
											gpu->fifoBuffer[1],
											rectangleSize);
									GPU_GP1_01(gpu, 0);
								}
							}
							break;
						case 0x64: // GP0(0x64): textured rectangle, variable
								   // size, opaque, texture-blending
						case 0x65: // GP0(0x65): textured rectangle, variable
								   // size, opaque, raw-texture
						case 0x66: // GP0(0x66): textured rectangle, variable
								   // size, semi-transparent, texture-blending
						case 0x67: // GP0(0x67): textured rectangle, variable
								   // size, semi-transparent, raw-texture
							if (gpu->commandsInFifo < 4) {
								gpu->fifoBuffer[gpu->commandsInFifo++] = word;
								if (gpu->commandsInFifo == 4) {
									GPU_texturedRectangle(gpu,
											gpu->fifoBuffer[0],
											gpu->fifoBuffer[1],
											gpu->fifoBuffer[2],
											gpu->fifoBuffer[3]);
									GPU_GP1_01(gpu, 0);
								}
							}
							break;
						case 0x6C: // GP0(0x6C): textured rectangle, 1x1
								   // (nonsense), opaque, texture-blending
						case 0x6D: // GP0(0x6D): textured rectangle, 1x1
								   // (nonsense), opaque, raw-texture
						case 0x6E: // GP0(0x6E): textured rectangle, 1x1
								   // (nonsense), semi-transparent, texture-blending
						case 0x6F: // GP0(0x6F): textured rectangle, 1x1
								   // (nonsense), semi-transparent, raw-texture
						case 0x74: // GP0(0x74): textured rectangle, 8x8,
								   // opaque, texture-blending
						case 0x75: // GP0(0x75): textured rectangle, 8x8,
								   // opaque, raw-texture
						case 0x76: // GP0(0x76): textured rectangle, 8x8,
								   // semi-transparent, texture-blending
						case 0x77: // GP0(0x77): textured rectangle, 8x8,
								   // semi-transparent, raw-texture
						case 0x7C: // GP0(0x7C): textured rectangle, 16x16,
								   // opaque, texture-blending
						case 0x7D: // GP0(0x7D): textured rectangle, 16x16,
								   // opaque, raw-texture
						case 0x7E: // GP0(0x7E): textured rectangle, 16x16,
								   // semi-transparent, texture-blending
						case 0x7F: // GP0(0x7F): textured rectangle, 16x16,
								   // semi-transparent, raw-texture
							if (gpu->commandsInFifo < 3) {
								gpu->fifoBuffer[gpu->commandsInFifo++] = word;
								if (gpu->commandsInFifo == 3) {
									int32_t rectangleSize = 0;
									switch (logical_rshift(
											gpu->fifoBuffer[0], 24) & 0xFF) {
										case 0x6C:
										case 0x6D:
										case 0x6E:
										case 0x6F: // 1x1 size
											rectangleSize = 0x00010001;
											break;
										case 0x74:
										case 0x75:
										case 0x76:
										case 0x77: // 8x8 size
											rectangleSize = 0x00080008;
											break;
										case 0x7C:
										case 0x7D:
										case 0x7E:
										case 0x7F: // 16x16 size
											rectangleSize = 0x00100010;
											break;
									}
									GPU_texturedRectangle(gpu,
											gpu->fifoBuffer[0],
											gpu->fifoBuffer[1],
											gpu->fifoBuffer[2],
											rectangleSize);
									GPU_GP1_01(gpu, 0);
								}
							}
							break;
						case 0x80:
						case 0x81:
						case 0x82:
						case 0x83:
						case 0x84:
						case 0x85:
						case 0x86:
						case 0x87:
						case 0x88:
						case 0x89:
						case 0x8A:
						case 0x8B:
						case 0x8C:
						case 0x8D:
						case 0x8E:
						case 0x8F:
						case 0x90:
						case 0x91:
						case 0x92:
						case 0x93:
						case 0x94:
						case 0x95:
						case 0x96:
						case 0x97:
						case 0x98:
						case 0x99:
						case 0x9A:
						case 0x9B:
						case 0x9C:
						case 0x9D:
						case 0x9E:
						case 0x9F: // GP0(0x80): copy rectangle (VRAM to VRAM)
							if (gpu->commandsInFifo < 4) {
								gpu->fifoBuffer[gpu->commandsInFifo++] = word;
								if (gpu->commandsInFifo == 4) {
									GPU_GP0_80(gpu,
											gpu->fifoBuffer[0],
											gpu->fifoBuffer[1],
											gpu->fifoBuffer[2],
											gpu->fifoBuffer[3]);
									GPU_GP1_01(gpu, 0);
								}
							}
							break;
						case 0xA0:
						case 0xA1:
						case 0xA2:
						case 0xA3:
						case 0xA4:
						case 0xA5:
						case 0xA6:
						case 0xA7:
						case 0xA8:
						case 0xA9:
						case 0xAA:
						case 0xAB:
						case 0xAC:
						case 0xAD:
						case 0xAE:
						case 0xAF:
						case 0xB0:
						case 0xB1:
						case 0xB2:
						case 0xB3:
						case 0xB4:
						case 0xB5:
						case 0xB6:
						case 0xB7:
						case 0xB8:
						case 0xB9:
						case 0xBA:
						case 0xBB:
						case 0xBC:
						case 0xBD:
						case 0xBE:
						case 0xBF: // GP0(0xA0): copy rectangle (CPU to VRAM)
							if (gpu->commandsInFifo < 3) {
								gpu->fifoBuffer[gpu->commandsInFifo++] = word;
								if (gpu->commandsInFifo == 3) {
									// Clear bit 28 (DMA ready) of status
									// register
									gpu->statusRegister &= 0xEFFFFFFF;

									// Clear bit 26 (ready to receive command
									// word) of status register
									gpu->statusRegister &= 0xFBFFFFFF;

									// Trigger DMA input
									gpu->dmaWriteInProgress = 0xA0;
									gpu->dmaBufferIndex = 0;
									gpu->dmaWidthInPixels =
											gpu->fifoBuffer[2] & 0xFFFF;
									gpu->dmaWidthInPixels =
											((gpu->dmaWidthInPixels - 1)
											& 0x3FF) + 1;
									gpu->dmaHeightInPixels = logical_rshift(
											gpu->fifoBuffer[2],
											16) & 0xFFFF;
									gpu->dmaHeightInPixels =
											((gpu->dmaHeightInPixels - 1)
											& 0x1FF) + 1;
									gpu->dmaWidthInPixels =
											(gpu->dmaWidthInPixels == 0) ?
											0x400 : gpu->dmaWidthInPixels;
									gpu->dmaHeightInPixels =
											(gpu->dmaHeightInPixels == 0) ?
											0x200 : gpu->dmaHeightInPixels;
									gpu->dmaNeededBytes =
											gpu->dmaWidthInPixels *
											gpu->dmaHeightInPixels;
									gpu->dmaNeededBytes *= 4;
								}
							}
							break;
						case 0xC0:
						case 0xC1:
						case 0xC2:
						case 0xC3:
						case 0xC4:
						case 0xC5:
						case 0xC6:
						case 0xC7:
						case 0xC8:
						case 0xC9:
						case 0xCA:
						case 0xCB:
						case 0xCC:
						case 0xCD:
						case 0xCE:
						case 0xCF:
						case 0xD0:
						case 0xD1:
						case 0xD2:
						case 0xD3:
						case 0xD4:
						case 0xD5:
						case 0xD6:
						case 0xD7:
						case 0xD8:
						case 0xD9:
						case 0xDA:
						case 0xDB:
						case 0xDC:
						case 0xDD:
						case 0xDE:
						case 0xDF: // GP0(0xC0): copy rectangle (VRAM to CPU)
							if (gpu->commandsInFifo < 3) {
								gpu->fifoBuffer[gpu->commandsInFifo++] = word;
								if (gpu->commandsInFifo == 3) {
									// Clear bit 28 (DMA ready) of status
									// register
									gpu->statusRegister &= 0xEFFFFFFF;

									// Set bit 27 (ready to send VRAM to CPU)
									gpu->statusRegister |= 0x08000000;

									// Clear bit 26 (ready to receive command
									// word) of status register
									gpu->statusRegister &= 0xFBFFFFFF;

									// Trigger DMA output
									gpu->dmaReadInProgress = 0xC0;
									gpu->dmaBufferIndex = 0;
									gpu->dmaWidthInPixels =
											gpu->fifoBuffer[2] & 0xFFFF;
									gpu->dmaWidthInPixels =
											((gpu->dmaWidthInPixels - 1)
											& 0x3FF) + 1;
									gpu->dmaHeightInPixels = logical_rshift(
											gpu->fifoBuffer[2], 16) & 0xFFFF;
									gpu->dmaHeightInPixels =
											((gpu->dmaHeightInPixels - 1)
											& 0x1FF) + 1;
									gpu->dmaWidthInPixels =
											(gpu->dmaWidthInPixels == 0) ?
											0x400 : gpu->dmaWidthInPixels;
									gpu->dmaHeightInPixels =
											(gpu->dmaHeightInPixels == 0) ?
											0x200 : gpu->dmaHeightInPixels;
									gpu->dmaNeededBytes =
											gpu->dmaWidthInPixels *
											gpu->dmaHeightInPixels;
									gpu->dmaNeededBytes *= 4;
								}
							}
							break;
					}
					break;
			}
			break;
	}
         */
    }
}