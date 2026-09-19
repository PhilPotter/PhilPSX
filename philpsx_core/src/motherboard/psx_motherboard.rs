// SPDX-License-Identifier: GPL-3.0
// psx_motherboard.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

use std::{
    error::Error,
    ffi::OsStr,
    fs::File,
    io::Read,
};
use philpsx_utility::{EndiannessSwapper, LogicalRightShifter, SystemBusHolder};
use crate::{
    motherboard::{
        Motherboard,
        MotherboardBridge,
        psx_motherboard::psx_timer::PsxTimerModule,
    },
};

/// Size of the RAM area in bytes.
const RAM_SIZE: usize = 2097152;

/// Size of the scratchpad area in bytes.
const SCRATCHPAD_SIZE: usize = 1024;

/// Size of the BIOS area in bytes.
const BIOS_SIZE: usize = 524288;

/// Private timer module, just to keep the code cleaner/more separate than in the C version.
mod psx_timer;

/// This struct models the central 'motherboard' of the PlayStation, storing things
/// like the RAM, timers and others.
pub struct PsxMotherboard {

    // 2MiB of RAM (heap allocated).
    ram: Vec<u8>,

    // 1 KiB of scratchpad area (heap allocated).Strictly speaking
    // this is inside the CPU in the real hardware, but makes more
    // sense to put it here.
    scratchpad: Vec<u8>,

    // 512 KiB of BIOS (heap allocated). This stores the BIOS once
    // it is copied into memory. Store in unsigned byte form despite
    // original C version using i8 (because it was in turn based on)
    // the Java version that used byte (which is signed).
    bios: Vec<u8>,

    // Timer module.
    timer_module: PsxTimerModule,

    // Register declarations.
    cache_control_reg: u32,
    interrupt_status_reg: u32,
    interrupt_mask_reg: u32,
    expansion1_base_address: u32,
    expansion2_base_address: u32,
    expansion1_delay_size: u32,
    expansion2_delay_size: u32,
    expansion3_delay_size: u32,
    bios_rom_delay_size: u32,
    spu_delay_size: u32,
    cdrom_delay_size: u32,
    common_delay: u32,
    ram_size: u32,
    bios_post: u8,

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
    cdrom_interrupt_number: u8,
    cdrom_interrupt_enabled: bool,
    interrupt_cycles: i32,
}

/// Implementation functions for the motherboard itself.
impl PsxMotherboard {

    /// Creates a new motherboard object with the correct initial state.
    pub fn new(bios_path: &OsStr) -> Result<Self, Box<dyn Error>> {

        let mut motherboard = PsxMotherboard {

            // Setup memory areas.
            ram: vec![0; RAM_SIZE],
            scratchpad: vec![0; SCRATCHPAD_SIZE],
            bios: vec![0; BIOS_SIZE],

            // Setup timer module.
            timer_module: PsxTimerModule::new(),

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
        };

        // Populate BIOS with passed in data.
        motherboard.load_bios_data_to_memory(bios_path)?;

        Ok(motherboard)
    }

    /// Copies the bytes from the passed in slice to our BIOS memory area.
    fn load_bios_data_to_memory(&mut self, bios_path: &OsStr) -> Result<(), Box<dyn Error>> {

        let mut bios_file = File::open(bios_path)?;
        Ok(bios_file.read_exact(self.bios.as_mut_slice())?)
    }
}

/// Implementation functions to be called from anything that understands what
/// a Motherboard object is.
impl Motherboard for PsxMotherboard {

    /// This function appends a cycle count to the system count.
    fn append_sync_cycles(&mut self, bridge: &mut dyn MotherboardBridge, cycles: i32) {

        self.interrupt_cycles += cycles;
        bridge.gpu_append_sync_cycles(self, cycles);
        bridge.controllers_append_sync_cycles(self, cycles);
        self.timer_module.append_sync_cycles(cycles);
    }

    /// This function determines the number of stall cycles to use.
    fn how_how_many_stall_cycles(&self, address: u32) -> i32 {

        // Mask address to clamp it to word boundary.
        let temp_address = address & 0xFFFFFFFC;

        // Check which area the address is in and set cycles accordingly.
        match temp_address {

            // RAM.
            0..0x200000 => 6,

            // BIOS.
            0x1FC00000..0x1FC80000 => 1,

            // Cache control register.
            0xFFFE0130 => 1,

            // Otherwise, use standard delay of most registers.
            _ => 4,
        }
    }

