// SPDX-License-Identifier: GPL-3.0
// psx_timer.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

use philpsx_utility::LogicalRightShifter;
use crate::{
    motherboard::{
        MotherboardBridge,
        psx_motherboard::PsxMotherboard,
    },
};

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

    /// Resync all timers to the current point. To make this work with the motherboard
    /// bridge while in a separate file for a struct contained within it, I've made it a
    pub fn resync(motherboard: &mut PsxMotherboard, bridge: &mut dyn MotherboardBridge) {

        // Get HBlank and VBlank status.
        let hblank = bridge.gpu_is_in_hblank(motherboard);
        let vblank = bridge.gpu_is_in_vblank(motherboard);

        if hblank {
            for i in 0..3 {
                motherboard.timer_module.hblank_happened[i] = true;
            }
        }
        if vblank {
            for i in 0..3 {
                motherboard.timer_module.vblank_happened[i] = true;
            }
        }

        // Convert sync figure to GPU cycles in case its needed.
        for i in 0..3 {
            motherboard.timer_module.gpu_cycles_to_sync[i] +=
                ((motherboard.timer_module.cpu_cycles_to_sync[i] as f32) / 7.0 * 11.0) as i32;

            // Add in top up values.
            motherboard.timer_module.cpu_cycles_to_sync[i] += motherboard.timer_module.cpu_topup[i];
            motherboard.timer_module.gpu_cycles_to_sync[i] += motherboard.timer_module.gpu_topup[i];
        }

        // Get clock source for all three timers.
        for i in 0..3 {
            motherboard.timer_module.clock_source[i] =
                (motherboard.timer_module.timer_mode[i] >> 8) & 0x3;
        }

        // Calculate future top up values.
        motherboard.timer_module.cpu_topup[0] = 0;
        motherboard.timer_module.gpu_topup[0] =
            if motherboard.timer_module.clock_source[0] == 0 ||
                motherboard.timer_module.clock_source[0] == 2 {
                0
            } else {
                let gpu_cycles_to_sync = motherboard.timer_module.gpu_cycles_to_sync[0];
                bridge.gpu_how_many_dotclock_gpu_cycles_left(
                    motherboard,
                    gpu_cycles_to_sync
                )
            };
        motherboard.timer_module.cpu_topup[1] = 0;
        motherboard.timer_module.gpu_topup[1] =
            if motherboard.timer_module.clock_source[1] == 0 ||
                motherboard.timer_module.clock_source[1] == 2 {
                0
            } else {
                let gpu_cycles_to_sync = motherboard.timer_module.gpu_cycles_to_sync[1]; 
                bridge.gpu_how_many_hblank_gpu_cycles_left(
                    motherboard,
                    gpu_cycles_to_sync
                )
            };
        motherboard.timer_module.cpu_topup[2] =
            if motherboard.timer_module.clock_source[2] < 2 {
                0
            } else {
                motherboard.timer_module.cpu_cycles_to_sync[2] % 8
            };
        motherboard.timer_module.gpu_topup[2] = 0;

        // Get increment count for all three timers.
    }
}