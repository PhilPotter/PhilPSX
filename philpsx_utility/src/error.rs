// SPDX-License-Identifier: GPL-3.0
// error.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

use std::{
    error::Error,
    fmt,
    fmt::{
        Display,
        Formatter,
    },
};

/// This struct is a basic error type, for throwing custom errors from
/// places in the codebase where we aren't emiting a standard library error.
#[derive(Debug)]
pub struct PhilPSXError {

    // A string, just to hold the error.
    error_str: String,
}

impl Error for PhilPSXError {}

impl Display for PhilPSXError {

    /// Implements fmt so our error displays as expected.
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error_str)
    }
}

impl PhilPSXError {

    /// Create a new error.
    pub fn error(error_str: &str) -> Box<PhilPSXError> {
        Box::new(
            PhilPSXError {
                error_str: String::from(error_str),
            }
        )
    }
}