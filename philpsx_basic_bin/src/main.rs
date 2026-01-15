// SPDX-License-Identifier: GPL-3.0
// lib.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

use std::ffi::OsString;

// This file is the core of the basic client - it exists merely as a CLI-based
// program to load in a BIOS file and CD image. In due course, other UIs will
// likely come with full GUI support.

use clap::Parser;
use philpsx_core::{
    cdrom_drive::psx_cdrom_drive::PsxCdromDrive,
    controllers::psx_controllers::PsxControllers,
    cpu::r3051::R3051,
    motherboard::psx_motherboard::PsxMotherboard,
    spu::psx_spu::PsxSpu,
};

#[derive(Parser)]
#[command(
    version,
    about = "A basic barebones UI for the PhilPSX emulator",
    long_about = None
)]
struct PhilPsxArgs {
    #[arg(
        long = "cd",
        help = "An optional CD Cue file",
        id = "Cue file"
    )]
    cd: Option<OsString>,

    #[arg(
        long = "bios",
        help = "A compatible PS1 BIOS file",
        id = "BIOS file"
    )]
    bios: OsString,
}

fn main() {
    let philpsx_args = PhilPsxArgs::parse();

    let mut cdrom_drive = PsxCdromDrive::new();
    let mut controllers = PsxControllers::new();
    let mut cpu = R3051::new();
    let mut motherboard = PsxMotherboard::new(&philpsx_args.bios);
    let mut spu = PsxSpu::new();
}