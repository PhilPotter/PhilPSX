// SPDX-License-Identifier: GPL-3.0
// cpu.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

use crate::{bridges::motherboard::MotherboardBridgeImpl, motherboard::MotherboardBridge};

use super::super::{
    cdrom_drive::CdromDrive,
    controllers::Controllers,
    cpu::{Cpu, CpuBridge},
    motherboard::Motherboard,
    spu::Spu,
};

/// This struct contains internal references for all other
/// required components that might be needed inside a CpuBridge.
/// Not the best approach, but at least it keeps the details
/// out of each component itself and isolates them here.
/// Once I have things working (albeit super slowly) I will
/// perhaps think about a way to represent the structure differently
/// from the C original, such that this is no longer required.
pub struct CpuBridgeImpl<'a> {
    cdrom_drive: &'a mut dyn CdromDrive,
    controllers: &'a mut dyn Controllers,
    motherboard: &'a mut dyn Motherboard,
    spu: &'a mut dyn Spu,
}

/// Mapping functions for the bridge.
impl<'a> CpuBridge for CpuBridgeImpl<'a> {

    fn append_sync_cycles(&mut self, cpu: &mut dyn Cpu, cycles: i32) {
        let mut bridge = CpuBridgeImpl::get_motherboard_bridge(
            self.cdrom_drive,
            self.controllers,
            cpu,
            self.spu
        );
        self.motherboard.append_sync_cycles(&mut bridge, cycles);
    }

    fn how_how_many_stall_cycles(&self, cpu: &mut dyn Cpu, address: u32) -> i32 {
        0
    }

    fn ok_to_increment(&self, cpu: &mut dyn Cpu, address: u32) -> bool {
        false
    }

    fn scratchpad_enabled(&self, cpu: &mut dyn Cpu) -> bool {
        false
    }

    fn instruction_cache_enabled(&self, cpu: &mut dyn Cpu) -> bool {
        false
    }

    fn read_byte(&self, cpu: &mut dyn Cpu, address: u32) -> u8 {
        0
    }

    fn read_word(&self, cpu: &mut dyn Cpu, address: u32) -> u32 {
        0
    }

    fn write_byte(&mut self, cpu: &mut dyn Cpu, address: u32, value: u8) {
    }

    fn write_word(&mut self, cpu: &mut dyn Cpu, address: u32, value: u32) {
    }

    fn increment_interrupt_counters(&mut self, cpu: &mut dyn Cpu) {
    }
}

/// This implementation exists just to create the bridge and convert it as needed.
impl<'a, 'b> CpuBridgeImpl<'a> {

    /// Creates a new CpuBridgeImpl object.
    pub fn new(
        cdrom_drive: &'b mut dyn CdromDrive,
        controllers: &'b mut dyn Controllers,
        motherboard: &'b mut dyn Motherboard,
        spu: &'b mut dyn Spu
    ) -> Self where 'b: 'a{
        CpuBridgeImpl {
            cdrom_drive,
            controllers,
            motherboard,
            spu,
        }
    }

    /// Creates a motherboard bridge from this bridge, meaning we can
    /// call functions on the motherboard that require it.
    fn get_motherboard_bridge(
        cdrom_drive: &'b mut dyn CdromDrive,
        controllers: &'b mut dyn Controllers,
        cpu: &'b mut dyn Cpu,
        spu: &'b mut dyn Spu
    ) -> impl MotherboardBridge {
        MotherboardBridgeImpl::new(
            cdrom_drive,
            controllers,
            cpu,
            spu,
        )
    }
}