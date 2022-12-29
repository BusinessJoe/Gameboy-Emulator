mod cartridge;
mod cpu;
mod emulator;
mod gameboy;
mod memory;
mod ppu;
mod register;
mod screen;
mod timer;

use crate::emulator::GameboyEmulator;
use crate::gameboy::GameBoyState;
use std::env;

fn main() {
    env_logger::init();

    let rom_path = env::args().nth(1).expect("Expected a path to a rom");

    let mut gameboy = GameBoyState::new();
    gameboy.load(&rom_path);
    gameboy.cpu.boot();

    GameboyEmulator::run(gameboy);
}
