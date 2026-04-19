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
}

/// This trait provides an implementation-opaque way of the motherboard
/// calling methods from elsewhere in the system via a 'bridge'.
pub trait MotherboardBridge {
}