use std::collections::VecDeque;

use crate::component::{Addressable, ElapsedTime, Steppable};
use crate::cpu::{instruction::*, register::*};
use crate::error::Result;
use crate::memory::MemoryBus;
use log::{debug, info};

pub struct Cpu {
    pub registers: Registers,
    pub sp: u16,
    pub pc: u16,
    pub(crate) interrupt_enabled: bool,
    pub(crate) halted: bool,
    pub(crate) halt_bug_on_next_opcode: bool,
    pub(crate) opcode_queue: VecDeque<u8>,
}

impl Cpu {
    pub fn new() -> Cpu {
        let mut cpu = Cpu {
            registers: Registers::default(),
            sp: 0,
            pc: 0,
            interrupt_enabled: false,
            halted: false,
            halt_bug_on_next_opcode: false,
            opcode_queue: VecDeque::new(),
        };
        cpu.emulate_bootrom();
        cpu
    }

    /// Initialize the CPU's flags to post-bootrom values
    fn emulate_bootrom(&mut self) {
        self.pc = 0x100;
        self.registers.a = 0x01;
        self.registers.f = 0xB0.into();
        self.set_word_register(WReg::BC, 0x0013);
        self.set_word_register(WReg::DE, 0x00D8);
        self.set_word_register(WReg::HL, 0x014D);
        self.sp = 0xFFFE;
    }

    /// Called at the beginning of an interrupt helper
    fn check_single_interrupt(
        &mut self,
        memory_bus: &mut MemoryBus,
        bit: u8,
        address: u16,
    ) -> Result<u8> {
        // Check IME flag and relevant bit in IE flag.
        let ie_flag = memory_bus.read_u8(0xffff)?;
        let if_flag = memory_bus.read_u8(0xff0f)?;
        if self.interrupt_enabled && ((ie_flag >> bit) & 1 == 1) && ((if_flag >> bit) & 1 == 1) {
            info!(
                "Handling interrupt: {}",
                match bit {
                    0 => "vblank",
                    1 => "lcdc",
                    2 => "timer",
                    3 => "serial",
                    4 => "joypad",
                    _ => "UNKNOWN",
                }
            );

            // Reset interrupt bit in IF flag
            let if_flag = memory_bus.read_u8(0xff0f)?;
            memory_bus.write_u8(0xff0f, if_flag & !(1 << bit))?;

            // Reset IME flag
            self.interrupt_enabled = false;

            // Execute jump to interrupt vector instruction.
            self.execute_instruction(memory_bus, Instruction::INTERNAL_JUMP_INTERRUPT(address))?;

            // Routine takes 5 M-cycles
            return Ok(5);
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
            return Ok(0);
        }
    }

    fn check_interrupts(&mut self, memory_bus: &mut MemoryBus) -> Result<u8> {
        // If IE and IF
        if memory_bus.read_u8(0xFFFF)? & memory_bus.read_u8(0xFF0F)? != 0 {
            // Unhalt
            if self.halted {
                info! {"Unhalting"};
                self.halted = false;
            }
            // Handle interrupts by priority (starting at bit 0 - V-Blank)
            for bit in 0..=4 {
                if self.interrupt_enabled {
                    let address = 0x40 + bit * 0x8;
                    let elapsed_cycles =
                        self.check_single_interrupt(memory_bus, bit, address.into())?;
                    if elapsed_cycles > 0 {
                        return Ok(elapsed_cycles);
                    }
                } else {
                    // info!("IME not set");
                }
            }
        }

        Ok(0)
    }

    pub fn get_byte_from_pc(&mut self, memory_bus: &mut MemoryBus) -> Result<u8> {
        let byte = match self.opcode_queue.pop_front() {
            Some(opcode) => {
                //trace!("Read queued byte {:#04x}", opcode);
                opcode
            }
            None => {
                let byte = memory_bus.read_u8(self.pc.into())?;
                //trace!("Read byte {:#04x}", byte);
                if self.halt_bug_on_next_opcode {
                    self.opcode_queue.push_back(byte);
                    self.halt_bug_on_next_opcode = false;
                }
                self.pc = self.pc.wrapping_add(1);
                byte
            }
        };

        Ok(byte)
    }

    pub fn get_signed_byte_from_pc(&mut self, memory_bus: &mut MemoryBus) -> Result<i8> {
        Ok(self.get_byte_from_pc(memory_bus)? as i8)
    }

    pub fn get_word_from_pc(&mut self, memory_bus: &mut MemoryBus) -> Result<u16> {
        let bytes = [
            self.get_byte_from_pc(memory_bus)?,
            self.get_byte_from_pc(memory_bus)?,
        ];
        let word = u16::from_le_bytes(bytes);
        Ok(word)
    }

    pub fn set_register(&mut self, reg: Reg, value: u8) {
        match reg {
            Reg::A => self.registers.a = value,
            Reg::B => self.registers.b = value,
            Reg::C => self.registers.c = value,
            Reg::D => self.registers.d = value,
            Reg::E => self.registers.e = value,
            Reg::H => self.registers.h = value,
            Reg::L => self.registers.l = value,
        }
    }

    pub fn set_word_register(&mut self, word_reg: WReg, value: u16) {
        match word_reg {
            WReg::AF => self.registers.set_af(value),
            WReg::BC => self.registers.set_bc(value),
            WReg::DE => self.registers.set_de(value),
            WReg::HL => self.registers.set_hl(value),
            WReg::SP => self.sp = value,
            WReg::PC => self.pc = value,
        }
    }

    pub fn get_register(&self, reg: Reg) -> u8 {
        match reg {
            Reg::A => self.registers.a,
            Reg::B => self.registers.b,
            Reg::C => self.registers.c,
            Reg::D => self.registers.d,
            Reg::E => self.registers.e,
            Reg::H => self.registers.h,
            Reg::L => self.registers.l,
        }
    }

    pub fn get_word_register(&self, word_reg: WReg) -> u16 {
        match word_reg {
            WReg::AF => self.registers.get_af(),
            WReg::BC => self.registers.get_bc(),
            WReg::DE => self.registers.get_de(),
            WReg::HL => self.registers.get_hl(),
            WReg::SP => self.sp,
            WReg::PC => self.pc,
        }
    }

    pub fn push(&mut self, memory_bus: &mut MemoryBus, value: u8) -> Result<()> {
        self.sp -= 1;
        memory_bus.write_u8(self.sp.into(), value)
    }

    pub fn pop(&mut self, memory_bus: &mut MemoryBus) -> Result<u8> {
        let value = memory_bus.read_u8(self.sp.into())?;
        self.sp += 1;
        Ok(value)
    }
}

impl Steppable for Cpu {
    type Context = MemoryBus;

    fn step(&mut self, context: &mut Self::Context, _elapsed: u32) -> Result<ElapsedTime> {
        let mut memory_bus = context;

        let mut elapsed_cycles = if !self.halted {
            // Get and execute opcode
            let opcode = self.get_byte_from_pc(&mut memory_bus)?;
            let elapsed_cycles;
            if opcode == 0xCB {
                let opcode = self.get_byte_from_pc(&mut memory_bus)?;
                elapsed_cycles = self.execute_cb_opcode(&mut memory_bus, opcode)?;
            } else {
                elapsed_cycles = self.execute_regular_opcode(&mut memory_bus, opcode)?;
            }
            elapsed_cycles * 4 // convert from M-cycles to T-cycles
        } else {
            // Return 4 T-cycles (1 M-cycle)
            4
        };

        elapsed_cycles += self.check_interrupts(&mut memory_bus)?;

        Ok(elapsed_cycles.into())
    }
}
