use std::{thread, time};
mod cpu;
use cpu::CPU;


fn main() {
    env_logger::init();

    let mut cpu = CPU::new();
    cpu.load("cpu_instrs/cpu_instrs.gb");
    cpu.boot();

    loop {
        cpu.tick();
        thread::sleep(time::Duration::from_millis(10));
    }
}
