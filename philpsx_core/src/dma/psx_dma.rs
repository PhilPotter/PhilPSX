// SPDX-License-Identifier: GPL-3.0
// psx_dma.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

use philpsx_utility::{EndiannessSwapper, SystemBusHolder};
use crate::{
    dma::{DmaArbiter, DmaArbiterBridge},
};

/// Channel values.
const CHANNEL_COUNT: usize = 7;
const MDEC_IN: usize = 0;
const MDEC_OUT: usize = 1;
const GPU: usize = 2;
const CDROM: usize = 3;
const SPU: usize = 4;
const PIO: usize = 5;
const OTC: usize = 6;


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

        // Get DMA base address and correct endianness.
        let mut base_address = self.channel_registers[GPU * 3];
        base_address = base_address.swap_endianness();
        base_address = bridge.virtual_to_physical(self, base_address);

        // Get block control and correct endianness.
        let mut block_control = self.channel_registers[GPU * 3 + 1];
        block_control = block_control.swap_endianness();

        // Get channel control register and correct endianness.
        let mut channel_control = self.channel_registers[GPU * 3 + 2];
        channel_control = channel_control.swap_endianness();

        // Act according to specified mode.
        match (channel_control & 0x600) >> 9 {

            1 => {
                // Store base address to temp variable.
                let mut temp_address = base_address;

                // Get block size in words.
                let mut block_size = 0xFFFF & block_control;
                if block_size == 0 {
                    block_size = 0x10000;
                }

                // Get number of blocks.
                let mut num_of_blocks = 0xFFFF & (block_control >> 16);
                if num_of_blocks == 0 {
                    num_of_blocks = 0x10000;
                }

                // Calculate total number of words and cycles.
                let num_of_words = block_size * num_of_blocks;

                // Read from or write to GPU depending on channel control register.
                let write_to_gpu = channel_control & 0x1 == 0x1;
                let backward = (channel_control >> 1) & 0x1 == 0x1;
                if write_to_gpu {
                    // Write to GPU from RAM.
                    for _ in 0..num_of_words {
                        let word = bridge.read_word(self, temp_address);
                        bridge.gpu_submit_to_gp0(self, word);

                        if backward {
                            temp_address -= 4;
                        } else {
                            temp_address += 4;
                        }
                    }
                } else {
                    // Read from GPU to RAM.
                    for _ in 0..num_of_words {
                        let word = bridge.gpu_read_response(self);
                        bridge.write_word(self, temp_address, word);

                        if backward {
                            temp_address -= 4;
                        } else {
                            temp_address += 4;
                        }
                    }
                }

                // Set BA to 0 directly in register (little-endian) for speed.
                self.channel_registers[GPU * 3 + 1] &= 0xFFFF0000;

                // Return the number of DMA cycles.
                num_of_words as i32
            },

            2 => {
                // Iterate over linked list, sending commands to GPU GP0.
                let mut next_address = base_address;
                let mut cycle_count = 0;
                loop {
                    // Store as current address.
                    let current_address = next_address;

                    // Read word into next_address, update base address register
                    // and correct endianness.
                    next_address = bridge.read_word(self, next_address);
                    self.channel_registers[GPU * 3] = next_address & 0xFFFFFF00;
                    next_address = next_address.swap_endianness();

                    // Get number of words we need and mask them from next_address.
                    let num_of_words = (next_address & 0xFF000000) >> 24;
                    next_address &= 0xFFFFFF;

                    // Iterate current block and send commands.
                    for i in 1..=num_of_words {
                        let word = bridge.read_word(self, current_address + i * 4);
                        bridge.gpu_submit_to_gp0(self, word);
                        cycle_count += 1;
                    }

                    // This was originally a do-while loop in the C version, so
                    // invert this check at the end of the loop body and break on
                    // true.
                    if next_address == 0xFFFFFF {
                        break;
                    }
                }

                // Return the number of DMA cycles.
                cycle_count
            },

            _ => {
                panic!("DMA: This transfer mode is not implemented for GPU DMA");
            },
        }
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