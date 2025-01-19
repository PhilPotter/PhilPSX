// SPDX-License-Identifier: GPL-3.0
// lib.rs - Copyright Phillip Potter, 2025, under GPLv3 only.

// This crate contains useful utility functions that can be used throughout the codebase.

/// This function provides a shorthand way of casting 0x hexadecimal-style literals to
/// i32, so we don't have to keep writing `u32 as i32` on the end, which feels weird
/// given we are just representing the fact that these values are storable inside an
/// i32 (as in the original C source). It's debatable whether this is any cleaner than
/// just writing `0xFFFFFFFFu32 as i32`, but it feels nicer to me.
#[inline(always)]
pub fn as_i32(num: u32) -> i32 {
    num as i32
}