// SPDX-License-Identifier: GPL-3.0
// math.rs - Copyright Phillip Potter, 2025, under GPLv3 only.

use std::ops;

// Data structures of use for the CP2 / Geometry Transformation Engine.
// Fully aware this is not the most efficient approach, but maybe that
// can come later. For now, just trying to understand what I wrote all
// those years ago and make it clearer :-D

/// A three by three matrix for representing any of:
/// - The rotation matrix.
/// - The light matrix.
/// - The light colour matrix.
#[derive(Copy, Clone)]
pub struct CP2Matrix {

    top_row: [i64; 3],
    middle_row: [i64; 3],
    bottom_row: [i64; 3],
}

/// A one column vector of threem for usage alongside matrices.
#[derive(Copy, Clone)]
pub struct CP2Vector {

    top: i64,
    middle: i64,
    bottom: i64,
}

impl CP2Matrix {

    /// Creates a new CP2Matrix object with the correct initial state.
    pub fn new(
        top_row: [i64; 3],
        middle_row: [i64; 3],
        bottom_row: [i64; 3]
    ) -> Self {
        CP2Matrix {

            // Set rows.
            top_row,
            middle_row,
            bottom_row,
        }
    }

    /// Access the top-left element.
    pub fn top_left(&self) -> i64 {
        self.top_row[0]
    }

    /// Access the top-middle element.
    pub fn top_middle(&self) -> i64 {
        self.top_row[1]
    }

    /// Access the top-right element.
    pub fn top_right(&self) -> i64 {
        self.top_row[2]
    }

    /// Access the middle-left element.
    pub fn middle_left(&self) -> i64 {
        self.middle_row[0]
    }

    /// Access the middle element.
    pub fn middle(&self) -> i64 {
        self.middle_row[1]
    }

    /// Access the middle-right element.
    pub fn middle_right(&self) -> i64 {
        self.middle_row[2]
    }

    /// Access the bottom-left element.
    pub fn bottom_left(&self) -> i64 {
        self.bottom_row[0]
    }

    /// Access the bottom-middle element.
    pub fn bottom_middle(&self) -> i64 {
        self.bottom_row[1]
    }

    /// Access the bottom-right element.
    pub fn bottom_right(&self) -> i64 {
        self.bottom_row[2]
    }
}

impl CP2Vector {

    /// Creates a new CP2Vector object with the correct initial state.
    pub fn new(
        top: i64,
        middle: i64,
        bottom: i64,
    ) -> Self {
        CP2Vector {

            // Set column.
            top,
            middle,
            bottom,
        }
    }

    /// Access the top element.
    pub fn top(&self) -> i64 {
        self.top
    }

    /// Access the middle element.
    pub fn middle(&self) -> i64 {
        self.middle
    }

    /// Access the bottom element.
    pub fn bottom(&self) -> i64 {
        self.bottom
    }
}

/// Implements multiplication between CP2Matrix and CP2Vector.
impl ops::Mul<CP2Vector> for CP2Matrix {

    // Output should be a CP2Vector as well.
    type Output = CP2Vector;

    fn mul(self, rhs: CP2Vector) -> Self::Output {
        CP2Vector::new(
            self.top_row[0] * rhs.top + self.top_row[1] * rhs.middle + self.top_row[2] * rhs.bottom,
            self.middle_row[0] * rhs.top + self.middle_row[1] * rhs.middle + self.middle_row[2] * rhs.bottom,
            self.bottom_row[0] * rhs.top + self.bottom_row[1] * rhs.middle + self.bottom_row[2] * rhs.bottom
        )
    }
}

/// Implements addition between two CP2Vector values.
impl ops::Add<CP2Vector> for CP2Vector {

    // Output should be a CP2Vector.
    type Output = CP2Vector;

    fn add(self, rhs: CP2Vector) -> Self::Output {
        CP2Vector::new(
            self.top + rhs.top,
            self.middle + rhs.middle,
            self.bottom + rhs.bottom
        )
    }
}

#[cfg(test)]
mod tests {

    use super::{CP2Matrix, CP2Vector};

    #[test]
    fn multiplication_works_as_expected_between_matrix_and_vector() {

        let input_matrix = CP2Matrix::new(
            [1, 2, 3],
            [4, 5, 6],
            [7, 8, 9]
        );
        let input_vector = CP2Vector::new(
            10, 11, 12
        );

        let output_vector = input_matrix * input_vector;

        assert_eq!(output_vector.top(), 68);
        assert_eq!(output_vector.middle(), 167);
        assert_eq!(output_vector.bottom(), 266);
    }

    #[test]
    fn addition_works_as_expected_between_two_vectors() {

        let input_vector_1 = CP2Vector::new(
            1, 2, 3
        );
        let input_vector_2 = CP2Vector::new(
            2, 4, 6
        );

        let output_vector = input_vector_1 + input_vector_2;

        assert_eq!(output_vector.top(), 3);
        assert_eq!(output_vector.middle(), 6);
        assert_eq!(output_vector.bottom(), 9);
    }

    #[test]
    fn matrix_by_vector_multiplication_then_addition_works_as_expected() {

        let input_matrix = CP2Matrix::new(
            [1, 2, 3],
            [4, 5, 6],
            [7, 8, 9]
        );
        let input_vector_1 = CP2Vector::new(
            10, 11, 12
        );
        let input_vector_2 = CP2Vector::new(
            2, 4, 6
        );

        let output_vector = input_matrix * input_vector_1 + input_vector_2;

        assert_eq!(output_vector.top(), 70);
        assert_eq!(output_vector.middle(), 171);
        assert_eq!(output_vector.bottom(), 272);
    }

    #[test]
    fn clarify_sign_extension_behaviour_when_going_from_i32_to_i64() {

        let i32_1 = -1_i32;
        let i32_2 = -1_i32;
        let i32_3 = -1_i32;

        let vector = CP2Vector::new(
            i32_1 as i64,
            i32_2 as i64,
            i32_3 as i64
        );

        assert_eq!(vector.top(), -1_i64);
        assert_eq!(vector.middle(), -1_i64);
        assert_eq!(vector.bottom(), -1_i64);
    }
}