    /// This function determines if an address is OK to increment.
    fn ok_to_increment(&self, address: u32) -> bool {

        !(0x1F801800..=0x1F801803).contains(&address)
    }

    /// This function determines if the scratchpad is enabled.
    fn scratchpad_enabled(&self) -> bool {

        self.cache_control_reg.swap_endianness() & 0x88 == 0x88
    }

    /// This function determines if the instruction cache is enabled.
    fn instruction_cache_enabled(&self) -> bool {

        self.cache_control_reg.swap_endianness() & 0x800 == 0x800
    }

    /// This function reads a byte from the system address space.
    fn read_byte(&mut self, bridge: &mut dyn MotherboardBridge, address: u32) -> u8 {

        // Shift value and index, should we need them.
        let bits_1_0_index = address & 0x3;
        let shift = 24 - ((bits_1_0_index & 0x3) * 8);

        match address {

            // RAM.
            0..0x200000 => self.ram[address as usize],

            // BIOS ROM.
            0x1FC00000..0x1FC80000 => self.bios[(address - 0x1FC00000) as usize],

            // Everything else.
            _ => {
                match address {

                    // Expansion Region 1 - do nothing for now.
                    0x1F000000..0x1F800000 => 0,

                    // Scratchpad - read from data cache scratchpad if enabled.
                    0x1F800000..0x1F800400 => if self.scratchpad_enabled() {
                        self.scratchpad[(address - 0x1F800000) as usize]
                    } else {
                        0
                    },

                    // I/O ports.
                    0x1F801000..0x1F802000 => {

                        match address {

                            // Expansion 1 base address.
                            0x1F801000..0x1F801004 => {
                                (self.expansion1_base_address >> shift) as u8
                            },

                            // Expansion 2 base address.
                            0x1F801004..0x1F801008 => {
                                (self.expansion2_base_address >> shift) as u8
                            },

                            // Expansion 1 delay size.
                            0x1F801008..0x1F80100C => {
                                (self.expansion1_delay_size >> shift) as u8
                            },

                            // Expansion 3 delay size.
                            0x1F80100C..0x1F801010 => {
                                (self.expansion3_delay_size >> shift) as u8
                            },

                            // BIOS ROM delay size.
                            0x1F801010..0x1F801014 => {
                                (self.bios_rom_delay_size >> shift) as u8
                            },

                            // SPU delay size.
                            0x1F801014..0x1F801018 => {
                                (self.spu_delay_size >> shift) as u8
                            },

                            // CD-ROM delay size.
                            0x1F801018..0x1F80101C => {
                                (self.cdrom_delay_size >> shift) as u8
                            },

                            // Expansion 2 delay size.
                            0x1F80101C..0x1F801020 => {
                                (self.expansion2_delay_size >> shift) as u8
                            },

                            // Common delay.
                            0x1F801020..0x1F801024 => {
                                (self.common_delay >> shift) as u8
                            },

                            // Controller I/O.
                            0x1F801040..0x1F801050 => {
                                bridge.controllers_read_byte(self, address as u8)
                            },

                            // RAM size.
                            0x1F801060..0x1F801064 => {
                                (self.ram_size >> shift) as u8
                            },

                            // Interrupt status register.
                            0x1F801070..0x1F801074 => {
                                (self.interrupt_status_reg >> shift) as u8
                            },

                            // Interrupt mask register.
                            0x1F801074..0x1F801078 => {
                                (self.interrupt_mask_reg >> shift) as u8
                            },

                            // DMA read.
                            0x1F801080..0x1F801100 => {
                                bridge.dma_read_byte(self, address)
                            },

                            // Timer 0 counter value.
                            0x1F801100..0x1F801104 => {
                                PsxTimerModule::read_counter_value(
                                    self,
                                    bridge,
                                    0
                                ).logical_rshift(shift as i32) as u8
                            },

                            // Timer 0 mode value.
                            0x1F801104..0x1F801108 => {
                                PsxTimerModule::read_mode(
                                    self,
                                    bridge,
                                    0,
                                    false
                                ).logical_rshift(shift as i32) as u8
                            },

                            // Timer 0 target value.
                            0x1F801108..0x1F80110C => {
                                self.timer_module.read_target_value(
                                    0
                                ).logical_rshift(shift as i32) as u8
                            },

                            // Timer 1 counter value.
                            0x1F801110..0x1F801114 => {
                                PsxTimerModule::read_counter_value(
                                    self,
                                    bridge,
                                    1
                                ).logical_rshift(shift as i32) as u8
                            },

                            // Timer 1 mode value.
                            0x1F801114..0x1F801118 => {
                                PsxTimerModule::read_mode(
                                    self,
                                    bridge,
                                    1,
                                    false
                                ).logical_rshift(shift as i32) as u8
                            },

                            // Timer 1 target value.
                            0x1F801118..0x1F80111C => {
                                self.timer_module.read_target_value(
                                    1
                                ).logical_rshift(shift as i32) as u8
                            },

                            // Timer 2 counter value.
                            0x1F801120..0x1F801124 => {
                                PsxTimerModule::read_counter_value(
                                    self,
                                    bridge,
                                    2
                                ).logical_rshift(shift as i32) as u8
                            },

                            // Timer 2 mode value.
                            0x1F801124..0x1F801128 => {
                                PsxTimerModule::read_mode(
                                    self,
                                    bridge,
                                    2,
                                    false
                                ).logical_rshift(shift as i32) as u8
                            },

                            // Timer 2 target value.
                            0x1F801128..0x1F80112C => {
                                self.timer_module.read_target_value(
                                    2
                                ).logical_rshift(shift as i32) as u8
                            },

                            // CD-ROM.
                            0x1F801800..0x1F801804 => {
                                match bits_1_0_index {
                                    0 => bridge.cdrom_drive_read_1800(self),
                                    1 => bridge.cdrom_drive_read_1801(self),
                                    2 => bridge.cdrom_drive_read_1802(self),
                                    3 => bridge.cdrom_drive_read_1803(self),
                                    _ => 0,
                                }
                            },

                            // GPU response.
                            0x1F801810..0x1F801814 => {
                                (bridge.gpu_read_response(self) >> shift) as u8
                            },

                            // GPU status.
                            0x1F801814..0x1F801818 => {
                                (bridge.gpu_read_status(self) >> shift) as u8
                            },

                            // SPU read.
                            0x1F801C00..0x1F802000 => {
                                // Fake SPU read.
                                let adjusted_address = address - 0x1F801C00;
                                bridge.spu_read_byte(self, adjusted_address)
                            },

                            _ => 0,
                        }
                    },

                    // Expansion region 2 (I/O ports).
                    0x1F802000..0x1F803000 => {
                        // Read from BIOS post register.
                        if address == 0x1F802041 {
                            self.bios_post
                        } else {
                            0
                        }
                    },

                    // Expansion region 3 (multipurpose).
                    0x1FA00000..0x1FC00000 => {
                        // Do nothing for now.
                        0
                    },

                    // I/O ports (cache control).
                    0xFFFE0000..0xFFFE0200 => {
                        match address {

                            // Cache control register.
                            0xFFFE0130..0xFFFE0134 => {
                                (self.cache_control_reg >> shift) as u8
                            },

                            _ => 0,
                        }
                    },

                    _ => 0,
                }
            },
        }
    }

