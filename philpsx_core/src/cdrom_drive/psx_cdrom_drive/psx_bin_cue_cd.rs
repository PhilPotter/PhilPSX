// SPDX-License-Identifier: GPL-3.0
// psx_bin_cue_cd.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

use std::{
    error::Error,
    ffi::OsStr,
    fmt,
    fs::File,
    io::{
        BufReader, Read, Seek, SeekFrom
    },
    path::{
        MAIN_SEPARATOR_STR, Path
    },
};
use log::{
    log_enabled,
    Level,
};
use super::Cdrom;

use philpsx_utility::error::PhilPSXError;

/// This gap size represents the first initial two second gap before data begins.
const INITIAL_GAP_SIZE: usize = 2352 * 150; // Two seconds worth (at 75 frames per second).

/// This enum represents all possible track types.
/// It is intentionally incomplete for now to keep things simpler.
#[derive(Copy, Clone, Debug, PartialEq)]
enum PsxCdTrackType {
    AUDIO,
    MODE2_2352,
}

/// Lets us pretty print the track type.
impl fmt::Display for PsxCdTrackType {

    // Prints human-readable track type.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PsxCdTrackType::AUDIO => "Audio",
                PsxCdTrackType::MODE2_2352 => "Mode 2 (2352 byte sectors)",
            }
        )
    }
}

/// This enum represents the possible states whilst detecting tracks
/// from a cue file.
enum PsxCdTrackDetectionState {
    PreTrackListings,
    OnTrackLine,
    ReadingTrackIndexes,
    CreatingTrack,
}

/// This struct models a CD itself, and abstracts image type away from the emulator,
/// allowing different image types to be supported.
pub struct PsxBinCueCd {

    // This stores the file reference for the CD image.
    cd_file: BufReader<File>,

    // This stores the size of the CD image in bytes.
    cd_file_size: usize,

    // This allows us to keep a list of tracks from the image.
    track_list: Vec<PsxCdTrack>,
}

/// This struct models a track on the CD. Unlike for the original C implementation, we aren't
/// mapping the entire file with mmap as this requires unsafe and I want to see how far I can
/// get without it. For now, we simply use an optional buffered file descriptor and seek/read
/// using it.
struct PsxCdTrack {

    // Track properties.
    track_number: i32,
    track_type: PsxCdTrackType,
    track_pregap: Option<PsxCdTrackPregap>, // Here as a marker more than anything else.
    track_indexes: Vec<PsxCdTrackIndex>,
    offset: usize,
}

/// This struct models a track's various sections that appear as INDEX in a cue file.
struct PsxCdTrackIndex {

    // Index properties - these are the first and last byte index
    // within the CD, without offset applied.
    index_start: usize,
    index_end: usize,
}

/// Utility functions for the track indexes themselves.
impl PsxCdTrackIndex {

    /// This function tells us if the specified address is inside this index.
    fn contains(&self, address: usize) -> bool {
        address >= self.index_start && address <= self.index_end
    }
}

/// This struct models a track's pregap (if it has one).
#[derive(Copy, Clone)]
struct PsxCdTrackPregap {

    // Pregap properties.
    pregap_size: usize,
}

/// This type lets us store intermediate tracks without final end values etc.
struct IntermediateTrack {

    track_type: PsxCdTrackType,
    track_pregap: Option<PsxCdTrackPregap>,
    track_indexes: Vec<usize>,
}

/// Implementation functions for PsxBinCueCd.
impl PsxBinCueCd {

    /// This function opens a cue file specified by cd_path and then maps it.
    pub fn new(path: &OsStr) -> Result<Self, Box<dyn Error>> {

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
        let bin_size = get_bin_file_size(&mut bin_file_reader)?;

        // Now calculate our track listings properly.
        let track_list = get_track_listings(&cue_file_lines, bin_size)?;

        // Instantiate our CD.
        let cd = PsxBinCueCd {
            cd_file: bin_file_reader,
            cd_file_size: bin_size,
            track_list,
        };

        // Log out the state of the CD.
        if log_enabled!(Level::Debug) {
            log::debug!("CD: State of loaded CD: {}", cd);
        }

        Ok(cd)
    }
}

