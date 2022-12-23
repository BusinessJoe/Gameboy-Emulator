use crate::cpu::{instruction::*, register::*};
use crate::gameboy::MemoryBus;
use log::{debug, info, trace};
use std::sync::{Arc, Mutex};

use crate::cpu::CPU;

impl CPU {
    pub fn new(memory_bus: Arc<Mutex<MemoryBus>>) -> Self {
        Self {
            registers: Registers::default(),
            sp: 0,
            pc: 0,
            interrupt_enabled: false,
            halted: false,
            halt_bug_opcode: None,
            memory_bus,
        }
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

    /// Called at the beginning of an interrupt helper
    fn handle_single_interrupt(&mut self, bit: u8, address: u16) {
        // Check IME flag and relevant bit in IE flag.
        let ie_flag = self.get_memory_value(0xFFFF);
        let ie_flag_bit = (ie_flag >> bit) & 1;
        if self.interrupt_enabled && ie_flag_bit == 1 {
            debug!(
                "handling interrupt {}",
                match bit {
                    0 => "vblank",
                    1 => "lcdc",
                    2 => "timer",
                    3 => "serial",
                    4 => "joypad",
                    _ => "UNKNOWN",
                }
            );

            // Reset IF bit and IME flag
            self.set_memory_value(0xFF0F, ie_flag & !(1 << bit));
            self.interrupt_enabled = false;

            // Push PC onto stack. LSB is last/top of the stack.
            let bytes = self.pc.to_le_bytes();
            self.push(bytes[1]);
            self.push(bytes[0]);

            // Jump to starting address of interrupt
            self.pc = address;
        } else {
            debug!(
                "ignoring interrupt {}",
                match bit {
                    0 => "vblank",
                    1 => "lcdc",
                    2 => "timer",
                    3 => "serial",
                    4 => "joypad",
                    _ => "UNKNOWN",
                }
            );
        }
    }

    fn handle_interrupts(&mut self) {
        // If IE and IF
        if self.get_memory_value(0xFFFF) & self.get_memory_value(0xFF0F) != 0 {
            if self.interrupt_enabled {
                // Unhalt
                self.halted = false;

                // Handle interrupts by priority (starting at bit 0 - V-Blank)
                for bit in 0..=4 {
                    let address = 0x40 + bit * 0x8;
                    self.handle_single_interrupt(bit, address.into());
                }
            } else {
                // Wake up, but don't handle any interrupts.
                self.halted = false;
            }
        }
    }

    fn execute_opcode(&mut self, opcode: u8) -> u8 {
        let elapsed_cycles;
        if opcode == 0xCB {
            let opcode = self.get_byte_from_pc();
            elapsed_cycles = self.execute_cb_opcode(opcode);
        } else {
            elapsed_cycles = self.execute_regular_opcode(opcode);
        }

        self.handle_interrupts();

        elapsed_cycles
    }

    /// Handle a single timestep in the cpu, returning the number of elapsed cycles
    pub fn tick(&mut self) -> u8 {
        let elapsed_cycles = if !self.halted {
            // Get and execute opcode
            let pc = self.pc;
            let opcode = self.get_byte_from_pc();

            // Decodes and executes opcode, returns number of elapsed cycles.
            self.execute_opcode(opcode)
        } else {
            debug!("Halted");
            // Return 1 cycle
            1
        };

        self.handle_interrupts();

        elapsed_cycles
    }

    pub fn get_byte_from_pc(&mut self) -> u8 {
        match self.halt_bug_opcode {
            Some(opcode) => {
                self.halt_bug_opcode = None;
                debug!("Read halt bug byte {:#04x}", opcode);
                opcode
            }
            None => {
                let byte = self.get_memory_value(self.pc.into());
                debug!("Read byte {:#04x}", byte);
                self.pc += 1;
                byte
            }
        }
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
        self.memory_bus.lock().unwrap().get(address)
    }

    pub fn set_memory_value(&mut self, address: usize, value: u8) {
        trace!("setting memory at {:#x}", address);
        self.memory_bus.lock().unwrap().set(address, value);
    }

    pub fn push(&mut self, value: u8) {
        self.sp -= 1;
        self.set_memory_value(self.sp as usize, value);
    }

    pub fn pop(&mut self) -> u8 {
        let value = self.get_memory_value(self.sp as usize);
        self.sp += 1;
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gameboy::MemoryBus;

    #[test]
    fn test_increment_b() {
        let memory_bus = Arc::new(Mutex::new(MemoryBus::default()));
        let mut cpu = CPU::new(memory_bus);
        cpu.boot();
        let b_reg = cpu.registers.b;
        cpu.execute_opcode(0x04);
        assert_eq!(b_reg + 1, cpu.registers.b);
    }

    #[test]
    fn halt_bug() {
        let memory_bus = Arc::new(Mutex::new(MemoryBus::default()));
        let mut cpu = CPU::new(memory_bus.clone());
        cpu.boot();

        // The halt bug requires IME to be reset and IE & IF =/= 0.
        cpu.interrupt_enabled = false;
        memory_bus.lock().unwrap().set(0xFFFF, 0xFF);
        memory_bus.lock().unwrap().set(0xFF0F, 0xFF);

        // Set up an increment B instruction at the current opcode.
        // We expect this to be executed twice due to the halt bug.
        let pc = cpu.pc;
        memory_bus.lock().unwrap().set(pc.into(), 0x04);

        // Store the initial value of the B register so we can check
        // that it increments twice.
        let b_reg = cpu.registers.b;

        // Execute halt
        cpu.execute_opcode(0x76);

        cpu.tick();
        cpu.tick();

        assert_eq!(b_reg + 2, cpu.registers.b);

        // The next instruction should run normally (a NOP in this case).
        // Check that the increment doesn't execute again.
        cpu.tick();
        assert_eq!(b_reg + 2, cpu.registers.b);
    }
}
