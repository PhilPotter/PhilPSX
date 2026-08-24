// SPDX-License-Identifier: GPL-3.0
// gpu.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

use crate::{
    cdrom_drive::CdromDrive,
    controllers::Controllers,
    cpu::Cpu,
    dma::DmaArbiter,
    gpu::{Gpu, GpuBridge},
    motherboard::Motherboard,
    spu::Spu,
};
use crate::bridges::motherboard::MotherboardBridgeImpl;
use crate::motherboard::MotherboardBridge;

/// This struct contains internal references for all other
/// required components that might be needed inside a GpuBridge.
pub struct GpuBridgeImpl<'a> {
    cdrom_drive: &'a mut dyn CdromDrive,
    controllers: &'a mut dyn Controllers,
    cpu: &'a mut dyn Cpu,
    motherboard: &'a mut dyn Motherboard,
    spu: &'a mut dyn Spu,
    dma: &'a mut dyn DmaArbiter,
}

/// Mapping functions for the bridge.
impl<'a> GpuBridge for GpuBridgeImpl<'a> {

    fn set_gpu_interrupt_delay(&mut self, gpu: &mut dyn Gpu, delay: i32) {
        let (motherboard, _) = self.get_motherboard_and_bridge(gpu);
        motherboard.set_gpu_interrupt_delay(delay);
    }
}

/// This implementation exists just to create the bridge.
impl<'a, 'b> GpuBridgeImpl<'a> {

    /// Create a new GpuBridgeImpl object.
    pub fn new(
        cdrom_drive: &'b mut dyn CdromDrive,
        controllers: &'b mut dyn Controllers,
        cpu: &'b mut dyn Cpu,
        motherboard: &'b mut dyn Motherboard,
        spu: &'b mut dyn Spu,
        dma: &'b mut dyn DmaArbiter,
    ) -> Self where 'b: 'a {
        GpuBridgeImpl {
            cdrom_drive,
            controllers,
            cpu,
            motherboard,
            spu,
            dma,
        }
    }

    /// Creates a motherboard bridge from this bridge, and also returns a
    /// motherboard reference too, meaning we can call functions on the
    /// motherboard that require a bridge, and pass this new object to them.
    fn get_motherboard_and_bridge(
        &'b mut self,
        gpu: &'b mut dyn Gpu
    ) -> (&'b mut dyn Motherboard, impl MotherboardBridge) {
        let motherboard_bridge = MotherboardBridgeImpl::new(
            self.cdrom_drive,
            self.controllers,
            self.cpu,
            self.spu,
            gpu,
            self.dma,
        );
        (self.motherboard, motherboard_bridge)
    }
}