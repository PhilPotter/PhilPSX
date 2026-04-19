// SPDX-License-Identifier: GPL-3.0
// motherboard.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

use super::super::{
    cdrom_drive::CdromDrive,
    controllers::Controllers,
    cpu::Cpu,
    motherboard::MotherboardBridge,
    spu::Spu,
};

/// This struct contains internal references for all other
/// required components that might be needed inside a MotherboardBridge.
pub struct MotherboardBridgeImpl<'a> {
    cdrom_drive: &'a mut dyn CdromDrive,
    controllers: &'a mut dyn Controllers,
    cpu: &'a mut dyn Cpu,
    spu: &'a mut dyn Spu,
}

/// Mapping functions for the bridge.
impl<'a> MotherboardBridge for MotherboardBridgeImpl<'a> {
}

/// This implementation exists just to create the bridge.
impl<'a, 'b> MotherboardBridgeImpl<'a> {

    /// Creates a new MotherboardBridgeImpl object.
    pub fn new(
        cdrom_drive: &'b mut dyn CdromDrive,
        controllers: &'b mut dyn Controllers,
        cpu: &'b mut dyn Cpu,
        spu: &'b mut dyn Spu
    ) -> Self where 'b: 'a {
        MotherboardBridgeImpl {
            cdrom_drive,
            controllers,
            cpu,
            spu,
        }
    }
}