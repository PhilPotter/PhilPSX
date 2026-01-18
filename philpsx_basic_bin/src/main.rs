// SPDX-License-Identifier: GPL-3.0
// lib.rs - Copyright Phillip Potter, 2026, under GPLv3 only.

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
use sdl3::pixels::Color;
use std::{
    ffi::OsString,
    process::ExitCode,
    time::Duration,
};

const PHILPSX_WINDOW_TITLE: &str = "PhilPSX - a Sony PlayStation 1 Emulator";
const PHILPSX_WINDOW_WIDTH: u32 = 1024;
const PHILPSX_WINDOW_HEIGHT: u32 = 768;

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

fn main() -> ExitCode {
    
    // Parse arguments and initialise logger.
    let philpsx_args = PhilPsxArgs::parse();
    colog::init();

    log::info!("PhilPSX initialising...");
    
    // Initialise SDL context.
    let sdl_context = match sdl3::init() {
        Ok(context) => context,
        Err(error) => {
            log::error!("Failed to initialise SDL: {}, exiting...", error);
            return ExitCode::FAILURE;
        },
    };

    // Initialise SDL video subsystem.
    let sdl_video_subsystem = match sdl_context.video() {
        Ok(video) => video,
        Err(error) => {
            log::error!("Failed to initialise SDL video subsystem: {}, exiting...", error);
            return ExitCode::FAILURE;
        },
    };

    let mut cdrom_drive = PsxCdromDrive::new();
    let mut controllers = PsxControllers::new();
    let mut cpu = R3051::new();
    let mut motherboard = PsxMotherboard::new(&philpsx_args.bios);
    let mut spu = PsxSpu::new();

    // Create a dummy window for now, just to make sure SDL works.
    let sdl_window = match sdl_video_subsystem.window(
        PHILPSX_WINDOW_TITLE,
        PHILPSX_WINDOW_WIDTH,
        PHILPSX_WINDOW_HEIGHT
    )
    .position_centered()
    .build() {
        Ok(window) => window,
        Err(error) => {
            log::error!("Failed to create SDL window: {}, exiting...", error);
            return ExitCode::FAILURE;
        },
    };

    // Display it.
    let mut sdl_canvas = sdl_window.into_canvas();
    sdl_canvas.set_draw_color(Color::RGB(0, 0, 255));
    sdl_canvas.clear();
    sdl_canvas.present();

    std::thread::sleep(Duration::from_secs(30));

    // We finished normally, return success code therefore.
    log::info!("PhilPSX exiting...");
    ExitCode::SUCCESS
}