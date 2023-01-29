use gameboy_emulator::emulator::GameboyEmulator;
use gameboy_emulator::cartridge::build_cartridge;
use std::env;
use std::fs;

fn main() -> Result<(), ()> {
    env_logger::init();

    let rom_path = env::args().nth(1).expect("expected a path to a rom");
    let bytes = fs::read(rom_path).expect("could not read file");
    let cartridge = build_cartridge(&bytes).expect("failed to build cartridge");

    GameboyEmulator::run(cartridge)?;

    Ok(())
}
