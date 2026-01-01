// SPDX-License-Identifier: GPL-3.0
// lib.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

// Crate-wide lines to disable specific lints:

// Given the code has been carefully ported from C by hand as a re-learning
// experience, to keep semantics as close as possible, there will be no
// derived Default implementations unless needed.
#![allow(clippy::new_without_default)]

// We use upper-case acronyms for some enums, in order to match the original
// C source more closely.
#![allow(clippy::upper_case_acronyms)]

/// This module contains PlayStation CPU-related functionality.
pub mod cpu;

/// This module contains PlayStation motherboard related functionality.
pub mod motherboard;

/// This module contains PlayStation sound chip related functionality.
pub mod spu;

/// This module contains PlayStation controller related functionality.
pub mod controllers;