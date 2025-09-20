// SPDX-License-Identifier: GPL-3.0
// psx_motherboard.rs - Copyright Phillip Potter, 2025, under GPLv3 only.

use super::Motherboard;

/// Size of the RAM area in bytes.
const RAM_SIZE: usize = 2097152;

/// Size of the scratchpad area in bytes.
const SCRATCHPAD_SIZE: usize = 1024;

/// Size of the BIOS area in bytes.
const BIOS_SIZE: usize = 524288;

/// This struct models the central 'motherboard' of the PlayStaton, storing things
/// like the RAM, timers and others.
pub struct PsxMotherboard {

    // 2MiB of RAM (heap allocated).
    ram: Vec<i8>,

    // 1 KiB of scratchpad area (heap allocated).Strictly speaking
    // this is inside the CPU in the real hardware, but makes more
    // sense to put it here.
    scratchpad: Vec<i8>,

    // 512 KiB of BIOS (heap allocated). This stores the BIOS once
    // it is copied into memory.
    bios: Vec<i8>,

    // Register declarations.
    cache_control_reg: i32,
    interrupt_status_reg: i32,
    interrupt_mask_reg: i32,
    expansion1_base_address: i32,
    expansion2_base_address: i32,
    expansion1_delay_size: i32,
    expansion2_delay_size: i32,
    expansion3_delay_size: i32,
    bios_rom_delay_size: i32,
    spu_delay_size: i32,
    cdrom_delay_size: i32,
    common_delay: i32,
    ram_size: i32,
    bios_post: i8,

    // These interrupt-specific registers allow us to delay interrupts so
    // they trigger at the proper time.
    gpu_interrupt_delay: i64,
    dma_interrupt_delay: i64,
    cdrom_interrupt_delay: i64,
    gpu_interrupt_counter: i64,
    dma_interrupt_counter: i64,
    cdrom_interrupt_counter: i64,
    timers_interrupt_delay: [i64; 3],
    timers_interrupt_counter: [i64; 3],
    cdrom_interrupt_number: i32,
    cdrom_interrupt_enabled: bool,
    interrupt_cycles: i32,

    // Timer specific fields below. In the C version, these were contained
    // within a separate heap-allocated object.

    // Variables for the three timers.
    timer_mode: [i32; 3],
    timer_counter_value: [i32; 3],
    timer_target_value: [i32; 3],
    clock_source: [i32; 3],
    increment_by: [i32; 3],
    new_value: [i32; 3],
    interrupt_happened_once_or_more: [bool; 3],

    // Variables to track CPU cycles and GPU cycles.
    cpu_cycles_to_sync: [i32; 3],
    gpu_cycles_to_sync: [i32; 3],
    cpu_topup: [i32; 3],
    gpu_topup: [i32; 3],
    hblank_happened: [bool; 3],
    vblank_happened: [bool; 3],
}

/// Implementation functions for the motherboard itself.
impl PsxMotherboard {

    /// Creates a new motherboard object with the correct initial state.
    pub fn new(bios_data: &[i8]) -> Self {

        let mut motherboard = PsxMotherboard {

            // Setup memory areas.
            ram: vec![0; RAM_SIZE],
            scratchpad: vec![0; SCRATCHPAD_SIZE],
            bios: vec![0; BIOS_SIZE],

            // Setup registers.
            cache_control_reg: 0,
            interrupt_status_reg: 0,
            interrupt_mask_reg: 0,
            expansion1_base_address: 0,
            expansion2_base_address: 0,
            expansion1_delay_size: 0,
            expansion2_delay_size: 0,
            expansion3_delay_size: 0,
            bios_rom_delay_size: 0,
            spu_delay_size: 0,
            cdrom_delay_size: 0,
            common_delay: 0,
            ram_size: 0,
            bios_post: 0,

            // Setup interrupt delays.
            gpu_interrupt_delay: -1,
            dma_interrupt_delay: -1,
            cdrom_interrupt_delay: -1,
            gpu_interrupt_counter: 0,
            dma_interrupt_counter: 0,
            cdrom_interrupt_counter: 0,
            timers_interrupt_delay: [-1; 3],
            timers_interrupt_counter: [0; 3],
            cdrom_interrupt_number: 0,
            cdrom_interrupt_enabled: false,
            interrupt_cycles: 0,

            // Setup timer variables.
            timer_mode: [0; 3],
            timer_counter_value: [0; 3],
            timer_target_value: [0; 3],
            clock_source: [0; 3],
            increment_by: [0; 3],
            new_value: [0; 3],
            interrupt_happened_once_or_more: [false; 3],

            // Setup CPU cycles and GPU cycles variables.
            cpu_cycles_to_sync: [0; 3],
            gpu_cycles_to_sync: [0; 3],
            cpu_topup: [0; 3],
            gpu_topup: [0; 3],
            hblank_happened: [false; 3],
            vblank_happened: [false; 3],
        };

        // Populate BIOS with passed in data.
        motherboard.load_bios_data_to_memory(bios_data);

        motherboard
    }

    /// Copies the bytes from the passed in slice to our BIOS memory area.
    fn load_bios_data_to_memory(&mut self, bios_data: &[i8]) {
        self.bios.copy_from_slice(bios_data);
    }
}

/// Implementation functions to be called from anything that understands what
/// a Motherboard object is.
impl Motherboard for PsxMotherboard {

}