mod cpu;
mod timer;
mod gameboy;

use crate::gameboy::GameBoyState;
use std::env;
use std::io::stdin;

fn main() {
    env_logger::init();

    let rom_path = env::args().nth(1).unwrap();

    let mut gameboy = GameBoyState::new();
    gameboy.load(&rom_path);
    gameboy.cpu.boot();

    let mut target: Option<u16> = None;
    loop {
        gameboy.tick();
        let mut string = String::new();
        if target.is_some() && gameboy.cpu.pc == target.unwrap() {
            target = None;
        }
        if target.is_none() {
            stdin().read_line(&mut string);
            // remove newline
            string.pop();
            let without_prefix = string.trim_start_matches("0x");
            target = u16::from_str_radix(without_prefix, 16).ok();
        }
    }
}
