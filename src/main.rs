use gameboy_emulator::emulator::GameboyEmulator;
use gameboy_emulator::cartridge::build_cartridge;
use std::fs;

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

fn main() -> Result<(), ()> {
    env_logger::init();

    let args = Args::parse();

    let bytes = fs::read(args.rom_path).expect("could not read file");
    let cartridge = build_cartridge(&bytes).expect("failed to build cartridge");

    GameboyEmulator::run(cartridge, args.debug)?;

    Ok(())
}
