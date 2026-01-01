// SPDX-License-Identifier: GPL-3.0
// spu.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

/// This module contains the default sound chip implementation. There
/// may be others in future.
pub mod psx_spu;

/// This trait provides an implementation-opaque way of calling SPU
/// methods from elsewhere in the system via a 'bridge'.
pub trait Spu {
    
}