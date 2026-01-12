// SPDX-License-Identifier: GPL-3.0
// psx_cd.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

use std::{
    fs::File,
    io::BufReader,
};

/// This struct models a CD itself, and abstracts image type away from the emulator,
/// allowing different image types to be supported.
pub struct PsxCd {

    // This stores the file reference for the CD image.
    cd_file: Option<BufReader<File>>,

    // This stores the size of the CD image in bytes.
    cd_file_size: u64,

    // This allows us to keep a list of tracks from the image.
    track_list: Vec<PsxCdTrack>,
}

/// This struct models a track on the CD. Unlike for the original C implementation, we aren't
/// mapping the entire file with mmap as this requires unsafe and I want to see how far I can
/// get without it. For now, we simply use an optional buffered file descriptor and seek/read
/// using it.
pub struct PsxCdTrack {

    // Track properties.
    track_number: i32,
    track_type: i32,
    track_start: i64,
    track_end: i64,
    offset: i64,
}

/// Implementation functions for PsxCd.
impl PsxCd {

    /// Creates a new CD object with the correct initial state.
    pub fn new() -> Self {
        PsxCd {

            // Set file descriptor as unset for now.
            cd_file: None,

            // Set file size as 0 for now.
            cd_file_size: 0,

            // Set track listing as empty for now.
            track_list: vec![],
        }
    }
}