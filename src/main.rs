use gameboy_emulator::emulator::GameboyEmulator;
use gameboy_emulator::gameboy::GameBoyState;
use std::env;

fn main() {
    env_logger::init();

    let rom_path = env::args().nth(1).expect("Expected a path to a rom");

    let mut gameboy = GameBoyState::new();
    gameboy.load(&rom_path).unwrap();
    gameboy.cpu.borrow_mut().boot();

    GameboyEmulator::run(gameboy);
}
