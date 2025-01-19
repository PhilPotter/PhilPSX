// SPDX-License-Identifier: GPL-3.0
// lib.rs - Copyright Phillip Potter, 2025, under GPLv3 only.

// This crate contains useful utility functions that can be used throughout the codebase.

/// Exists to allow us to define custom trait operations on `i32`.
type CustomInt32 = i32;

/// Exists to allow us to define custom trait operations on `i64`.
type CustomInt64 = i64;

/// Exists to allow us to define custom trait operations on `u32`.
type CustomUInt32 = u32;

/// Exists to allow us to define custom trait operations on `u64`.
type CustomUInt64 = u64;

/// This trait exists to allow us to implement `logical_rshift` in the same way as the
/// C macro original, at least from a semantic perspective.
pub trait CustomInteger {

    type Output;

    /// This function should return a signed value, logically right-shifted by the
    /// specified amount and of the same width as the original, whether it is signed
    /// or unsigned.
    fn logical_rshift(self, shift_by: i32) -> Self::Output;
}

impl CustomInteger for CustomInt32 {

    type Output = i32;

    /// Logically shifts right by specified amount, returning `i32`.
    #[inline(always)]
    fn logical_rshift(self, shift_by: i32) -> Self::Output {
        ((self as u32) >> shift_by) as Self::Output
    }
}

impl CustomInteger for CustomInt64 {

    type Output = i64;

    /// Logically shifts right by specified amount, returning `i64`.
    #[inline(always)]
    fn logical_rshift(self, shift_by: i32) -> Self::Output {
        ((self as u64) >> shift_by) as Self::Output
    }
}

impl CustomInteger for CustomUInt32 {

    type Output = i32;

    /// Logically shifts right by specified amount, returning `i32`.
    #[inline(always)]
    fn logical_rshift(self, shift_by: i32) -> Self::Output {
        (self >> shift_by) as Self::Output
    }
}

impl CustomInteger for CustomUInt64 {

    type Output = i64;

    /// Logically shifts right by specified amount, returning `i64`.
    #[inline(always)]
    fn logical_rshift(self, shift_by: i32) -> Self::Output {
        (self >> shift_by) as Self::Output
    }
}

/// Re-exported stdlib `min` function, to keep all our utility functions together
/// here in the same way they are for the C macro versions.
pub use std::cmp::min;

#[cfg(test)]
mod tests {

    use super::CustomInteger;

    #[test]
    fn logical_rshift_should_work_as_expected_for_i32() {

        let input = 0xFFFFFFFFu32 as i32;
        let output = input.logical_rshift(1);

        assert_eq!(output, 0x7FFFFFFF);
    }

    #[test]
    fn logical_rshift_should_work_as_expected_for_i64() {

        let input = 0xFFFFFFFFFFFFFFFFu64 as i64;
        let output = input.logical_rshift(1);

        assert_eq!(output, 0x7FFFFFFFFFFFFFFF);
    }

    #[test]
    fn logical_rshift_should_work_as_expected_for_u32() {

        let input = 0xFFFFFFFFu32;
        let output = input.logical_rshift(1);

        assert_eq!(output, 0x7FFFFFFF);
    }

    #[test]
    fn logical_rshift_should_work_as_expected_for_u64() {

        let input = 0xFFFFFFFFFFFFFFFFu64;
        let output = input.logical_rshift(1);

        assert_eq!(output, 0x7FFFFFFFFFFFFFFF);
    }
}