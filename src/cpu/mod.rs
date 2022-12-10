mod cpu;
mod instruction;
mod register;

use crate::cpu::register::Registers;
use std::rc::Rc;
use std::sync::Mutex;
use crate::gameboy::MemoryBus;

#[derive(Debug)]
pub struct CPU {
    pub pc: u16,
    registers: Registers,
    sp: u16,
    interrupt_enabled: bool,
    halted: bool,
    halt_bug_opcode: Option<u8>,
    memory_bus: Rc<Mutex<MemoryBus>>,
}

