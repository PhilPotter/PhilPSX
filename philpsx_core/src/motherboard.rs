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
}

/// This trait provides an implementation-opaque way of the motherboard
/// calling methods from elsewhere in the system via a 'bridge'.
pub trait MotherboardBridge {
}