// SPDX-License-Identifier: GPL-3.0
// psx_cd.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

use std::{
    error::Error,
    ffi::OsStr,
    fs::File,
    io::{
        BufReader,
        Read,
        SeekFrom,
        Seek,
    },
    path::Path,
};

use philpsx_utility::error::PhilPSXError;

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

    /// This function tells us if the CD object is currently associated with a
    /// loaded image file.
    pub fn is_empty(&self) -> bool {
        self.cd_file.is_none()
    }

    /// This function opens a cue file specified by cd_path and then maps it.
    /// In future, it will support other image types as well.
    pub fn load_cd(
        &mut self,
        path: &OsStr
    ) -> Result<(), Box<dyn Error>> {

        log::info!("CD: Loading CD image...");

        // First do some basic validation on the path.
        if path.is_empty() {
            return Err(PhilPSXError::error("CD: Provided CD path was empty"));
        }
        else if path.len() < 5 {
            return Err(
                PhilPSXError::error(
                    concat!(
                        "CD: ",
                        "Provided CD path was too short and therefore ",
                        "cannot be a cue file of the form x.cue or x.CUE"
                    )
                )
            );
        }

        let string_path = path.to_str().ok_or(
            PhilPSXError::error("CD: Provided CD path could not be converted to a string slice")
        )?;
        if !string_path.to_ascii_lowercase().ends_with(".cue") {
            return Err(PhilPSXError::error("CD: Provided CD path was not a cue file"));
        }

        log::info!("CD: Cue file path is {}", string_path);

        // Open the cue file.
        let mut cue_file = File::open(Path::new(path))?;

        // Handle the UTF-8 BOM in the cue file if present.
        handle_utf8_bom(&mut cue_file)?;

        Ok(())
    }
}

/// This function detects a UTF-8 BOM and moves the file position past it if
/// need be. If this process fails, we bubble up the error.
fn handle_utf8_bom(cue_file: &mut File) -> Result<(), Box<dyn Error>> {

    // First, set position to start of cue file.
    cue_file.seek(SeekFrom::Start(0))?;

    // Now declare a three bye array and check for the BOM. If it's there,
    // we will thus move past it.
    let mut bom_array = [0u8; 3];
    let bytes_read = cue_file.read(&mut bom_array)?;
    if bytes_read != 3 {
        return Err(PhilPSXError::error("CD: Could not check for UTF-8 BOM, unable to read cue file"));
    }
    match bom_array {

        // BOM found, do nothing.
        [0xEF, 0xBB, 0xBF] => {
            log::debug!("CD: cue file contained UTF-8 BOM");
        },

        // BOM not found, set file position to start.
        _ => {
            log::debug!("CD: cue file contained no UTF-8 BOM, resetting position...");
            cue_file.seek(SeekFrom::Start(0))?;
        },
    };

    Ok(())
}