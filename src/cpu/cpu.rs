use crate::cpu::{instruction::*, register::*};

#[derive(Debug)]
pub struct CPU {
    pub registers: Registers,
    pub sp: u16,
    pub pc: u16,
    stack: [u8; 1024],
    memory_bus: MemoryBus,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            registers: Registers::default(),
            stack: [0; 1024],
            sp: 0,
            pc: 0,
            memory_bus: MemoryBus::default(),
        }
    }

    pub fn boot(&mut self) {
        // Initialize stack pointer.
        // And do other things :)
        todo!();
    }

    pub fn set_register(&mut self, reg: Register, value: u8) {
        match reg {
            Register::A => self.registers.a = value,
            Register::B => self.registers.b = value,
            Register::C => self.registers.c = value,
            Register::D => self.registers.d = value,
            Register::E => self.registers.e = value,
            Register::H => self.registers.h = value,
            Register::L => self.registers.l = value,
        }
    }

    pub fn set_word_register(&mut self, word_reg: WordRegister, value: u16) {
        match word_reg {
            WordRegister::AF => self.registers.set_af(value),
            WordRegister::BC => self.registers.set_bc(value),
            WordRegister::DE => self.registers.set_de(value),
            WordRegister::HL => self.registers.set_hl(value),
            WordRegister::SP => self.sp = value,
            WordRegister::PC => self.pc = value,
        }
    }

    pub fn get_register(&self, reg: Register) -> u8 {
        match reg {
            Register::A => self.registers.a,
            Register::B => self.registers.b,
            Register::C => self.registers.c,
            Register::D => self.registers.d,
            Register::E => self.registers.e,
            Register::H => self.registers.h,
            Register::L => self.registers.l,
        }
    }

    pub fn get_word_register(&self, word_reg: WordRegister) -> u16 {
        match word_reg {
            WordRegister::AF => self.registers.get_af(),
            WordRegister::BC => self.registers.get_bc(),
            WordRegister::DE => self.registers.get_de(),
            WordRegister::HL => self.registers.get_hl(),
            WordRegister::SP => self.sp,
            WordRegister::PC => self.pc,
        }
    }

    pub fn get_memory_value(&self, address: usize) -> u8 {
        self.memory_bus.get(address)
    }

    pub fn set_memory_value(&mut self, address: usize, value: u8) {
        self.memory_bus.set(address, value);
    }

    pub fn push(&mut self, value: u8) {
        self.stack[self.sp as usize] = value;
        self.sp += 1;
    }

    pub fn pop(&mut self) -> u8 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }
}

/// Mock memory bus
#[derive(Debug)]
struct MemoryBus {
    data: [u8; 4096],
}

impl Default for MemoryBus {
    fn default() -> Self {
        Self { data: [0; 4096] }
    }
}

impl MemoryBus {
    pub fn get(&self, address: usize) -> u8 {
        self.data[address]
    }

    pub fn set(&mut self, address: usize, value: u8) {
        self.data[address] = value;
    }
}
