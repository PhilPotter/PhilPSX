// SPDX-License-Identifier: GPL-3.0
// motherboard.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

use crate::{
    cdrom_drive::CdromDrive,
    controllers::Controllers,
    cpu::Cpu,
    motherboard::MotherboardBridge,
    spu::Spu,
    gpu::Gpu,
    dma::DmaArbiter,
};
use crate::motherboard::Motherboard;

/// This struct contains internal references for all other
/// required components that might be needed inside a MotherboardBridge.
pub struct MotherboardBridgeImpl<'a> {
    cdrom_drive: &'a mut dyn CdromDrive,
    controllers: &'a mut dyn Controllers,
    cpu: &'a mut dyn Cpu,
    spu: &'a mut dyn Spu,
    gpu: &'a mut dyn Gpu,
    dma: &'a mut dyn DmaArbiter,
}

/// Mapping functions for the bridge.
impl<'a> MotherboardBridge for MotherboardBridgeImpl<'a> {

    fn gpu_append_sync_cycles(&mut self, _: &mut dyn Motherboard, cycles: i32) {
        self.gpu.append_sync_cycles(cycles);
    }

    fn controllers_append_sync_cycles(&mut self, _: &mut dyn Motherboard, cycles: i32) {
        self.controllers.append_sync_cycles(cycles);
    }
}

/// This implementation exists just to create the bridge.
impl<'a, 'b> MotherboardBridgeImpl<'a> {

    /// Creates a new MotherboardBridgeImpl object.
    pub fn new(
        cdrom_drive: &'b mut dyn CdromDrive,
        controllers: &'b mut dyn Controllers,
        cpu: &'b mut dyn Cpu,
        spu: &'b mut dyn Spu,
        gpu: &'b mut dyn Gpu,
        dma: &'b mut dyn DmaArbiter
    ) -> Self where 'b: 'a {
        MotherboardBridgeImpl {
            cdrom_drive,
            controllers,
            cpu,
            spu,
            gpu,
            dma,
        }
    }
}