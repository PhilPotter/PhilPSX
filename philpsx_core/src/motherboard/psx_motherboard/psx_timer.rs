// SPDX-License-Identifier: GPL-3.0
// psx_timer.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

use crate::motherboard::MotherboardBridge;

pub struct PsxTimerModule {

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

/// Implementation functions for the timer module.
impl PsxTimerModule {

    /// Creates a new timer module object with the correct initial state.
    pub fn new() -> Self {
        PsxTimerModule {

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
        }
    }

    /// This tells the timer to add some cycles to the count it needs to sync by.
    pub fn append_sync_cycles(&mut self, cycles: i32) {
        self.cpu_cycles_to_sync
            .iter_mut()
            .for_each(|these_cycles| *these_cycles += cycles)
    }

    /// Read from the specified timer's counter value register.
    pub fn read_counter_value(&mut self, bridge: &mut dyn MotherboardBridge, timer: isize) {

    }

    /// Resync all timers to the current point.
    pub fn resync(&mut self, bridge: &mut dyn MotherboardBridge) {

        // Get HBlank and VBlank status.
    }
}