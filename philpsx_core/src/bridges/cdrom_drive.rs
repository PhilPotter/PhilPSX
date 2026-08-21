// SPDX-License-Identifier: GPL-3.0
// cdrom_drive.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

use crate::{
    bridges::motherboard::MotherboardBridgeImpl,
    cdrom_drive::{CdromDrive, CdromDriveBridge},
    controllers::Controllers,
    cpu::Cpu,
    dma::DmaArbiter,
    gpu::Gpu,
    motherboard::{Motherboard, MotherboardBridge},
    spu::Spu,
};

/// This struct contains internal references for all other
/// required components that might be needed inside a CdromDriveBridge.
pub struct CdromDriveBridgeImpl<'a> {
    controllers: &'a mut dyn Controllers,
    cpu: &'a mut dyn Cpu,
    motherboard: &'a mut dyn Motherboard,
    spu: &'a mut dyn Spu,
    gpu: &'a mut dyn Gpu,
    dma: &'a mut dyn DmaArbiter,
}

/// Mapping functions for the bridge.
impl<'a> CdromDriveBridge for CdromDriveBridgeImpl<'a> {

    fn set_cdrom_interrupt_enabled(&mut self, cdrom_drive: &mut dyn CdromDrive, enabled: bool) {
        let (motherboard, _) = self.get_motherboard_and_bridge(cdrom_drive);
        motherboard.set_cdrom_interrupt_enabled(enabled);
    }

    fn set_cdrom_interrupt_delay(&mut self, cdrom_drive: &mut dyn CdromDrive, delay: i32) {
        let (motherboard, _) = self.get_motherboard_and_bridge(cdrom_drive);
        motherboard.set_cdrom_interrupt_delay(delay);
    }

    fn set_cdrom_interrupt_number(&mut self, cdrom_drive: &mut dyn CdromDrive, number: u8) {
        let (motherboard, _) = self.get_motherboard_and_bridge(cdrom_drive);
        motherboard.set_cdrom_interrupt_number(number);
    }
}

/// This implementation exists just to create the bridge.
impl<'a, 'b> CdromDriveBridgeImpl<'a> {

    /// Creates a new CdromDriveBridgeImpl object.
    pub fn new(
        controllers: &'b mut dyn Controllers,
        cpu: &'b mut dyn Cpu,
        motherboard: &'b mut dyn Motherboard,
        spu: &'b mut dyn Spu,
        gpu: &'b mut dyn Gpu,
        dma: &'b mut dyn DmaArbiter,
    ) -> Self where 'b: 'a {
        CdromDriveBridgeImpl {
            controllers,
            cpu,
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
        cdrom_drive: &'b mut dyn CdromDrive
    ) -> (&'b mut dyn Motherboard, impl MotherboardBridge) {
        let motherboard_bridge = MotherboardBridgeImpl::new(
            cdrom_drive,
            self.controllers,
            self.cpu,
            self.spu,
            self.gpu,
            self.dma,
        );
        (self.motherboard, motherboard_bridge)
    }
}