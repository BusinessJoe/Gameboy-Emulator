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
    fn _handle_interrupt(&mut self, bit: u8, address: u16) {
        // Check IME flag and relevant bit in IE flag.
        let ie_flag = self.get_memory_value(0xFFFF);
        if self.interrupt_enabled && ((ie_flag >> bit) & 1 == 1) {
            debug!(
                "handling interrupt {}",
                match bit {
                    0 => "vblank",
                    1 => "lcdc",
                    2 => "timer",
                    3 => "serial",
                    4 => "high to low",
                    _ => "UNKNOWN",
                }
            );

            // Reset IF flag
            self.set_memory_value(0xFF0F, ie_flag & !(1 << bit));

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
                    4 => "high to low",
                    _ => "UNKNOWN",
                }
            );
        }
    }

    fn handle_vblank(&mut self) {}
    fn handle_lcdc(&mut self) {}
    fn handle_timer(&mut self) {
        self._handle_interrupt(2, 0x0050)
    }
    fn handle_serial_transfer_connection(&mut self) {}
    fn handle_high_to_low_p10_to_p13(&mut self) {}

    fn handle_interrupts(&mut self) {
        if self.interrupt_enabled {
            // If IE and IF
            if self.get_memory_value(0xFFFF) & self.get_memory_value(0xFF0F) != 0 {
                // Unhalt
                debug! {"UNHALT"};
                self.halted = false;

                // Handle interrupts by priority (starting at bit 0 - V-Blank)

                // V-Blank
                self.handle_vblank();

                // LCDC Status
                self.handle_lcdc();

                // Timer Overflow
                self.handle_timer();

                // Serial Transfer Connection
                self.handle_serial_transfer_connection();

                // High-to-Low of P10-P13
                self.handle_high_to_low_p10_to_p13();
            }
        }
    }

    /// Handle a single timestep in the cpu
    pub fn tick(&mut self) -> u8 {
        self.handle_interrupts();

        if !self.halted {
            // Get and execute opcode
            let pc = self.pc;
            let opcode = self.get_byte_from_pc();
            let elapsed_cycles;
            if opcode == 0xCB {
                let opcode = self.get_byte_from_pc();
                debug!("CB opcode {:#04x} at pc {:#06x}", opcode, pc);
                elapsed_cycles = self.execute_cb_opcode(opcode);
            } else {
                debug!("opcode {:#04x} at pc {:#06x}", opcode, pc);
                elapsed_cycles = self.execute_regular_opcode(opcode);
            }
            trace!(
                "AF: {:#06x} BC: {:#06x} DE: {:#06x} HL: {:#06x} SP: {:#06x} PC: {:#06x}",
                self.registers.get_af(),
                self.registers.get_bc(),
                self.registers.get_de(),
                self.registers.get_hl(),
                self.sp,
                self.pc
            );
            elapsed_cycles
        } else {
            debug!("Halted");
            // Return 1 cycle
            1
        }
    }

    pub fn get_byte_from_pc(&mut self) -> u8 {
        match self.halt_bug_opcode {
            Some(opcode) => {
                self.halt_bug_opcode = None;
                trace!("Read halt bug byte {:#04x}", opcode);
                opcode
            }
            None => {
                let byte = self.get_memory_value(self.pc.into());
                trace!("Read byte {:#04x}", byte);
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
