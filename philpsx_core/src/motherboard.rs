// SPDX-License-Identifier: GPL-3.0
// motherboard.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

use philpsx_utility::SystemBusHolder;

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

    /// The caller must call this to read a word from the system address space.
    fn read_word(&mut self, bridge: &mut dyn MotherboardBridge, address: u32) -> u32;

    /// The CPU must call this to write a byte to the system address space.
    fn write_byte(&mut self, bridge: &mut dyn MotherboardBridge, address: u32, value: u8);

    /// The caller must call this to write a word to the system address space.
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

    /// The DMA arbiter must call this to set the DMA interrupt delay.
    fn set_dma_interrupt_delay(&mut self, delay: i32);

    /// The component caller must call this to set the system bus holder.
    fn set_system_bus_holder(
        &mut self,
        bridge: &mut dyn MotherboardBridge,
        holder: SystemBusHolder
    );

    /// The component caller must call this to convert a virtual address
    /// to a physical address.
    fn virtual_to_physical(&mut self, bridge: &mut dyn MotherboardBridge, address: u32) -> u32;

    /// The component caller must call this to submit GP0 commands to the GPU.
    fn gpu_submit_to_gp0(&mut self, bridge: &mut dyn MotherboardBridge, word: u32);

    /// The component caller must call this to read GPU responses.
    fn gpu_read_response(&mut self, bridge: &mut dyn MotherboardBridge) -> u32;
}

/// This trait provides an implementation-opaque way of the motherboard
/// calling methods from elsewhere in the system via a 'bridge'.
pub trait MotherboardBridge {

    /// The motherboard must call this to set the CPU's system bus holder value.
    fn cpu_set_system_bus_holder(
        &mut self,
        motherboard: &mut dyn Motherboard,
        holder: SystemBusHolder
    );

    /// The motherboard must call this to convert a virtual
    /// address to a physical address.
    fn cpu_virtual_to_physical(&mut self, motherboard: &mut dyn Motherboard, address: u32) -> u32;

    /// The motherboard must call this to set the CD-ROM drive's interrupt flag register.
    fn cdrom_set_interrupt_number(&mut self, motherboard: &mut dyn Motherboard, interrupt_num: u8);

    /// The motherboard must call this to append a cycle count to the GPU's count.
    fn gpu_append_sync_cycles(&mut self, motherboard: &mut dyn Motherboard, cycles: i32);

    /// The motherboard must call this to ensure GPU counters etc. are kept up to date.
    fn gpu_execute_gpu_cycles(&mut self, motherboard: &mut dyn Motherboard);

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

    /// The motherboard must call this to submit GP0 commands to the GPU.
    fn gpu_submit_to_gp0(&mut self, motherboard: &mut dyn Motherboard, word: u32);

    /// The motherboard must call this to read GPU responses.
    fn gpu_read_response(&mut self, motherboard: &mut dyn Motherboard) -> u32;

    /// The motherboard must call this to append a cycle count to the controllers implementation's count.
    fn controllers_append_sync_cycles(&mut self, motherboard: &mut dyn Motherboard, cycles: i32);
}