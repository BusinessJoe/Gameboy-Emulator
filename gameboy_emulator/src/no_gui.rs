use gameboy_emulator::gameboy::GameBoyState;
use gameboy_emulator::joypad::JoypadInput;
use gameboy_emulator::ppu::TileColor;
use gameboy_emulator::{cartridge::Cartridge, gameboy::Interrupt};
use js_sys::Intl::Collator;
use log::*;
use strum::IntoEnumIterator;

use std::fs;
use std::time::{Duration, Instant};

use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to .gb rom file
    #[arg(short = 'r', long = "rom", required = true)]
    rom_path: String,

    /// Debug mode
    #[arg(short, long, default_value_t = false)]
    debug: bool,
}

fn main() -> Result<(), String> {
    env_logger::init();

    let args = Args::parse();

    let bytes = fs::read(args.rom_path).expect("could not read file");
    let cartridge = Cartridge::cartridge_from_data(&bytes).expect("failed to build cartridge");

    let mut gameboy_state = GameBoyState::new();
    gameboy_state
        .load_cartridge(cartridge)
        .map_err(|e| e.to_string())?;

    let joypad = gameboy_state.get_joypad();
    let memory_bus = gameboy_state.get_memory_bus();
    let ppu = gameboy_state.get_ppu();

    let mut total_cycles = 0;
    let mut frame_cycles = 0;
    let mut start = Instant::now();
    'mainloop: loop {
        // Tick gameboy for a frame
        let elapsed_cycles = gameboy_state.tick_for_frame();
        total_cycles += elapsed_cycles;
        frame_cycles += elapsed_cycles;

        // render_screen(gameboy_state.get_screen(), &mut canvas);

        // let duration = start.elapsed();
        // if duration > Duration::from_millis(1000 / 60) {
        //     warn!("Time elapsed this frame is: {:?} > 16ms", duration);
        // } else {
        //     std::thread::sleep(Duration::from_millis(1000 / 60) - duration);
        // }
        // start = Instant::now();
    }
    Ok(())
}