    /// This function reads a word from the system address space.
    fn read_word(&mut self, bridge: &mut dyn MotherboardBridge, address: u32) -> u32 {
        0
    }

    /// This function writes a byte to the system address space.
    fn write_byte(&mut self, bridge: &mut dyn MotherboardBridge, address: u32, value: u8) {
    }

    /// This function writes a word to the system address space.
    fn write_word(&mut self, bridge: &mut dyn MotherboardBridge, address: u32, value: u32) {
    }

    /// This function is used increment interrupt counters and trigger
    /// timer updates and GPU updates to be done.
    fn increment_interrupt_counters(&mut self, bridge: &mut dyn MotherboardBridge) {

        // Resync all timers before doing anything else. Originally
        // this was a separate call from the CPU in the C version.
        PsxTimerModule::resync(self, bridge);

        // Now execute the required GPU cycles.
        bridge.gpu_execute_gpu_cycles(self);

        let interrupt_cycles_i64 = self.interrupt_cycles as i64;
        self.gpu_interrupt_counter += interrupt_cycles_i64;
        self.dma_interrupt_counter += interrupt_cycles_i64;
        self.cdrom_interrupt_counter += interrupt_cycles_i64;
        for i in 0..3 {
            self.timers_interrupt_counter[i] += interrupt_cycles_i64;
        }

        // Handle GPU.
        if self.gpu_interrupt_delay != -1 &&
            self.gpu_interrupt_counter > self.gpu_interrupt_delay {
            // Set in little-endian mode for speed.
            self.interrupt_status_reg |= 0x01000000;
            self.gpu_interrupt_delay = -1;
        }

        // Handle DMA.
        if self.dma_interrupt_delay != -1 &&
            self.dma_interrupt_counter > self.dma_interrupt_delay {
            // Set in little-endian mode for speed.
            self.interrupt_status_reg |= 0x08000000;
            self.dma_interrupt_delay = -1;
        }

        // Handle CD-ROM.
        if self.cdrom_interrupt_delay != -1 &&
            self.cdrom_interrupt_counter > self.cdrom_interrupt_delay {
            // Set in little-endian mode for speed.
            if self.cdrom_interrupt_enabled {
                self.interrupt_status_reg |= 0x04000000;
            }

            // Set this interrupt handler back to inactive.
            self.cdrom_interrupt_delay = -1;

            // Also set interrupt number in CD-ROM interrupt flag register.
            let cdrom_interrupt_number = self.cdrom_interrupt_number;
            bridge.cdrom_drive_set_interrupt_number(self, cdrom_interrupt_number);
        }

        // Handle Timer 0.
        if self.timers_interrupt_delay[0] != -1 &&
            self.timers_interrupt_counter[0] > self.timers_interrupt_delay[0] {
            // Set in little-endian mode for speed.
            self.interrupt_status_reg |= 0x10000000;
            self.timers_interrupt_delay[0] = -1;
        }

        // Handle Timer 1.
        if self.timers_interrupt_delay[1] != -1 &&
            self.timers_interrupt_counter[1] > self.timers_interrupt_delay[1] {
            // Set in little-endian mode for speed.
            self.interrupt_status_reg |= 0x20000000;
            self.timers_interrupt_delay[1] = -1;
        }

        // Handle Timer 2.
        if self.timers_interrupt_delay[2] != -1 &&
            self.timers_interrupt_counter[2] > self.timers_interrupt_delay[2] {
            // Set in little-endian mode for speed.
            self.interrupt_status_reg |= 0x40000000;
            self.timers_interrupt_delay[2] = -1;
        }

        // Reset interrupt cycles.
        self.interrupt_cycles = 0;
    }

