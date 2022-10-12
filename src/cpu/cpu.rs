use crate::cpu::{instruction::*, register::*};
use std::fs;
use log::trace;

#[derive(Debug)]
pub struct CPU {
    pub registers: Registers,
    pub sp: u16,
    pub pc: u16,
    pub interrupt_enabled: bool,
    memory_bus: MemoryBus,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            registers: Registers::default(),
            sp: 0,
            pc: 0,
            interrupt_enabled: false,
            memory_bus: MemoryBus::default(),
        }
    }

    pub fn load(&mut self, filename: &str) {
        let bytes = fs::read(filename).unwrap();
        for (idx, b) in bytes.into_iter().enumerate() {
            self.set_memory_value(idx, b); 
        }
        trace!("{:#x}", &self.memory_bus.data[0x100]);
    }

    pub fn boot(&mut self) {
        // Initialize things.
        self.pc = 0x100;
        self.registers.a = 0x01;
        self.registers.f = 0xB0.into();
        self.set_word_register(WordRegister::BC, 0x0013);
        self.set_word_register(WordRegister::DE, 0x00D8);
        self.set_word_register(WordRegister::HL, 0x014D);
        self.sp = 0xFFFE;
    }

    pub fn tick(&mut self) {
        let pc = self.pc;
        let opcode = self.get_byte_from_pc();
        if opcode == 0xCB {
            let opcode = self.get_byte_from_pc();
            trace!("CB opcode {:#04x} at pc {:#06x}", opcode, pc);
            self.execute_cb_opcode(opcode);
        } else {
            trace!("opcode {:#04x} at pc {:#06x}", opcode, pc);
            self.execute_regular_opcode(opcode);
        }
        trace!("AF: {:#06x} BC: {:#06x} DE: {:#06x} HL: {:#06x} SP: {:#06x} PC: {:#06x}", self.registers.get_af(), self.registers.get_bc(), self.registers.get_de(), self.registers.get_hl(), self.sp, self.pc);
    }

    pub fn get_byte_from_pc(&mut self) -> u8 {
        let byte = self.get_memory_value(self.pc.into());
        trace!("Read byte {:#04x}", byte);
        self.pc += 1;
        byte
    }

    pub fn get_signed_byte_from_pc(&mut self) -> i8 {
        self.get_byte_from_pc() as i8
    }

    pub fn get_word_from_pc(&mut self) -> u16 {
        let bytes = [self.get_byte_from_pc(), self.get_byte_from_pc()];
        let word = u16::from_le_bytes(bytes);
        trace!("Read byte {:#06x}", word);
        word
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
        trace!("getting memory at {:#x}", address);
        self.memory_bus.get(address)
    }

    pub fn set_memory_value(&mut self, address: usize, value: u8) {
        trace!("setting memory at {:#x}", address);
        self.memory_bus.set(address, value);
    }

    pub fn push(&mut self, value: u8) {
        self.set_memory_value(self.sp as usize, value);
        self.sp -= 1;
    }

    pub fn pop(&mut self) -> u8 {
        self.sp += 1;
        self.get_memory_value(self.sp as usize)
    }
}

/// Mock memory bus
#[derive(Debug)]
struct MemoryBus {
    pub data: [u8; 0x10000],
}

impl Default for MemoryBus {
    fn default() -> Self {
        Self { data: [0; 0x10000] }
    }
}

impl MemoryBus {
    pub fn get(&self, address: usize) -> u8 {
        self.data[address]
    }

    pub fn set(&mut self, address: usize, value: u8) {
        if address == 0xFF02 && value == 0x81 {
            let chr = char::from_u32(self.data[0xFF01] as u32).unwrap();
            println!("{}", chr);
        }
        self.data[address] = value;
    }
}
