mod cpu;
mod instruction;
mod register;

use crate::cpu::register::Registers;
use crate::gameboy::MemoryBus;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct CPU {
    pub pc: u16,
    registers: Registers,
    sp: u16,
    interrupt_enabled: bool,
    halted: bool,
    halt_bug_opcode: Option<u8>,
    memory_bus: Arc<Mutex<MemoryBus>>,
}
