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
        motherboard.timer_module.increment_by[0] =
            if motherboard.timer_module.clock_source[0] == 0 ||
                motherboard.timer_module.clock_source[0] == 2 {
                motherboard.timer_module.cpu_cycles_to_sync[0]
            } else {
                let gpu_cycles_to_sync = motherboard.timer_module.gpu_cycles_to_sync[0];
                bridge.gpu_how_many_dotclock_increments(
                    motherboard,
                    gpu_cycles_to_sync
                )
            };
        motherboard.timer_module.increment_by[1] =
            if motherboard.timer_module.clock_source[1] == 0 ||
                motherboard.timer_module.clock_source[1] == 2 {
                motherboard.timer_module.cpu_cycles_to_sync[1]
            } else {
                let gpu_cycles_to_sync = motherboard.timer_module.gpu_cycles_to_sync[1];
                bridge.gpu_how_many_hblank_increments(
                    motherboard,
                    gpu_cycles_to_sync
                )
            };
        motherboard.timer_module.increment_by[2] =
            if motherboard.timer_module.clock_source[2] < 2 {
                motherboard.timer_module.cpu_cycles_to_sync[2]
            } else {
                motherboard.timer_module.cpu_cycles_to_sync[2] / 8
            };

        // Do increment to temporary value.
        motherboard.timer_module.new_value[0] =
            motherboard.timer_module.timer_counter_value[0] + motherboard.timer_module.increment_by[0];
        motherboard.timer_module.new_value[1] =
            motherboard.timer_module.timer_counter_value[1] + motherboard.timer_module.increment_by[1];
        motherboard.timer_module.new_value[2] =
            motherboard.timer_module.timer_counter_value[2] + motherboard.timer_module.increment_by[2];

        // Adjust as required by synchronisation mode.
        // Timer 0:
        if motherboard.timer_module.timer_mode[0] & 0x1 == 0x1 {

            match (motherboard.timer_module.timer_mode[0] >> 1) & 0x3 {

                0 => {
                    if hblank {
                        motherboard.timer_module.new_value[0] =
                            motherboard.timer_module.timer_counter_value[0];
                    }
                },

                1 => {
                    if hblank {
                        motherboard.timer_module.new_value[0] = 0;
                    }
                },

                2 => {
                    motherboard.timer_module.new_value[0] = if hblank {
                        0
                    } else {
                        motherboard.timer_module.timer_counter_value[0]
                    };
                },

                _ => {
                    if !motherboard.timer_module.hblank_happened[0] {
                        motherboard.timer_module.new_value[0] =
                            motherboard.timer_module.timer_counter_value[0];
                    }
                },
            }
        }

        // Timer 1:
        if motherboard.timer_module.timer_mode[1] & 0x1 == 0x1 {

            match (motherboard.timer_module.timer_mode[1] >> 1) & 0x3 {

                0 => {
                    if vblank {
                        motherboard.timer_module.new_value[1] =
                            motherboard.timer_module.timer_counter_value[1];
                    }
                },

                1 => {
                    if vblank {
                        motherboard.timer_module.new_value[1] = 0;
                    }
                },

                2 => {
                    motherboard.timer_module.new_value[1] = if vblank {
                        0
                    } else {
                        motherboard.timer_module.timer_counter_value[1]
                    };
                },

                _ => {
                    if !motherboard.timer_module.vblank_happened[1] {
                        motherboard.timer_module.new_value[1] =
                            motherboard.timer_module.timer_counter_value[1];
                    }
                },
            }
        }

        // Timer 2:
        if motherboard.timer_module.timer_mode[2] & 0x1 == 0x1 {

            match (motherboard.timer_module.timer_mode[2] >> 1) & 0x3 {

                0 | 3 => {
                    motherboard.timer_module.new_value[2] =
                        motherboard.timer_module.timer_counter_value[2];
                },

                _ => (),
            }
        }

        for i in 0..3 {
            motherboard.timer_module.timer_counter_value[i] = motherboard.timer_module.new_value[i];
        }

        // Deal with interrupts and target values.
        for i in 0..3 {

            // Wipe cycles.
            motherboard.timer_module.cpu_cycles_to_sync[i] = 0;
            motherboard.timer_module.gpu_cycles_to_sync[i] = 0;

            // Check for interrupts.
            let mut int_flag = false;
            if motherboard.timer_module.timer_counter_value[i] >=
                motherboard.timer_module.timer_target_value[i] {
                motherboard.timer_module.timer_mode[i] |= 0x800;
                if (motherboard.timer_module.timer_mode[i] & 0x10) == 0x10 {
                    int_flag = true;
                }
            }
            if motherboard.timer_module.timer_counter_value[i] >= 0xFFFF {
                motherboard.timer_module.timer_mode[i] |= 0x1000;
                if (motherboard.timer_module.timer_mode[i] & 0x20) == 0x20 {
                    int_flag = true;
                }
            }
            if int_flag {
                Self::trigger_timer_interrupt(motherboard, i);
            }

            // Reset values here if needed.
            if motherboard.timer_module.timer_counter_value[i] > 0xFFFF &&
                (motherboard.timer_module.timer_mode[1] & 0x8) == 0 {
                motherboard.timer_module.timer_counter_value[i] = 0;
            }
            else if motherboard.timer_module.timer_counter_value[i] >
                motherboard.timer_module.timer_target_value[i] &&
                (motherboard.timer_module.timer_mode[i] & 0x8) == 0x8 {
                motherboard.timer_module.timer_counter_value[i] = 0;
            }
        }
    }

    /// Handle interrupt logic.
    fn trigger_timer_interrupt(motherboard: &mut PsxMotherboard, timer: usize) {

        // One-shot mode.
        if (motherboard.timer_module.timer_mode[timer] & 0x40) == 0 {
            // If this is first IRQ.
            if !motherboard.timer_module.interrupt_happened_once_or_more[timer] {
                // Check for pulse/toggle mode.
                if (motherboard.timer_module.timer_mode[timer] & 0x80) == 0 {
                    // Just set bit 10 to 0 and be done with it,
                    // triggering IRQ as well.
                    motherboard.timer_module.timer_mode[timer] &= 0xFFFFFBFF_u32 as i32;
                    motherboard.timers_interrupt_delay[timer] = 0;
                    motherboard.timers_interrupt_counter[timer] = 0;
                    motherboard.timer_module.interrupt_happened_once_or_more[timer] = true;
                } else {
                    // Invert flag, triggering IRQ if it is then 0.
                    if (motherboard.timer_module.timer_mode[timer] & 0x400) == 0x400 {
                        // Flip to 0 and trigger interrupt.
                        motherboard.timer_module.timer_mode[timer] &= 0xFFFFFBFF_u32 as i32;
                        motherboard.timers_interrupt_delay[timer] = 0;
                        motherboard.timers_interrupt_counter[timer] = 0;
                        motherboard.timer_module.interrupt_happened_once_or_more[timer] = true;
                    } else {
                        // Flip back to 1 and do nothing.
                        motherboard.timer_module.timer_mode[timer] |= 0x400;
                    }
                }
            }
        } else { // Repeated mode - don't check for one-shot flag.
            // Check for pulse/toggle mode.
            if (motherboard.timer_module.timer_mode[timer] & 0x80) == 0 {
                // Just set bit 10 to 0 and be done with it,
                // triggering IRQ as well.
                motherboard.timer_module.timer_mode[timer] &= 0xFFFFFBFF_u32 as i32;
                motherboard.timers_interrupt_delay[timer] = 0;
                motherboard.timers_interrupt_counter[timer] = 0;
            } else {
                // Invert flag, triggering IRQ if it is then 0.
                if (motherboard.timer_module.timer_mode[timer] & 0x400) == 0x400 {
                    // Flip to 0 and trigger interrupt.
                    motherboard.timer_module.timer_mode[timer] &= 0xFFFFFBFF_u32 as i32;
                    motherboard.timers_interrupt_delay[timer] = 0;
                    motherboard.timers_interrupt_counter[timer] = 0;
                } else {
                    // Flip back to 1 and do nothing.
                    motherboard.timer_module.timer_mode[timer] |= 0x400;
                }
            }
        }
    }

    /// Read from the specified timer's counter value register.
    pub fn read_counter_value(
        motherboard: &mut PsxMotherboard,
        bridge: &mut dyn MotherboardBridge,
        timer: usize
    ) -> i32 {

        // Catch up to current cycle count.
        Self::resync(motherboard, bridge);

        // If in pulse mode, set bit 10 back to 1 now.
        if (motherboard.timer_module.timer_mode[timer] & 0x80) == 0 {
            motherboard.timer_module.timer_mode[timer] |= 0x400;
        }

        swap_endianness(motherboard.timer_module.timer_counter_value[timer])
    }

    /// Read from the specified timer's mode register.
    pub fn read_mode(
        motherboard: &mut PsxMotherboard,
        bridge: &mut dyn MotherboardBridge,
        timer: usize,
        bit_11_12_override: bool
    ) -> i32 {

        // Catch up to current cycle count.
        Self::resync(motherboard, bridge);

        // If in pulse mode, set bit 10 back to 1 now.
        if (motherboard.timer_module.timer_mode[timer] & 0x80) == 0 && !bit_11_12_override {
            motherboard.timer_module.timer_mode[timer] |= 0x400;
        }

        // Store mode register in temporary variable.
        let ret_val = motherboard.timer_module.timer_mode[timer];

        // Reset bits 11 and 12 if not overridden, as these relate to
        // reaching certain values.
        if !bit_11_12_override {
            motherboard.timer_module.timer_mode[timer] &= 0xFFFFE7FF_u32 as i32;
        }

        swap_endianness(ret_val)
    }

    /// Read from the specified timer's target value register.
    pub fn read_target_value(&mut self, timer: usize) -> i32 {

        // If in pulse mode, set bit 10 back to 1 now.
        if (self.timer_mode[timer] & 0x80) == 0 {
            self.timer_mode[timer] |= 0x400;
        }

        swap_endianness(self.timer_target_value[timer])
    }

    /// Write to the specified timer's counter value register.
    pub fn write_counter_value(
        motherboard: &mut PsxMotherboard,
        bridge: &mut dyn MotherboardBridge,
        timer: usize,
        value: i32
    ) {
        Self::resync(motherboard, bridge);
        let value = swap_endianness(value);
        motherboard.timer_module.timer_counter_value[timer] = 0xFFFF & value;
    }

    /// Write to the specified timer's mode register.
    pub fn write_mode(
        motherboard: &mut PsxMotherboard,
        bridge: &mut dyn MotherboardBridge,
        timer: usize,
        value: i32
    ) {
        Self::resync(motherboard, bridge);
        let mut value = swap_endianness(value);

        // Set bit 10 to turn off interrupt request.
        value |= 0x400;

        // Set bits 13-15 to 0.
        value &= 0xFFFF1FFF_u32 as i32;

        // Reset hblank and vblank happened markers.
        motherboard.timer_module.hblank_happened[timer] = false;
        motherboard.timer_module.vblank_happened[timer] = false;

        // Reset one-shot marker.
        motherboard.timer_module.interrupt_happened_once_or_more[timer] = false;

        motherboard.timer_module.timer_mode[timer] = value;

        // Reset counter value.
        motherboard.timer_module.timer_counter_value[timer] = 0;
    }

    /// Write to the specified timer's target value register.
    pub fn write_target_value(
        motherboard: &mut PsxMotherboard,
        bridge: &mut dyn MotherboardBridge,
        timer: usize,
        value: i32
    ) {
        Self::resync(motherboard, bridge);
        let value = swap_endianness(value);
        motherboard.timer_module.timer_target_value[timer] = 0xFFFF & value;
    }
}

/// This utility function swaps the endianness of a signed word for us.
#[inline(always)]
fn swap_endianness(word: i32) -> i32 {
    (word << 24) | ((word & 0xFF00) << 8) | (word & 0xFF0000) >> 8 | word.logical_rshift(24)
}