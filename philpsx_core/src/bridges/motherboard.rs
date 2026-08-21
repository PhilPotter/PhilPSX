// SPDX-License-Identifier: GPL-3.0
// motherboard.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

use crate::{
    bridges::cdrom_drive::CdromDriveBridgeImpl,
    cdrom_drive::{CdromDrive, CdromDriveBridge},
    controllers::Controllers,
    cpu::Cpu,
    motherboard::{Motherboard, MotherboardBridge},
    spu::Spu,
    gpu::Gpu,
    dma::DmaArbiter,
};

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

    fn cdrom_set_interrupt_number(&mut self, _: &mut dyn Motherboard, interrupt_num: u8) {
        self.cdrom_drive.set_interrupt_number(interrupt_num);
    }

    fn gpu_append_sync_cycles(&mut self, _: &mut dyn Motherboard, cycles: i32) {
        self.gpu.append_sync_cycles(cycles);
    }

    fn gpu_is_in_hblank(&mut self, _: &mut dyn Motherboard) -> bool {
        self.gpu.is_in_hblank()
    }

    fn gpu_is_in_vblank(&mut self, _: &mut dyn Motherboard) -> bool {
        self.gpu.is_in_vblank()
    }

    fn gpu_how_many_dotclock_gpu_cycles_left(
        &self,
        _: &mut dyn Motherboard,
        gpu_cycles: i32
    ) -> i32 {
        self.gpu.how_many_dotclock_gpu_cycles_left(gpu_cycles)
    }

    fn gpu_how_many_dotclock_increments(&self, _: &mut dyn Motherboard, gpu_cycles: i32) -> i32 {
        self.gpu.how_many_dotclock_increments(gpu_cycles)
    }

    fn gpu_how_many_hblank_gpu_cycles_left(&self, _: &mut dyn Motherboard, gpu_cycles: i32) -> i32 {
        self.gpu.how_many_hblank_gpu_cycles_left(gpu_cycles)
    }

    fn gpu_how_many_hblank_increments(&self, _: &mut dyn Motherboard, gpu_cycles: i32) -> i32 {
        self.gpu.how_many_hblank_increments(gpu_cycles)
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

    /// Creates a CD-ROM drive bridge from this bridge, and also returns a
    /// CD-ROM drive reference too, meaning we can call functions on the
    /// CD-ROM drive that require a bridge, and pass this new object to them.
    fn get_cdrom_and_bridge(
        &'b mut self,
        motherboard: &'b mut dyn Motherboard
    ) -> (&'b mut dyn CdromDrive, impl CdromDriveBridge) {
        let cdrom_drive_bridge = CdromDriveBridgeImpl::new(
            self.controllers,
            self.cpu,
            motherboard,
            self.spu,
            self.gpu,
            self.dma,
        );
        (self.cdrom_drive, cdrom_drive_bridge)
    }
}