/// Allows us to display a loaded CD in nicely printed format.
impl fmt::Display for PsxBinCueCd {
    /// Print a CD object nicely.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let output_str = {
            // File is loaded, so construct nicely formatted properties.
            let mut displayed_cd_str = String::new();

            displayed_cd_str.push_str("{\n");
            displayed_cd_str.push_str(&format!(
                "  Size in bytes of emulated CD: {}\n",
                // Unwraps safe based on invariant.
                self.track_list
                    .last()
                    .unwrap()
                    .track_indexes
                    .last()
                    .unwrap()
                    .index_end
                    + 1
            ));
            displayed_cd_str.push_str(&format!("  Tracks: {}\n", self.track_list.len()));

            self.track_list.iter().for_each(|track| {
                // Print track number.
                displayed_cd_str.push_str(&format!(
                    "\n  Track {} ({}):\n",
                    track.track_number, track.track_type,
                ));

                // Print pregap info if there is one.
                if let Some(pregap) = track.track_pregap {
                    displayed_cd_str.push_str(&format!("  Pregap bytes: {}\n", pregap.pregap_size));
                }

                // Now print indexes.
                track.track_indexes.iter().for_each(|index| {
                    displayed_cd_str.push_str(&format!(
                        "  Index start: {}, Index end: {}\n",
                        index.index_start, index.index_end,
                    ));
                });
            });

            displayed_cd_str.push_str("}\n");

            displayed_cd_str
        };

        write!(f, "{}", &output_str)
    }
}

/// Functions to satisy Cdrom trait.
impl Cdrom for PsxBinCueCd {

    /// This function tells us the drive is loaded.
    fn is_loaded(&self) -> bool {
        true
    }

    /// This function reads a byte from the specified real CD address.
    fn read_byte(&mut self, address: usize) -> Result<u8, Box<dyn Error>> {

        // Iterate through all tracks and indexes until we find what we need.
        let track_containing_address = self.track_list
            .iter()
            .find(|track| {
                track.track_indexes
                    .iter()
                    .any(|index| index.contains(address))
            });

        // If we found the track, great, otherwise just return 0.
        match track_containing_address {

            Some(track) => {
                // Offset our address by the offset of the given track.
                let bin_address = address - track.offset;

                // Now read the byte.
                self.cd_file.seek(SeekFrom::Start(bin_address as u64))?;
                let mut byte_array = [0];
                self.cd_file.read_exact(&mut byte_array)?;

                Ok(byte_array[0])
            },

            None => Ok(0),
        }
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
            log::debug!("CD: Cue file contained UTF-8 BOM");
        },

        // BOM not found, set file position to start.
        _ => {
            log::debug!("CD: Cue file contained no UTF-8 BOM, resetting position...");
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
            .map(String::from)
            .collect()
    )
}

