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

    /// This function should return sign-extended version of the original value, based
    /// on extension from the n-th most significant bit as specified. It can be used
    /// for arbitrary widths within the type (for example 16-bit values).
    fn sign_extend(self, from_bit: i32) -> Self::Output;
}

impl CustomInteger for CustomInt32 {

    type Output = i32;

    /// Logically shifts right by specified amount, returning `i32`.
    #[inline(always)]
    fn logical_rshift(self, shift_by: i32) -> Self::Output {
        ((self as u32) >> shift_by) as Self::Output
    }

    /// Sign extends based on the specified bit, with 31 being most significant and
    /// 0 being least significant.
    #[inline(always)]
    fn sign_extend(self, from_bit: i32) -> Self::Output {

        let bit_pattern_to_test = 0x1_i32 << from_bit;
        let extension_pattern = (0xFFFFFFFE_u32 as i32) << from_bit;

        if self & bit_pattern_to_test == 0 {
            self
        } else {
            self | extension_pattern
        }
    }
}

impl CustomInteger for CustomInt64 {

    type Output = i64;

    /// Logically shifts right by specified amount, returning `i64`.
    #[inline(always)]
    fn logical_rshift(self, shift_by: i32) -> Self::Output {
        ((self as u64) >> shift_by) as Self::Output
    }

    /// Sign extends based on the specified bit, with 63 being most significant and
    /// 0 being least significant.
    #[inline(always)]
    fn sign_extend(self, from_bit: i32) -> Self::Output {

        let bit_pattern_to_test = 0x1_i64 << from_bit;
        let extension_pattern = (0xFFFFFFFF_FFFFFFFE_u64 as i64) << from_bit;

        if self & bit_pattern_to_test == 0 {
            self
        } else {
            self | extension_pattern
        }
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
    fn sign_extend_should_extend_8_bit_value_if_bit_7_is_set_for_i32() {

        let input = 0x80;
        let output = input.sign_extend(7);

        assert_eq!(output, 0xFFFFFF80_u32 as i32);
    }

    #[test]
    fn sign_extend_should_leave_8_bit_value_if_bit_7_is_unset_for_i32() {

        let input = 0x70;
        let output = input.sign_extend(7);

        assert_eq!(output, 0x70);
    }

    #[test]
    fn sign_extend_should_extend_16_bit_value_if_bit_15_is_set_for_i32() {

        let input = 0x8000;
        let output = input.sign_extend(15);

        assert_eq!(output, 0xFFFF8000_u32 as i32);
    }

    #[test]
    fn sign_extend_should_leave_16_bit_value_if_bit_15_is_unset_for_i32() {

        let input = 0x7000;
        let output = input.sign_extend(15);

        assert_eq!(output, 0x7000);
    }

    #[test]
    fn sign_extend_should_extend_8_bit_value_if_bit_7_is_set_for_i64() {

        let input = 0x80_i64;
        let output = input.sign_extend(7);

        assert_eq!(output, 0xFFFFFFFF_FFFFFF80_u64 as i64);
    }

    #[test]
    fn sign_extend_should_leave_8_bit_value_if_bit_7_is_unset_for_i64() {

        let input = 0x70_i64;
        let output = input.sign_extend(7);

        assert_eq!(output, 0x70_i64);
    }

    #[test]
    fn sign_extend_should_extend_16_bit_value_if_bit_15_is_set_for_i64() {

        let input = 0x8000_i64;
        let output = input.sign_extend(15);

        assert_eq!(output, 0xFFFFFFFF_FFFF8000_u64 as i64);
    }

    #[test]
    fn sign_extend_should_leave_16_bit_value_if_bit_15_is_unset_for_i64() {

        let input = 0x7000_i64;
        let output = input.sign_extend(15);

        assert_eq!(output, 0x7000_i64);
    }

    #[test]
    fn sign_extend_should_extend_32_bit_value_if_bit_31_is_set_for_i64() {

        let input = 0x80000000_i64;
        let output = input.sign_extend(31);

        assert_eq!(output, 0xFFFFFFFF_80000000_u64 as i64);
    }

    #[test]
    fn sign_extend_should_leave_32_bit_value_if_bit_31_is_unset_for_i64() {

        let input = 0x70000000_i64;
        let output = input.sign_extend(31);

        assert_eq!(output, 0x70000000_i64);
    }
}