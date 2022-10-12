use std::io::stdin;
mod cpu;
use cpu::CPU;
use std::env;


fn main() {
    env_logger::init();

    let rom_path = env::args().nth(1).unwrap();

    let mut cpu = CPU::new();
    cpu.load(&rom_path);
    cpu.boot();

    let mut target: Option<u16> = None;
    loop {
        cpu.tick();
        let mut string = String::new();
        if target.is_some() && cpu.pc == target.unwrap() {
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
