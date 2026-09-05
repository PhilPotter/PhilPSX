// SPDX-License-Identifier: GPL-3.0
// psx_dma.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

use philpsx_utility::{EndiannessSwapper, SystemBusHolder};
use crate::{
    dma::{DmaArbiter, DmaArbiterBridge},
};

const CHANNEL_COUNT: usize = 7;

#[repr(usize)]
enum DmaChannel {
    MDEC_IN = 0,
    MDEC_OUT = 1,
    GPU = 2,
    CDROM = 3,
    SPU = 4,
    PIO = 5,
    OTC = 6,
}

/// This struct models orchestration of DMA operations inside the PlayStation.
pub struct PsxDmaArbiter {

    // Global DMA registers.
    dma_control_register: u32,
    dma_interrupt_register: u32,

    // Per-channel registers.
    channel_registers: [u32; 3 * CHANNEL_COUNT],
}

/// Implementation functions for the DMA arbiter component itself.
impl PsxDmaArbiter {

    /// Creates a new DMA arbiter object with the correct initial state.
    pub fn new() -> Self {
        PsxDmaArbiter {

            // Setup global registers.
            dma_control_register: 0,
            dma_interrupt_register: 0,

            // Setup per-channel registers.
            channel_registers: [0; 3 * CHANNEL_COUNT],
        }
    }

    /// This function handles DMA and makes data go to the right place.
    fn handle_dma_transactions(&mut self, bridge: &mut dyn DmaArbiterBridge) {

        // Check if we need to start any DMA requests.
        let mut dma_channel_control: [u32; CHANNEL_COUNT] = [0; CHANNEL_COUNT];
        let mut dma_channel_started: [u32; CHANNEL_COUNT] = [0; CHANNEL_COUNT];
        for i in 0..CHANNEL_COUNT {
            let word = self.channel_registers[i * 3 + 2];
            dma_channel_control[i] = word.swap_endianness();

            if (dma_channel_control[i] & 0x600) >> 9 == 0 {
                // Sync mode 0.
                if (dma_channel_control[i] & 0x11000000) == 0x11000000 {
                    dma_channel_started[i] = 0x1;
                }
            } else {
                // Other.
                dma_channel_started[i] = (dma_channel_control[i] >> 24) & 0x1;
            }
        }

        // Check if each one is actually enabled.
        let temp_control_register = self.dma_control_register.swap_endianness();

        let mut highest_priority = 8;
        let mut highest_priority_channel_so_far = None;
        for i in 0..CHANNEL_COUNT {
            if dma_channel_started[i] == 1 {
                let priority = (temp_control_register >> (i * 4)) & 0x7;
                let enabled = (temp_control_register >> (i * 4)) & 0x8 == 0x8;

                if enabled {
                    if priority <= highest_priority {
                        highest_priority = priority;
                        highest_priority_channel_so_far = Some(i);
                    }
                }
            }
        }

        // Perform DMA transfer itself if one is needed.
        let cpu_cycles = if highest_priority_channel_so_far.is_some() {

            // Set bus holder to DMA arbiter.
            bridge.set_system_bus_holder(self, SystemBusHolder::DMA);

            // Clear bit 28 of channel control register (remember it is
            // stored in little-endian mode).
            let highest_priority_channel_so_far = highest_priority_channel_so_far.unwrap();
            self.channel_registers[highest_priority_channel_so_far * 3 + 2] &= 0xFFFFFFEF;

            // Call correct method depending on channel or mode.
            let cycles = match highest_priority_channel_so_far {

                // MDECin.
                0 => {
                    log::warn!("DMA: MDECin DMA triggered");
                    0
                },

                // MDECout.
                1 => {
                    log::warn!("DMA: MDECout DMA triggered");
                    0
                },

                // GPU DMA.
                2 => self.handle_gpu(bridge),

                // CD-ROM.
                3 => self.handle_cdrom(bridge),

                // SPU.
                4 => {
                    log::warn!("DMA: SPUA DMA triggered");
                    0
                },

                // PIO.
                5 => {
                    log::warn!("DMA: PIO DMA triggered");
                    0
                },

                // OTC DMA.
                6 => self.handle_otc(bridge),

                // Default (0 cycles).
                _ => 0,
            };

            // Clear 24 bit of channel control register (again, remember endianness).
            self.channel_registers[highest_priority_channel_so_far * 3 + 2] &= 0xFFFFFFFE;

            // Set bus holder back to CPU.
            bridge.set_system_bus_holder(self, SystemBusHolder::CPU);

            // Trigger interrupt by masking correct bit (remember endianness).
            let mut temp_interrupt_register = self.dma_interrupt_register.swap_endianness();
            let mut int_mask = (0x00010000 << highest_priority_channel_so_far) | 0x00800000;

            if (temp_interrupt_register & int_mask) == int_mask {
                int_mask = 0x01000000 << highest_priority_channel_so_far;
                temp_interrupt_register |= int_mask;
                self.dma_interrupt_register = temp_interrupt_register.swap_endianness();

                // Set flag in system's Interrupt Status Register.
                bridge.set_dma_interrupt_delay(self, 0);
            }

            cycles
        } else {
            // Do nothing.
            0
        };

        // Commented out in original C version so leaving commented out here too.
        // Can't remember why, probably just because I never finished the damn thing!
        //bridge.append_sync_cycles(self, cpu_cycles);
    }

    /// This function handles GPU DMA transfers.
    fn handle_gpu(&mut self, bridge: &mut dyn DmaArbiterBridge) -> i32 {
        0
    }

    /// This function handles CD-ROM DMA transfers - it assumes a sync mode of 0.
    fn handle_cdrom(&mut self, bridge: &mut dyn DmaArbiterBridge) -> i32 {
        0
    }

    /// This function handles OTC DMA transfers - it assumes a sync mode of 0.
    fn handle_otc(&mut self, bridge: &mut dyn DmaArbiterBridge) -> i32 {
        0
    }
}

/// Implementation functions to be called from anything that understands what
/// a DMA arbiter object is.
impl DmaArbiter for PsxDmaArbiter {
}