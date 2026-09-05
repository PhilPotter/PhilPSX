// SPDX-License-Identifier: GPL-3.0
// dma.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

use philpsx_utility::SystemBusHolder;
use crate::{
    bridges::motherboard::MotherboardBridgeImpl,
    cdrom_drive::CdromDrive,
    controllers::Controllers,
    cpu::Cpu,
    dma::{DmaArbiter, DmaArbiterBridge},
    gpu::Gpu,
    motherboard::{Motherboard, MotherboardBridge},
    spu::Spu,
};

/// This struct contains internal references for all other
/// required components that might be needed inside a DmaArbiterBridge.
pub struct DmaArbiterBridgeImpl<'a> {
    cdrom_drive: &'a mut dyn CdromDrive,
    controllers: &'a mut dyn Controllers,
    cpu: &'a mut dyn Cpu,
    gpu: &'a mut dyn Gpu,
    motherboard: &'a mut dyn Motherboard,
    spu: &'a mut dyn Spu,
}

/// Mapping functions for the bridge.
impl<'a> DmaArbiterBridge for DmaArbiterBridgeImpl<'a> {

    fn set_dma_interrupt_delay(&mut self, dma: &mut dyn DmaArbiter, delay: i32) {
        let (motherboard, _) = self.get_motherboard_and_bridge(dma);
        motherboard.set_dma_interrupt_delay(delay);
    }

    fn set_system_bus_holder(&mut self, dma: &mut dyn DmaArbiter, holder: SystemBusHolder) {
        let (motherboard, mut bridge) = self.get_motherboard_and_bridge(dma);
        motherboard.set_system_bus_holder(&mut bridge, holder);
    }
}

/// This implementation exists just to create the bridge.
impl<'a, 'b> DmaArbiterBridgeImpl<'a> {

    /// Create a new DmaArbiterBridgeImpl object.
    pub fn new(
        cdrom_drive: &'b mut dyn CdromDrive,
        controllers: &'b mut dyn Controllers,
        cpu: &'b mut dyn Cpu,
        gpu: &'b mut dyn Gpu,
        motherboard: &'b mut dyn Motherboard,
        spu: &'b mut dyn Spu,
    ) -> Self where 'b: 'a {
        DmaArbiterBridgeImpl {
            cdrom_drive,
            controllers,
            cpu,
            gpu,
            motherboard,
            spu,
        }
    }

    /// Creates a motherboard bridge from this bridge, and also returns a
    /// motherboard reference too, meaning we can call functions on the
    /// motherboard that require a bridge, and pass this new object to them.
    fn get_motherboard_and_bridge(
        &'b mut self,
        dma: &'b mut dyn DmaArbiter
    ) -> (&'b mut dyn Motherboard, impl MotherboardBridge) {
        let motherboard_bridge = MotherboardBridgeImpl::new(
            self.cdrom_drive,
            self.controllers,
            self.cpu,
            self.spu,
            self.gpu,
            dma,
        );
        (self.motherboard, motherboard_bridge)
    }
}