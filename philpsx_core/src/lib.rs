// SPDX-License-Identifier: GPL-3.0
// lib.rs - Copyright Phillip Potter, 2025, under GPLv3 only.

// Crate-wide lines to disable specific lints:

// Given the code has been carefully ported from C by hand as a re-learning
// experience, to keep semantics as close as possible, there will be no
// derived Default implementations unless needed.
#![allow(clippy::new_without_default)]

// Given we are using i32 registers as in the original implementation,
// but using a lot of 0x hex style literals that would be negative numbers
// within two's complement i32 range, we don't want to be warned about
// casting these to i32.
#![allow(clippy::unnecessary_cast)]

// We use upper-case acronyms for some enums, in order to match the original
// C source more closely.
#![allow(clippy::upper_case_acronyms)]

pub mod cpu;