    /// This function returns a mutable reference to main memory.
    fn get_ram_reference(&mut self) -> &mut [u8] {
        &mut self.ram
    }

    /// This function triggers a chunk copy from the CD-ROM drive
    /// to main memory.
    fn cdrom_drive_chunk_copy(
        &mut self,
        bridge: &mut dyn MotherboardBridge,
        starting_byte_address: u32,
        num_of_bytes: u32
    ) {
        bridge.cdrom_drive_chunk_copy(self, starting_byte_address, num_of_bytes);
    }

    /// This function is used to specify if the CD-ROM drive interrupt is actually enabled.
    fn set_cdrom_interrupt_enabled(&mut self, enabled: bool) {
        self.cdrom_interrupt_enabled = enabled;
    }

    /// This function is used to specify the CD-ROM drive interrupt delay.
    fn set_cdrom_interrupt_delay(&mut self, delay: i32) {
        self.cdrom_interrupt_delay = delay as i64;
        self.cdrom_interrupt_counter = 0;
    }

    /// This function is used to set the CD-ROM drive interrupt number.
    fn set_cdrom_interrupt_number(&mut self, number: u8) {
        self.cdrom_interrupt_number = number;
    }

    /// This function is used to set the GPU interrupt delay.
    fn set_gpu_interrupt_delay(&mut self, delay: i32) {
        self.gpu_interrupt_delay = delay as i64;
        self.gpu_interrupt_counter = 0;
    }

    /// This function is used to set the DMA interrupt delay.
    fn set_dma_interrupt_delay(&mut self, delay: i32) {
        self.dma_interrupt_delay = delay as i64;
        self.dma_interrupt_counter = 0;
    }

    /// This function is used to set the system bus holder.
    fn set_system_bus_holder(
        &mut self,
        bridge: &mut dyn MotherboardBridge,
        holder: SystemBusHolder
    ) {
        bridge.cpu_set_system_bus_holder(self, holder);
    }

    /// This function calls the CPU to convert a virtual address
    /// to a physical address.
    fn virtual_to_physical(&mut self, bridge: &mut dyn MotherboardBridge, address: u32) -> u32 {
        bridge.cpu_virtual_to_physical(self, address)
    }

    /// This function submits GP0 commands to the GPU.
    fn gpu_submit_to_gp0(&mut self, bridge: &mut dyn MotherboardBridge, word: u32) {
        bridge.gpu_submit_to_gp0(self, word);
    }

    /// This function reads GPU responses.
    fn gpu_read_response(&mut self, bridge: &mut dyn MotherboardBridge) -> u32 {
        bridge.gpu_read_response(self)
    }
}