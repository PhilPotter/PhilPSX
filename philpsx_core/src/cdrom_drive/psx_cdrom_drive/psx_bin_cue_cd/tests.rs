// SPDX-License-Identifier: GPL-3.0
// tests.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

use super::{
    super::psx_bin_cue_cd,
    PsxCdTrackType,
};

// Constants for a single data track CD.
const SINGLE_DATA_TRACK_CUE_CONTENTS: &str = r#"
FILE "Single Data Track.bin" BINARY
  TRACK 01 MODE2/2352
    INDEX 01 00:00:00
"#;
const SINGLE_DATA_TRACK_BIN_SIZE: usize = 10760400;

// Constants for a multi track CD with both data and audio, and a pregap.
const MULTI_TRACK_CUE_CONTENTS: &str = r#"
FILE "Multi Track.bin" BINARY
  TRACK 01 MODE2/2352
    INDEX 01 00:00:00
  TRACK 02 AUDIO
    PREGAP 00:02:00
    INDEX 01 10:02:01
  TRACK 03 AUDIO
    INDEX 00 14:26:00
    INDEX 01 14:28:00
  TRACK 04 AUDIO
    INDEX 00 17:02:25
    INDEX 01 17:04:25
"#;
const MULTI_TRACK_BIN_SIZE: usize = 286532400;

// Tests for the Bin/Cue CD implementation.

#[test]
fn test_parsing_of_one_data_track_success() {

    // Given a single data track starting at 00:00:00,
    // parsing it should lead to the correct layout.
    let lines: Vec<String> = SINGLE_DATA_TRACK_CUE_CONTENTS
            .lines()
            .map(String::from)
            .collect();
    let track_listings = psx_bin_cue_cd::get_track_listings(
        &lines,
        SINGLE_DATA_TRACK_BIN_SIZE
    );

    // Check tracks properties.
    match track_listings {
        Ok(track_listings) => {
            assert_eq!(track_listings.len(), 1);
            assert_eq!(track_listings[0].track_number, 1);
            assert_eq!(track_listings[0].track_type, PsxCdTrackType::MODE2_2352);
            assert!(track_listings[0].track_pregap.is_none());
            assert_eq!(track_listings[0].track_indexes.len(), 1);
            assert_eq!(track_listings[0].track_indexes[0].index_start, 352800);
            assert_eq!(track_listings[0].track_indexes[0].index_end, 11113199);
            assert_eq!(track_listings[0].offset, 352800);
        },

        Err(error) => {
            panic!("Couldn't get track listings: {}", error);
        },
    }
}

#[test]
fn test_parsing_of_multi_track_success() {

    // Given a CD with a data track followed by multiple audio tracks
    // with a pregap, parsing it should lead to the correct layout.
    let lines: Vec<String> = MULTI_TRACK_CUE_CONTENTS
            .lines()
            .map(String::from)
            .collect();
    let track_listings = psx_bin_cue_cd::get_track_listings(
        &lines,
        MULTI_TRACK_BIN_SIZE
    );

    // Check tracks properties.
    match track_listings {
        Ok(track_listings) => {
            assert_eq!(track_listings.len(), 4);

            assert_eq!(track_listings[0].track_number, 1);
            assert_eq!(track_listings[0].track_type, PsxCdTrackType::MODE2_2352);
            assert!(track_listings[0].track_pregap.is_none());
            assert_eq!(track_listings[0].track_indexes.len(), 1);
            assert_eq!(track_listings[0].track_indexes[0].index_start, 352800);
            assert_eq!(track_listings[0].track_indexes[0].index_end, 106547951);
            assert_eq!(track_listings[0].offset, 352800);

            assert_eq!(track_listings[1].track_number, 2);
            assert_eq!(track_listings[1].track_type, PsxCdTrackType::AUDIO);
            match track_listings[1].track_pregap {
                Some(pregap) => {
                    assert_eq!(pregap.pregap_size, 352800);
                },
                None => panic!("No pregap for track 2"),
            };
            assert_eq!(track_listings[1].track_indexes.len(), 1);
            assert_eq!(track_listings[1].track_indexes[0].index_start, 106900752);
            assert_eq!(track_listings[1].track_indexes[0].index_end, 153467999);
            assert_eq!(track_listings[1].offset, 705600);

            assert_eq!(track_listings[2].track_number, 3);
            assert_eq!(track_listings[2].track_type, PsxCdTrackType::AUDIO);
            assert!(track_listings[2].track_pregap.is_none());
            assert_eq!(track_listings[2].track_indexes.len(), 2);
            assert_eq!(track_listings[2].track_indexes[0].index_start, 153468000);
            assert_eq!(track_listings[2].track_indexes[0].index_end, 153820799);
            assert_eq!(track_listings[2].track_indexes[1].index_start, 153820800);
            assert_eq!(track_listings[2].track_indexes[1].index_end, 181045199);
            assert_eq!(track_listings[2].offset, 705600);

            assert_eq!(track_listings[3].track_number, 4);
            assert_eq!(track_listings[3].track_type, PsxCdTrackType::AUDIO);
            assert!(track_listings[3].track_pregap.is_none());
            assert_eq!(track_listings[3].track_indexes.len(), 2);
            assert_eq!(track_listings[3].track_indexes[0].index_start, 181045200);
            assert_eq!(track_listings[3].track_indexes[0].index_end, 181397999);
            assert_eq!(track_listings[3].track_indexes[1].index_start, 181398000);
            assert_eq!(track_listings[3].track_indexes[1].index_end, 287237999);
            assert_eq!(track_listings[3].offset, 705600);
        },

        Err(error) => {
            panic!("Couldn't get track listings: {}", error);
        },
    }
}