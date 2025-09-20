// SPDX-License-Identifier: GPL-3.0
// motherboard.rs - Copyright Phillip Potter, 2025, under GPLv3 only.

use super::cpu::Cpu;

/// This module contains the default motherboard implementation. There
/// may be others in future.
pub mod psx_motherboard;

/// This trait provides an implementation-opaque way of calling motherboard
/// methods from elsewhere in the system via a 'bridge'.
pub trait Motherboard {}

/// This struct exists to allow us to reference all components mutably when
/// operating from the context of a motherboard call.
pub struct MotherboardComponents<'a> {
    cpu: &'a mut dyn Cpu,
}