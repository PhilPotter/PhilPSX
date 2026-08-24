// SPDX-License-Identifier: GPL-3.0
// motherboard.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

/// This module contains the default motherboard implementation. There
/// may be others in future.
pub mod psx_motherboard;

/// This trait provides an implementation-opaque way of calling motherboard
/// methods from elsewhere in the system.
pub trait Motherboard {

    /// The CPU must call this to append a cycle count to the system count.
    fn append_sync_cycles(&mut self, bridge: &mut dyn MotherboardBridge, cycles: i32);

    /// The CPU must call this to determine the number of stall cycles to use.
    fn how_how_many_stall_cycles(&self, address: u32) -> i32;

    /// The CPU must call this to determine if an address is OK to increment.
    fn ok_to_increment(&self, address: u32) -> bool;

    /// The CPU must call this to determine if the scratchpad is enabled.
    fn scratchpad_enabled(&self) -> bool;

    /// The CPU must call this to determine if the instruction cache is enabled.
    fn instruction_cache_enabled(&self) -> bool;

    /// The CPU must call this to read a byte from the system address space.
    fn read_byte(&mut self, bridge: &mut dyn MotherboardBridge, address: u32) -> u8;

    /// The CPU must call this to read a word from the system address space.
    fn read_word(&mut self, bridge: &mut dyn MotherboardBridge, address: u32) -> u32;

    /// The CPU must call this to write a byte to the system address space.
    fn write_byte(&mut self, bridge: &mut dyn MotherboardBridge, address: u32, value: u8);

    /// The CPU must call this to write a word to the system address space.
    fn write_word(&mut self, bridge: &mut dyn MotherboardBridge, address: u32, value: u32);

    /// The CPU must call this to increment interrupt counters and trigger
    /// timer updates and GPU updates to be done.
    fn increment_interrupt_counters(&mut self, bridge: &mut dyn MotherboardBridge);

    /// The CD-ROM drive must call this to specify if its interrupt is actually enabled.
    fn set_cdrom_interrupt_enabled(&mut self, enabled: bool);

    /// The CD-ROM drive must call this to specify its interrupt delay.
    fn set_cdrom_interrupt_delay(&mut self, delay: i32);

    /// The CD-ROM drive must call this to set the interrupt number inside
    /// the motherboard implementation.
    fn set_cdrom_interrupt_number(&mut self, number: u8);
    
    /// The GPU must call this to set the GPU interrupt delay.
    fn set_gpu_interrupt_delay(&mut self, delay: i32);
}

/// This trait provides an implementation-opaque way of the motherboard
/// calling methods from elsewhere in the system via a 'bridge'.
pub trait MotherboardBridge {

    /// The motherboard must call this to set the CD-ROM drive's interrupt flag register.
    fn cdrom_set_interrupt_number(&mut self, _: &mut dyn Motherboard, interrupt_num: u8);

    /// The motherboard must call this to append a cycle count to the GPU's count.
    fn gpu_append_sync_cycles(&mut self, motherboard: &mut dyn Motherboard, cycles: i32);
    
    /// The motherboard must call this to determine if the GPU is currently
    /// within the hblank phase of the scanline.
    fn gpu_is_in_hblank(&mut self, motherboard: &mut dyn Motherboard) -> bool;

    /// The motherboard must call this to determine if the GPU is currently
    /// within the vblank phase of screen drawing.
    fn gpu_is_in_vblank(&mut self, motherboard: &mut dyn Motherboard) -> bool;

    /// The motherboard must call this to determine how many GPU
    /// cycles will be left after a round of dotclock timer incrementation.
    fn gpu_how_many_dotclock_gpu_cycles_left(
        &self,
        motherboard: &mut dyn Motherboard,
        gpu_cycles: i32
    ) -> i32;

    /// The motherboard must call this to determine how many GPU dotclock
    /// timer increments are needed.
    fn gpu_how_many_dotclock_increments(
        &self,
        motherboard: &mut dyn Motherboard,
        gpu_cycles: i32
    ) -> i32;

    /// The motherboard must call this to determine how many GPU
    /// cycles will be left after a round of hblank timer incrementation.
    fn gpu_how_many_hblank_gpu_cycles_left(
        &self,
        motherboard: &mut dyn Motherboard,
        gpu_cycles: i32
    ) -> i32;

    /// The motherboard must call this to determine how many GPU hblank
    /// timer increments are needed.
    fn gpu_how_many_hblank_increments(
        &self,
        motherboard: &mut dyn Motherboard,
        gpu_cycles: i32
    ) -> i32;

    /// The motherboard must call this to append a cycle count to the controllers implementation's count.
    fn controllers_append_sync_cycles(&mut self, motherboard: &mut dyn Motherboard, cycles: i32);
}