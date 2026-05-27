// SPDX-License-Identifier: GPL-3.0
// gpu.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

/// This module contains the default GPU implementation. There may
/// be others in future.
pub mod psx_gpu;

/// This trait provides an implementation-opaque way of calling GPU
/// methods from elsewhere in the system.
pub trait Gpu {
}