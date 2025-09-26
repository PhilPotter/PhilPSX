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
        bridge: &mut dyn CpuBridge,
        holder: SystemBusHolder
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

    /// The CPU must call this to append a cycle count to the system count.
    fn append_sync_cycles(&mut self, cpu: &mut dyn Cpu, cycles: i32);

    /// The CPU must call this to determine how many stall cycles are needed.
    fn how_how_many_stall_cycles(&self, cpu: &mut dyn Cpu, address: i32) -> i32;

    /// The CPU must call this to determine if an address should be incremented.
    fn ok_to_increment(&self, cpu: &mut dyn Cpu, address: i64) -> bool;

    /// The CPU must call this to determine if the scratchpad is enabled.
    fn scratchpad_enabled(&self, cpu: &mut dyn Cpu) -> bool;

    /// The CPU must call this to determine if the instruction cache is enabled.
    fn instruction_cache_enabled(&self, cpu: &mut dyn Cpu) -> bool;

    /// The CPU must call this to read a byte from the system bus.
    fn read_byte(&self, cpu: &mut dyn Cpu, address: i32) -> i8;

    /// The CPU must call this to read a word from the system bus.
    fn read_word(&self, cpu: &mut dyn Cpu, address: i32) -> i32;

    /// The CPU must call this to write a byte to the system bus.
    fn write_byte(&mut self, cpu: &mut dyn Cpu, address: i32, value: i8);

    /// The CPU must call this to write a word to the system bus.
    fn write_word(&mut self, cpu: &mut dyn Cpu, address: i32, value: i8);

    /// The CPU must call this to increment all interrupt-relevant counters.
    fn increment_interrupt_counters(&mut self, cpu: &mut dyn Cpu);
}