/// This function takes the original lines we read from the cue file, and then
/// trims them of whitespace and strips all empty lines.
fn sanitise_lines_from_cue(lines: &[String]) -> Vec<String> {

    lines
        .iter()
        .map(|line| String::from(line.trim()))
        .filter(|line| !line.is_empty())
        .collect()
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
fn get_bin_file_size(bin_file_reader: &mut BufReader<File>) -> Result<usize, Box<dyn Error>> {

    Ok(bin_file_reader.seek(SeekFrom::End(0))? as usize)
}

/// This function gets the track listings from the cue file. It doesn't
/// support anything other than basic Mode 2 2352 byte sectors at the moment,
/// as this is what the original works with. It also doesn't support anything
/// other than INDEX 01. PREGAP and INDEX 00 are not yet supported but will
/// be in due course.
fn get_track_listings(cue_file_lines: &[String], bin_size: usize) -> Result<Vec<PsxCdTrack>, Box<dyn Error>> {

    // Sanitise lines here first.
    let cue_file_lines = sanitise_lines_from_cue(cue_file_lines);

    // Variables here will track our state as we move through each line.
    let mut current_state = PsxCdTrackDetectionState::PreTrackListings;
    let mut current_track_type: Option<PsxCdTrackType> = None;
    let mut current_track_pregap: Option<PsxCdTrackPregap> = None;
    let mut current_indexes: Vec<usize> = vec![];
    let mut tracks: Vec<IntermediateTrack> = vec![];

    // This lambda creates a track for us in the intermediate format.
    // It allows us to not duplicate the code.
    let create_intermediate_track = |
        track_type: &Option<PsxCdTrackType>,
        pregrap: &Option<PsxCdTrackPregap>,
        indexes: &Vec<usize>
    | -> Result<IntermediateTrack, Box<dyn Error>> {

        // Unwrap the track type - it's an error if we didn't find it.
        let track_type = match track_type {
            Some(t) => Ok(*t),
            None => Err(PhilPSXError::error("CD: No track type when creating track")),
        }?;

        // Copy the pregap.
        let pregap = *pregrap;

        // Copy the indexes.
        if indexes.is_empty() {
            return Err(PhilPSXError::error("CD: Track with no indexes encountered"))
        }
        let indexes = indexes.clone();

        Ok(
            IntermediateTrack {
                track_type,
                track_pregap: pregap,
                track_indexes: indexes
            }
        )
    };
    
    // Use a very basic state machine here to construct the tracks.
    let mut line_index = 0;
    while line_index < cue_file_lines.len() {

        let current_line = &cue_file_lines[line_index];

        match current_state {

            // We are pre-track listings here.
            PsxCdTrackDetectionState::PreTrackListings => {
                if current_line.starts_with("TRACK") {
                    current_state = PsxCdTrackDetectionState::OnTrackLine;
                } else {
                    line_index += 1;
                }
            },

            // We are now on a track line.
            PsxCdTrackDetectionState::OnTrackLine => {
                current_track_type = Some(get_track_type(current_line)?);
                current_state = PsxCdTrackDetectionState::ReadingTrackIndexes;
                line_index += 1;
            },

            // We are now reading an index from a track.
            PsxCdTrackDetectionState::ReadingTrackIndexes => {

                if current_line.starts_with("PREGAP") {
                    // We found a pregap, which isn't part of the bin file.
                    // Modify the offset accordingly.
                    current_track_pregap = Some(
                        PsxCdTrackPregap {
                            pregap_size: get_pregap_size(current_line)?
                        }
                    );
                    line_index += 1;
                }
                else if current_line.starts_with("INDEX") {
                    // Read this index start into current vector.
                    current_indexes.push(
                        get_track_index_start(current_line)?
                    );
                    line_index += 1;
                } else if current_line.starts_with("TRACK") {
                    // We found the next track, mark us as done with this track.
                    current_state = PsxCdTrackDetectionState::CreatingTrack;
                } else {
                    // For anything else we should keep going.
                    line_index += 1;
                }
            },

            // We are now creating a track.
            PsxCdTrackDetectionState::CreatingTrack => {

                let track = create_intermediate_track(
                    &current_track_type,
                    &current_track_pregap,
                    &current_indexes
                )?;

                tracks.push(track);
                current_track_type = None;
                current_track_pregap = None;
                current_indexes.clear();
                current_state = PsxCdTrackDetectionState::OnTrackLine;
            },
        };
    }

    // Create the last track now.
    tracks.push(
        create_intermediate_track(
            &current_track_type,
            &current_track_pregap,
            &current_indexes
        )?
    );

    // Now we need to go through each track, adjust each MM:SS:FF byte address to the address
    // it would be on a real CD, and store this alongside the offset. Also, we need to calculate
    // end addresses of each index.
    let mut current_offset = INITIAL_GAP_SIZE;
    let final_tracks: Vec<PsxCdTrack> = tracks
        .iter()
        .enumerate()
        .map(|i_and_track| {
            let i = i_and_track.0;
            let track = i_and_track.1;

            // Adjust offset if a pregap is present.
            current_offset += match track.track_pregap {
                Some(track_pregap) => track_pregap.pregap_size,
                None => 0,
            };

            // Now adjust the indexes into the form we need.
            let track_indexes = track.track_indexes
                .iter()
                .enumerate()
                .map(|j_and_index| {
                    let j = j_and_index.0;
                    let index = j_and_index.1;

                    PsxCdTrackIndex {
                        index_start: index + current_offset,
                        index_end: if j == track.track_indexes.len() - 1 {

                            // This is the last index, we either need to get the beginning of the next track - 1 byte
                            // as our end position, or we need the BIN file size itself.
                            if i == tracks.len() - 1 {

                                // This is the last track, so use the BIN file size.
                                bin_size + current_offset
                            } else {

                                // We can get the beginning of the next track and use this. The next
                                // track might have a pregap, but it isn't part of the current offset (yet).
                                // Therefore, applying the current offset to the beginning of the next
                                // track's first index will in theory get us to the start address of
                                // the pregap itself (which we will -1 from at the end of the outer if).
                                let next_track = &tracks[i+1];
                                next_track.track_indexes[0] + current_offset
                            }
                        } else {
                            track.track_indexes[j+1] + current_offset
                        } - 1 // Subtract 1 as this is the last valid index of the track, not its size.
                    }
                })
                .collect();

            PsxCdTrack {
                track_number: (i as i32) + 1,
                track_type: track.track_type,
                track_pregap: track.track_pregap,
                track_indexes,
                offset: current_offset,
            }
        })
        .collect();

    Ok(final_tracks)
}

/// This function gets the track type of a TRACK line.
fn get_track_type(track_line: &String) -> Result<PsxCdTrackType, Box<dyn Error>> {

    if !track_line.starts_with("TRACK") {
        return Err(
            PhilPSXError::error(
                &format!("CD: Track type routine was supplied a non-TRACK line: {}", track_line)
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

    // For now, just handle MODE2/2352 and AUDIO types.
    match track_type_str {

        "AUDIO" => Ok(PsxCdTrackType::AUDIO),

        "MODE2/2352" => Ok(PsxCdTrackType::MODE2_2352),

        _ => Err(
            PhilPSXError::error(
                &format!("CD: Unrecognised track type: {}", track_type_str)
            )
        )
    }
}

/// This function gets the start position from an index line
fn get_track_index_start(index_line: &String) -> Result<usize, Box<dyn Error>> {

    if !index_line.starts_with("INDEX") {
        return Err(
            PhilPSXError::error(
                &format!("CD: Track index start routine was supplied a non-INDEX line: {}", index_line)
            )
        );
    }

    let index_time_str = index_line
        .split_whitespace()
        .last()
        .ok_or(
            PhilPSXError::error(
                &format!("CD: Could not get track index start from line: {}", index_line)
            )
        )?;

    get_bytes_from_mm_ss_ff(index_time_str)
}

/// This function gets the size from a PREGAP line.
fn get_pregap_size(pregap_line: &String) -> Result<usize, Box<dyn Error>> {

    if !pregap_line.starts_with("PREGAP") {
        return Err(
            PhilPSXError::error(
                &format!("CD: Track pregap size routine was supplied a non-PREGAP line: {}", pregap_line)
            )
        );
    }

    let pregap_time_str = pregap_line
        .split_whitespace()
        .last()
        .ok_or(
            PhilPSXError::error(
                &format!("CD: Could not get track index start from line: {}", pregap_line)
            )
        )?;

    get_bytes_from_mm_ss_ff(pregap_time_str)
}

/// This converts the minutes:seconds:frames format into bytes.
fn get_bytes_from_mm_ss_ff(time_str: &str) -> Result<usize, Box<dyn Error>> {

    // Now split into minutes, seconds and frames.
    let mins_secs_frames: Result<Vec<usize>, _> = time_str
        .split(":")
        .map(|value| value.parse::<usize>())
        .collect();
    let mins_secs_frames = mins_secs_frames?;

    // If we don't have exactly three segments, then bail.
    if mins_secs_frames.len() != 3 {
        return Err(
            PhilPSXError::error(
                &format!("CD: Time string could not be split and parsed: {}", time_str)
            )
        );
    }

    // Now calculate our start position - this currently only works for the sector types
    // already supported at the moment (AUDIO and Mode 2/2352).
    let start_position =
        mins_secs_frames[0] * 10584000 + // Minutes.
        mins_secs_frames[1] * 176400 +   // Seconds.
        mins_secs_frames[2] * 2352;      // Frames.

    Ok(start_position)
}

#[cfg(test)]
mod tests;
