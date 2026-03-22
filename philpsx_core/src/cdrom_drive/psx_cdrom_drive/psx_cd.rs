// SPDX-License-Identifier: GPL-3.0
// psx_cd.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

use std::{
    error::Error,
    ffi::OsStr,
    fs::File,
    io::{
        BufReader, Read, Seek, SeekFrom
    },
    path::{
        MAIN_SEPARATOR_STR, Path
    },
};

use philpsx_utility::error::PhilPSXError;

/// This enum represents all possible track types.
/// It is intentionally incomplete for now to keep things simpler.
enum PsxCdTrackType {
    AUDIO,
    MODE2_2352,
}

/// This enum represents the possible states whilst detecting tracks
/// from a cue file.
enum PsxCdTrackDetectionState {
    PRE_TRACK_LISTINGS,
    ON_TRACK_LINE,
}

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
    track_type: PsxCdTrackType,
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
        let cue_file_path = Path::new(path);
        let mut cue_file = File::open(cue_file_path)?;

        // Handle the UTF-8 BOM in the cue file if present.
        handle_utf8_bom(&mut cue_file)?;

        // Now just read all the lines from the cue file - it's fine to just pull them
        // all into memory.
        let cue_file_lines = get_lines_from_cue(&mut cue_file)?;

        // Now that we have the lines, let's parse them to get our BIN file and track listing.
        let mut bin_file_reader = get_bin_file_reader(&cue_file_lines, cue_file_path)?;

        // Get our BIN file size.
        let bin_file_size = get_bin_file_size(&mut bin_file_reader)?;

        // Now calculate our track listings properly.
        let track_listings = get_track_listings(&cue_file_lines);

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

/// This function reads all lines from the cue file and returns them, or an error.
fn get_lines_from_cue(cue_file: &mut File) -> Result<Vec<String>, Box<dyn Error>> {

    // Allocate a new string and read into it.
    let mut cue_contents = String::new();
    cue_file.read_to_string(&mut cue_contents)?;

    // Now split the string into lines, and return it back.
    Ok(
        cue_contents
            .lines()
            .map(|line| String::from(line.trim()))
            .filter(|line| !line.is_empty())
            .collect()
    )
}

/// This function reads our BIN file path from the cue file and returns a BufReader
/// if found, or an error.
fn get_bin_file_reader(cue_file_lines: &[String], cue_file_path: &Path) -> Result<BufReader<File>, Box<dyn Error>> {

    // Find our FILE line.
    let file_line = cue_file_lines
        .iter()
        .find(|&line| line.starts_with("FILE"))
        .ok_or(PhilPSXError::error("CD: Could not find FILE line in cue file"))?;

    // Now parse the file path itself.
    let mut file_line_components = file_line
        .split('"');

    let bin_file_path_str = file_line_components
        .nth(1)
        .ok_or(PhilPSXError::error("CD: Could not find FILE path within line in cue file"))?;

    let is_binary = file_line_components
        .next()
        .ok_or(PhilPSXError::error("CD: Could not read FILE type within line in cue file"))?
        .contains("BINARY");

    if !is_binary {
        return Err(PhilPSXError::error("CD: FILE type was not binary"));
    }

    // Now lets actually open the file.
    let bin_file_path = Path::new(bin_file_path_str);
    let bin_file = if bin_file_path.is_absolute() {
        File::open(bin_file_path)
    } else {

        // Path is relative (as expected). Combine it.
        let cue_file_parent_str = cue_file_path
            .parent()
            .ok_or(PhilPSXError::error("CD: Could not get parent of cue file"))?
            .to_str()
            .ok_or(PhilPSXError::error("CD: Could not represent cue file path as UTF-8 string"))?;

        let combined_path_str = format!("{}{}{}", cue_file_parent_str, MAIN_SEPARATOR_STR, bin_file_path_str);
        let combined_path = Path::new(&combined_path_str);

        // Now open our new path.
        File::open(combined_path)
    }?;

    Ok(BufReader::new(bin_file))
}

/// This function calculates the size of our BIN file.
fn get_bin_file_size(bin_file_reader: &mut BufReader<File>) -> Result<u64, Box<dyn Error>> {

    Ok(bin_file_reader.seek(SeekFrom::End(0))?)
}

/// This function gets the track listings from the cue file. It doesn't
/// support anything other than basic Mode 2 2352 byte sectors at the moment,
/// as this is what the original works with. It also doesn't support anything
/// other than INDEX 01. PREGAP and INDEX 00 are not yet supported but will
/// be in due course.
fn get_track_listings(cue_file_lines: &[String]) -> Result<u64, Box<dyn Error>> {

    // Variables here will track our state as we move through each line.
    let mut current_state = PsxCdTrackDetectionState::PRE_TRACK_LISTINGS;
    let current_track_offset = 2352 * 150; // Two seconds worth (at 75 frames per second).
    
    // Use a very basic state machine here to construct the tracks.
    let mut line_index = 0;
    while line_index < cue_file_lines.len() {
        match current_state {
            
            // We are pre-track listings here.
            PsxCdTrackDetectionState::PRE_TRACK_LISTINGS => {
                if cue_file_lines[line_index].starts_with("TRACK") {
                    current_state = PsxCdTrackDetectionState::ON_TRACK_LINE;
                } else {
                    line_index += 1;
                }
            },

            // We are now on a track line.
            PsxCdTrackDetectionState::ON_TRACK_LINE => {

            },
        };
    }

    Ok(0)
}

/// This function gets the track type of a TRACK line.
fn get_track_type(track_line: &String) -> Result<PsxCdTrackType, Box<dyn Error>> {

    if !track_line.starts_with("TRACK") {
        return Err(
            PhilPSXError::error(
                &format!("CD: Track type routine was supplied non TRACK line: {}", track_line)
            )
        );
    }

    let track_type_str = track_line
        .split_whitespace()
        .last()
        .ok_or(
            PhilPSXError::error(
                &format!("CD: Could not get track type from line: {}", track_line)
            )
        )?;

    match track_type_str {
        
        // Only care about Mode 2 2352 byte sectors for now.
        "MODE2/2352" => Ok(PsxCdTrackType::MODE2_2352),

        _ => Err(
            PhilPSXError::error(
                &format!("CD: Unrecognised track type: {}", track_type_str)
            )
        )
    }
}