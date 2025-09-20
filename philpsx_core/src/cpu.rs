// SPDX-License-Identifier: GPL-3.0
// cpu.rs - Copyright Phillip Potter, 2025, under GPLv3 only.

use philpsx_utility::SystemBusHolder;

/// This module contains the default R3051 implmentation. There
/// may be others in future.
pub mod r3051;

/// This trait provides an implementation-opaque way of calling CPU
/// methods from elsewhere in the system. We supply a bridge object
/// so that the implementations can call out via it if they need
/// additional information/processing. This allows us to have
/// arbitrarily deep call stacks of components calling each other,
/// which is needed to reflect the semantics of the original C version.
pub trait Cpu {

    /// Implementations must use this to set the system bus holder.
    fn set_system_bus_holder(
        &mut self,
        holder: SystemBusHolder,
        bridge: &mut dyn CpuBridge
    );

    /// Implementations must use this to retrieve the system bus holder.
    fn get_system_bus_holder(
        &self,
        bridge: &mut dyn CpuBridge
    ) -> SystemBusHolder;
}

/// This trait provides an implementation-opaque way of the CPU
/// calling methods from elsewhere in the system via a 'bridge'.
pub trait CpuBridge {
}