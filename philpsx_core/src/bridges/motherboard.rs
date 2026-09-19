// SPDX-License-Identifier: GPL-3.0
// motherboard.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

use philpsx_utility::SystemBusHolder;
use crate::{
    bridges::{
        cdrom_drive::CdromDriveBridgeImpl,
        dma::DmaArbiterBridgeImpl,
        gpu::GpuBridgeImpl,
    },
    cdrom_drive::{CdromDrive, CdromDriveBridge},
    controllers::Controllers,
    cpu::Cpu,
    gpu::GpuBridge,
    motherboard::{Motherboard, MotherboardBridge},
    spu::Spu,
    gpu::Gpu,
    dma::{DmaArbiter, DmaArbiterBridge},
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

    fn controllers_append_sync_cycles(&mut self, _: &mut dyn Motherboard, cycles: i32) {
        self.controllers.append_sync_cycles(cycles);
    }

    fn controllers_read_byte(&mut self, _: &mut dyn Motherboard, address: u8) -> u8 {
        self.controllers.read_byte(address)
    }

    fn controllers_write_byte(
        &mut self,
        _: &mut dyn Motherboard,
        address: u8,
        value: u8
    ) {
        self.controllers.write_byte(address, value);
    }

    fn cpu_set_system_bus_holder(&mut self, _: &mut dyn Motherboard, holder: SystemBusHolder) {
        self.cpu.set_system_bus_holder(holder);
    }

    fn cpu_virtual_to_physical(&mut self, _: &mut dyn Motherboard, address: u32) -> u32 {
        self.cpu.virtual_to_physical(address)
    }

    fn cdrom_drive_chunk_copy(
        &mut self,
        motherboard: &mut dyn Motherboard,
        starting_byte_address: u32,
        num_of_bytes: u32
    ) {
        self.cdrom_drive.chunk_copy(
            motherboard.get_ram_reference(),
            starting_byte_address,
            num_of_bytes
        );
    }

    fn cdrom_drive_read_1800(&mut self, _: &mut dyn Motherboard) -> u8 {
        self.cdrom_drive.read_1800()
    }

    fn cdrom_drive_read_1801(&mut self, _: &mut dyn Motherboard) -> u8 {
        self.cdrom_drive.read_1801()
    }

    fn cdrom_drive_read_1802(&mut self, _: &mut dyn Motherboard) -> u8 {
        self.cdrom_drive.read_1802()
    }

    fn cdrom_drive_read_1803(&mut self, _: &mut dyn Motherboard) -> u8 {
        self.cdrom_drive.read_1803()
    }

    fn cdrom_drive_write_1800(&mut self, _: &mut dyn Motherboard, value: u8) {
        self.cdrom_drive.write_1800(value);
    }

    fn cdrom_drive_write_1801(&mut self, motherboard: &mut dyn Motherboard, value: u8) {
        let (cdrom, mut bridge) = self.get_cdrom_and_bridge(motherboard);
        cdrom.write_1801(&mut bridge, value);
    }

    fn cdrom_drive_write_1802(&mut self, _: &mut dyn Motherboard, value: u8) {
        self.cdrom_drive.write_1802(value);
    }

    fn cdrom_drive_write_1803(&mut self, motherboard: &mut dyn Motherboard, value: u8) {
        let (cdrom, mut bridge) = self.get_cdrom_and_bridge(motherboard);
        cdrom.write_1803(&mut bridge, value);
    }

    fn cdrom_drive_set_interrupt_number(&mut self, _: &mut dyn Motherboard, interrupt_num: u8) {
        self.cdrom_drive.set_interrupt_number(interrupt_num);
    }

    fn dma_read_byte(&mut self, _: &mut dyn Motherboard, address: u32) -> u8 {
        self.dma.read_byte(address)
    }

    fn dma_read_word(&mut self, _: &mut dyn Motherboard, address: u32) -> u32 {
        self.dma.read_word(address)
    }

    fn dma_write_byte(&mut self, motherboard: &mut dyn Motherboard, address: u32, value: u8) {
        let (dma, mut bridge) = self.get_dma_and_bridge(motherboard);
        dma.write_byte(&mut bridge, address, value);
    }

    fn dma_write_word(&mut self, motherboard: &mut dyn Motherboard, address: u32, value: u32) {
        let (dma, mut bridge) = self.get_dma_and_bridge(motherboard);
        dma.write_word(&mut bridge, address, value);
    }

    fn gpu_append_sync_cycles(&mut self, _: &mut dyn Motherboard, cycles: i32) {
        self.gpu.append_sync_cycles(cycles);
    }

    fn gpu_execute_gpu_cycles(&mut self, motherboard: &mut dyn Motherboard) {
        let (gpu, mut bridge) = self.get_gpu_and_bridge(motherboard);
        gpu.execute_gpu_cycles(&mut bridge);
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

    fn gpu_submit_to_gp0(&mut self, motherboard: &mut dyn Motherboard, word: u32) {
        let (gpu, mut bridge) = self.get_gpu_and_bridge(motherboard);
        gpu.submit_to_gp0(&mut bridge, word);
    }

    fn gpu_submit_to_gp1(&mut self, motherboard: &mut dyn Motherboard, word: u32) {
        let (gpu, mut bridge) = self.get_gpu_and_bridge(motherboard);
        gpu.submit_to_gp1(&mut bridge, word);
    }

    fn gpu_read_response(&mut self, motherboard: &mut dyn Motherboard) -> u32 {
        let (gpu, mut bridge) = self.get_gpu_and_bridge(motherboard);
        gpu.read_response(&mut bridge)
    }

    fn gpu_read_status(&mut self, motherboard: &mut dyn Motherboard) -> u32 {
        let (gpu, mut bridge) = self.get_gpu_and_bridge(motherboard);
        gpu.read_status(&mut bridge)
    }

    fn spu_read_byte(&mut self, _: &mut dyn Motherboard, address: u32) -> u8 {
        self.spu.read_byte(address)
    }

    fn spu_write_byte(&mut self, _: &mut dyn Motherboard, address: u32, value: u8) {
        self.spu.write_byte(address, value);
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

    /// Creates a DMA arbiter bridge from this bridge, and also returns a
    /// DMA arbiter reference too, meaning we can call functions on the
    /// DMA arbiter that require a bridge, and pass this new object to them.
    fn get_dma_and_bridge(
        &'b mut self,
        motherboard: &'b mut dyn Motherboard
    ) -> (&'b mut dyn DmaArbiter, impl DmaArbiterBridge) {
        let dma_bridge = DmaArbiterBridgeImpl::new(
            self.cdrom_drive,
            self.controllers,
            self.cpu,
            self.gpu,
            motherboard,
            self.spu,
        );
        (self.dma, dma_bridge)
    }

    /// Creates a GPU drive bridge from this bridge, and also returns a
    /// GPU reference too, meaning we can call functions on the
    /// GPU that require a bridge, and pass this new object to them.
    fn get_gpu_and_bridge(
        &'b mut self,
        motherboard: &'b mut dyn Motherboard
    ) -> (&'b mut dyn Gpu, impl GpuBridge) {
        let gpu_bridge = GpuBridgeImpl::new(
            self.cdrom_drive,
            self.controllers,
            self.cpu,
            motherboard,
            self.spu,
            self.dma,
        );
        (self.gpu, gpu_bridge)
    }
}