mod cpu;
mod execution_manager;
mod gameboy;
mod timer;

use crate::execution_manager::ExecutionManager;
use crate::gameboy::GameBoyState;
use std::env;
use std::io::stdin;

fn main() {
    env_logger::init();

    let rom_path = env::args().nth(1).expect("Expected a path to a rom");

    let mut gameboy = GameBoyState::new();
    gameboy.load(&rom_path);
    gameboy.cpu.boot();

    // Wrap gameboy in execution manager which begins execution
    let mut manager = ExecutionManager::new(gameboy);

    manager.add_breakpoint(49155);
    manager.remove_breakpoint(49155);
    manager.run();
}
