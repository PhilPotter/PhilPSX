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
}

/// This trait provides an implementation-opaque way of the motherboard
/// calling methods from elsewhere in the system via a 'bridge'.
pub trait MotherboardBridge {
}