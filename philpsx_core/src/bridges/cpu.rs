// SPDX-License-Identifier: GPL-3.0
// cpu.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

use crate::{
    bridges::motherboard::MotherboardBridgeImpl,
    cdrom_drive::CdromDrive,
    controllers::Controllers,
    cpu::{Cpu, CpuBridge},
    motherboard::{Motherboard, MotherboardBridge},
    spu::Spu,
    gpu::Gpu,
    dma::DmaArbiter,
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
    gpu: &'a mut dyn Gpu,
    dma: &'a mut dyn DmaArbiter,
}

/// Mapping functions for the bridge.
impl<'a> CpuBridge for CpuBridgeImpl<'a> {

    fn append_sync_cycles(&mut self, cpu: &mut dyn Cpu, cycles: i32) {
        let (motherboard, mut bridge) = self.get_motherboard_and_bridge(cpu);
        motherboard.append_sync_cycles(&mut bridge, cycles);
    }

    fn how_how_many_stall_cycles(&self, _: &mut dyn Cpu, address: u32) -> i32 {
        self.motherboard.how_how_many_stall_cycles(address)
    }

    fn ok_to_increment(&self, _: &mut dyn Cpu, address: u32) -> bool {
        self.motherboard.ok_to_increment(address)
    }

    fn scratchpad_enabled(&self, _: &mut dyn Cpu) -> bool {
        self.motherboard.scratchpad_enabled()
    }

    fn instruction_cache_enabled(&self, _: &mut dyn Cpu) -> bool {
        self.motherboard.instruction_cache_enabled()
    }

    fn read_byte(&mut self, cpu: &mut dyn Cpu, address: u32) -> u8 {
        let (motherboard, mut bridge) = self.get_motherboard_and_bridge(cpu);
        motherboard.read_byte(&mut bridge, address)
    }

    fn read_word(&mut self, cpu: &mut dyn Cpu, address: u32) -> u32 {
        let (motherboard, mut bridge) = self.get_motherboard_and_bridge(cpu);
        motherboard.read_word(&mut bridge, address)
    }

    fn write_byte(&mut self, cpu: &mut dyn Cpu, address: u32, value: u8) {
        let (motherboard, mut bridge) = self.get_motherboard_and_bridge(cpu);
        motherboard.write_byte(&mut bridge, address, value)
    }

    fn write_word(&mut self, cpu: &mut dyn Cpu, address: u32, value: u32) {
        let (motherboard, mut bridge) = self.get_motherboard_and_bridge(cpu);
        motherboard.write_word(&mut bridge, address, value)
    }

    fn increment_interrupt_counters(&mut self, cpu: &mut dyn Cpu) {
        let (motherboard, mut bridge) = self.get_motherboard_and_bridge(cpu);
        motherboard.increment_interrupt_counters(&mut bridge)
    }
}

/// This implementation exists just to create the bridge and convert it as needed.
impl<'a, 'b> CpuBridgeImpl<'a> {

    /// Creates a new CpuBridgeImpl object.
    pub fn new(
        cdrom_drive: &'b mut dyn CdromDrive,
        controllers: &'b mut dyn Controllers,
        motherboard: &'b mut dyn Motherboard,
        spu: &'b mut dyn Spu,
        gpu: &'b mut dyn Gpu,
        dma: &'b mut dyn DmaArbiter
    ) -> Self where 'b: 'a {
        CpuBridgeImpl {
            cdrom_drive,
            controllers,
            motherboard,
            spu,
            gpu,
            dma,
        }
    }

    /// Creates a motherboard bridge from this bridge, and also returns a
    /// motherboard reference too, meaning we can call functions on the
    /// motherboard that require a bridge, and pass this new object to them.
    fn get_motherboard_and_bridge(
        &'b mut self,
        cpu: &'b mut dyn Cpu
    ) -> (&'b mut dyn Motherboard, impl MotherboardBridge) {
        let motherboard_bridge = MotherboardBridgeImpl::new(
            self.cdrom_drive,
            self.controllers,
            cpu,
            self.spu,
            self.gpu,
            self.dma,
        );
        (self.motherboard, motherboard_bridge)
    }
}