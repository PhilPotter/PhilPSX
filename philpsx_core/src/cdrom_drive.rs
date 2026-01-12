// SPDX-License-Identifier: GPL-3.0
// cdrom_drive.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

use std::path::Path;

/// This module contains the default CD-ROM implementation, including the representation
/// of the 'CD' itself. There may be others in future.
pub mod psx_cdrom_drive;

/// This trait provides an implementation-opaque way of calling CD-ROM methods from
/// elsewhere in the system.
pub trait CdromDrive {

    /// Implementations must use this to read chunks into the supplied buffer.
    fn chunk_copy(
        &mut self,
        buffer: &mut Vec<i8>,
        start_index: i32,
        length: i32
    );

    /// Implementations must load the CD from the image file referenced by the
    /// supplied path.
    fn load_cd(
        &mut self,
        path: &Path
    );

    /// Implementations must return a byte from the index/status register.
    fn read_1800(&self) -> i8;

    /// Implementations must return a byte from port 0x1F801801.
    fn read_1801(&self) -> i8;

    /// Implementations must return a byte from port 0x1F801802.
    fn read_1802(&self) -> i8;

    /// Implementations must return a byte from port 0x1F801803.
    fn read_1803(&self) -> i8;

    /// Implementations must set the interrupt flag register contents.
    fn set_interrupt_number(&mut self, interrupt_num: i32);

    /// Implementations must write a byte to port 0x1F801800.
    fn write_1800(&mut self, value: i8);

    /// Implementations must write a byte to port 0x1F801801.
    fn write_1801(&mut self, value: i8);

    /// Implementations must write a byte to port 0x1F801802.
    fn write_1802(&mut self, value: i8);

    /// Implementations must write a byte to port 0x1F801803.
    fn write_1803(&mut self, value: i8);
}

/// This trait provides an implementation-opaque way of the CD-DROM drive calling
/// methods from elsewhere in the system via a 'bridge'.
pub trait CDROMDriveBridge {

    /// The CD-ROM drive must call this to specify if its interrupt is actually enabled. 
    fn set_cdrom_interrupt_enabled(&mut self, cdrom_drive: &mut dyn CdromDrive, enabled: bool);

    /// The CD-ROM drive must call this to specify its interrupt delay.
    fn set_cdrom_interrupt_delay(&mut self, cdrom_drive: &mut dyn CdromDrive, delay: i32);

    /// The CD-ROM drive must call this to specify its interrupt number.
    fn set_cdrom_interrupt_number(&mut self, cdrom_drive: &mut dyn CdromDrive, number: i32);
}