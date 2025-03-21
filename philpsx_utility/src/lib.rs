// SPDX-License-Identifier: GPL-3.0
// lib.rs - Copyright Phillip Potter, 2025, under GPLv3 only.

// This crate contains useful utility functions that can be used throughout the codebase.

/// Exists to allow us to define custom trait operations on `i32`.
type CustomInt32 = i32;

/// Exists to allow us to define custom trait operations on `i64`.
type CustomInt64 = i64;

/// This trait exists to allow us to implement `logical_rshift` in the same way as the
/// C macro original, at least from a semantic perspective.
pub trait CustomInteger {

    type Output;

    /// This function should return a signed value, logically right-shifted by the
    /// specified amount and of the same width as the original, without sign-extension.
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

/// Re-exported stdlib `min` function, to keep all our utility functions together
/// here in the same way they are for the C macro versions.
pub use std::cmp::min;

/// This function is intended for use with 16-bit values stored within an i32.
/// It will sign-extend them as necessary. It is useful due to using signed
/// i32 everywhere - this is a holdover from the original C version, which likewise
/// used this because I originally ported it from the even older Java version that
/// I wrote for my university degree.
#[inline(always)]
pub fn sign_extend(value: i32) -> i32 {
    if value & 0x8000 != 0 {
        value | 0xFFFF0000u32 as i32
    } else {
        value
    }
}


#[cfg(test)]
mod tests {

    use super::sign_extend;

    use super::CustomInteger;

    #[test]
    fn logical_rshift_should_work_as_expected_for_i32() {

        let input = 0xFFFFFFFF_u32 as i32;
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
    fn sign_extend_should_extend_16_bit_value_if_bit_15_is_set() {

        let input = 0x8000;
        let output = sign_extend(input);

        assert_eq!(output, 0xFFFF8000_u32 as i32);
    }

    #[test]
    fn sign_extend_should_leave_16_bit_value_if_bit_15_is_unset() {

        let input = 0x7000;
        let output = sign_extend(input);

        assert_eq!(output, 0x7000);
    }
}