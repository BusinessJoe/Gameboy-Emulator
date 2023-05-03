use crate::component::Addressable;
use crate::error::Result;
use crate::{cpu::Cpu, memory::MemoryBus};
use log::{error, info};
use strum_macros::AsRefStr;

pub enum Reg {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

impl Reg {
    fn get(&self, cpu: &Cpu) -> u8 {
        match self {
            Self::A => cpu.registers.a,
            Self::B => cpu.registers.b,
            Self::C => cpu.registers.c,
            Self::D => cpu.registers.d,
            Self::E => cpu.registers.e,
            Self::H => cpu.registers.h,
            Self::L => cpu.registers.l,
        }
    }

    fn set(&self, cpu: &mut Cpu, value: u8) {
        match self {
            Self::A => cpu.registers.a = value,
            Self::B => cpu.registers.b = value,
            Self::C => cpu.registers.c = value,
            Self::D => cpu.registers.d = value,
            Self::E => cpu.registers.e = value,
            Self::H => cpu.registers.h = value,
            Self::L => cpu.registers.l = value,
        }
    }
}

pub enum WReg {
    AF,
    BC,
    DE,
    HL,
    SP,
    PC,
}

impl WReg {
    fn get(&self, cpu: &Cpu) -> u16 {
        match self {
            Self::AF => cpu.registers.get_af(),
            Self::BC => cpu.registers.get_bc(),
            Self::DE => cpu.registers.get_de(),
            Self::HL => cpu.registers.get_hl(),
            Self::SP => cpu.sp,
            Self::PC => cpu.pc,
        }
    }

    fn set(&self, cpu: &mut Cpu, value: u16) {
        match self {
            Self::AF => cpu.registers.set_af(value),
            Self::BC => cpu.registers.set_bc(value),
            Self::DE => cpu.registers.set_de(value),
            Self::HL => cpu.registers.set_hl(value),
            Self::SP => cpu.sp = value,
            Self::PC => cpu.pc = value,
        }
    }
}

pub enum InstrArgByte {
    ImmediateByte(u8),
    AddressDirect(u16),
    AddressRegister(WReg),
    Register(Reg),
    Offset(Reg),
}

impl InstrArgByte {
    fn get_u8(&self, cpu: &Cpu, memory_bus: &mut MemoryBus) -> Result<u8> {
        match self {
            Self::ImmediateByte(byte) => Ok(*byte),
            Self::AddressDirect(address) => memory_bus.read_u8((*address).into()),
            Self::AddressRegister(wreg) => {
                let addr: u16 = wreg.get(cpu);
                memory_bus.read_u8(addr.into())
            }
            Self::Register(reg) => Ok(reg.get(cpu)),
            Self::Offset(reg) => {
                let addr = 0xff00 + u16::from(reg.get(cpu));
                memory_bus.read_u8(addr.into())
            }
        }
    }

    fn set_u8(&self, cpu: &mut Cpu, memory_bus: &mut MemoryBus, value: u8) -> Result<()> {
        match self {
            Self::ImmediateByte(_) => panic!(),
            Self::AddressDirect(address) => memory_bus.write_u8((*address).into(), value),
            Self::AddressRegister(wreg) => {
                let addr: u16 = wreg.get(cpu);
                memory_bus.write_u8(addr.into(), value)
            }
            Self::Register(reg) => Ok(reg.set(cpu, value)),
            Self::Offset(reg) => {
                let addr = 0xff00 + u16::from(reg.get(cpu));
                memory_bus.write_u8(addr.into(), value)
            }
        }
    }
}

pub enum InstrArgWord {
    ImmediateWord(u16),
    AddressDirect(u16),
    AddressRegister(WReg),
    WordRegister(WReg),
}

impl InstrArgWord {
    fn get_u16(&self, cpu: &Cpu) -> u16 {
        match self {
            Self::ImmediateWord(word) => *word,
            Self::AddressDirect(_) => panic!(),
            Self::AddressRegister(_) => panic!(),
            Self::WordRegister(wreg) => wreg.get(cpu),
        }
    }

    fn set_u16(&self, cpu: &mut Cpu, memory_bus: &mut MemoryBus, value: u16) -> Result<()> {
        match self {
            Self::ImmediateWord(_) => panic!(),
            Self::AddressDirect(address) => {
                let bytes = value.to_le_bytes();
                memory_bus.write_u8((*address).into(), bytes[0])?;
                memory_bus.write_u8((*address + 1).into(), bytes[1])?;
            }
            Self::AddressRegister(wreg) => {
                let addr: usize = wreg.get(cpu).into();

                let bytes = value.to_le_bytes();
                memory_bus.write_u8(addr, bytes[0])?;
                memory_bus.write_u8(addr + 1, bytes[1])?;
            }
            Self::WordRegister(wreg) => wreg.set(cpu, value),
        }
        Ok(())
    }
}

#[allow(non_camel_case_types)]
#[derive(AsRefStr)]
pub enum Instruction {
    // Internal instruction to jump to an interrupt vector. Has no associated opcode.
    INTERNAL_JUMP_INTERRUPT(u16),

    /* LD nn,n */
    LD(InstrArgByte, InstrArgByte),
    LD_16(InstrArgWord, InstrArgWord),

    /* LD SP,HL */
    LDHL_SP(i8),

    /* LDD */
    LDD_A_FROM_HL,
    LDD_A_INTO_HL,

    /* LDI */
    LDI_A_FROM_HL,
    LDI_A_INTO_HL,

    PUSH(InstrArgWord),
    POP(InstrArgWord),

    /* ADD */
    ADD(InstrArgByte),
    ADD_HL(InstrArgWord),
    ADD_SP(i8),

    /* ADC */
    ADC(InstrArgByte),

    /* SUB */
    SUB(InstrArgByte),

    /* SBC */
    SBC(InstrArgByte),

    AND(InstrArgByte),
    OR(InstrArgByte),
    XOR(InstrArgByte),
    CP(InstrArgByte),

    INC(InstrArgByte),
    INC_WORD(InstrArgWord),

    DEC(InstrArgByte),
    DEC_WORD(InstrArgWord),

    SWAP(InstrArgByte),

    DAA,

    CPL,

    CCF,
    SCF,

    NOP,

    HALT,
    STOP,

    DI,
    EI,

    RLC(InstrArgByte),
    RLCA,
    RL(InstrArgByte),
    RLA,
    RRC(InstrArgByte),
    RRCA,
    RR(InstrArgByte),
    RRA,

    SLA(InstrArgByte),
    SRA(InstrArgByte),
    SRL(InstrArgByte),

    BIT(Bit, InstrArgByte),
    SET(Bit, InstrArgByte),
    RES(Bit, InstrArgByte),

    JP(u16),
    JP_CONDITION(Flag, u16),
    JP_HL,

    JR(i8),
    JR_CONDITION(Flag, i8),

    CALL(u16),
    CALL_CONDITION(Flag, u16),

    RST(u16),

    RET,
    RET_CONDITION(Flag),
    RETI,
}

type Bit = u8;

pub enum Flag {
    NZ,
    Z,
    NC,
    C,
}

pub enum BranchStatus {
    Branch,
    NoBranch,
}

fn get_opcode_delay(opcode: u8) -> u8 {
    match opcode {
        0x00 => 1,
        0x01 => 3,
        0x02 => 2,
        0x03 => 2,
        0x04 => 1,
        0x05 => 1,
        0x06 => 2,
        0x07 => 1,
        0x08 => 5,
        0x09 => 2,
        0x0A => 2,
        0x0B => 2,
        0x0C => 1,
        0x0D => 1,
        0x0E => 2,
        0x0F => 1,
        0x10 => 1,
        0x11 => 3,
        0x12 => 2,
        0x13 => 2,
        0x14 => 1,
        0x15 => 1,
        0x16 => 2,
        0x17 => 1,
        0x18 => 3,
        0x19 => 2,
        0x1A => 2,
        0x1B => 2,
        0x1C => 1,
        0x1D => 1,
        0x1E => 2,
        0x1F => 1,
        0x20 => 2,
        0x21 => 3,
        0x22 => 2,
        0x23 => 2,
        0x24 => 1,
        0x25 => 1,
        0x26 => 2,
        0x27 => 1,
        0x28 => 2,
        0x29 => 2,
        0x2A => 2,
        0x2B => 2,
        0x2C => 1,
        0x2D => 1,
        0x2E => 2,
        0x2F => 1,
        0x30 => 2,
        0x31 => 3,
        0x32 => 2,
        0x33 => 2,
        0x34 => 3,
        0x35 => 3,
        0x36 => 3,
        0x37 => 1,
        0x38 => 2,
        0x39 => 2,
        0x3A => 2,
        0x3B => 2,
        0x3C => 1,
        0x3D => 1,
        0x3E => 2,
        0x3F => 1,
        0x40 => 1,
        0x41 => 1,
        0x42 => 1,
        0x43 => 1,
        0x44 => 1,
        0x45 => 1,
        0x46 => 2,
        0x47 => 1,
        0x48 => 1,
        0x49 => 1,
        0x4A => 1,
        0x4B => 1,
        0x4C => 1,
        0x4D => 1,
        0x4E => 2,
        0x4F => 1,
        0x50 => 1,
        0x51 => 1,
        0x52 => 1,
        0x53 => 1,
        0x54 => 1,
        0x55 => 1,
        0x56 => 2,
        0x57 => 1,
        0x58 => 1,
        0x59 => 1,
        0x5A => 1,
        0x5B => 1,
        0x5C => 1,
        0x5D => 1,
        0x5E => 2,
        0x5F => 1,
        0x60 => 1,
        0x61 => 1,
        0x62 => 1,
        0x63 => 1,
        0x64 => 1,
        0x65 => 1,
        0x66 => 2,
        0x67 => 1,
        0x68 => 1,
        0x69 => 1,
        0x6A => 1,
        0x6B => 1,
        0x6C => 1,
        0x6D => 1,
        0x6E => 2,
        0x6F => 1,
        0x70 => 2,
        0x71 => 2,
        0x72 => 2,
        0x73 => 2,
        0x74 => 2,
        0x75 => 2,
        0x76 => 1,
        0x77 => 2,
        0x78 => 1,
        0x79 => 1,
        0x7A => 1,
        0x7B => 1,
        0x7C => 1,
        0x7D => 1,
        0x7E => 2,
        0x7F => 1,
        0x80 => 1,
        0x81 => 1,
        0x82 => 1,
        0x83 => 1,
        0x84 => 1,
        0x85 => 1,
        0x86 => 2,
        0x87 => 1,
        0x88 => 1,
        0x89 => 1,
        0x8A => 1,
        0x8B => 1,
        0x8C => 1,
        0x8D => 1,
        0x8E => 2,
        0x8F => 1,
        0x90 => 1,
        0x91 => 1,
        0x92 => 1,
        0x93 => 1,
        0x94 => 1,
        0x95 => 1,
        0x96 => 2,
        0x97 => 1,
        0x98 => 1,
        0x99 => 1,
        0x9A => 1,
        0x9B => 1,
        0x9C => 1,
        0x9D => 1,
        0x9E => 2,
        0x9F => 1,
        0xA0 => 1,
        0xA1 => 1,
        0xA2 => 1,
        0xA3 => 1,
        0xA4 => 1,
        0xA5 => 1,
        0xA6 => 2,
        0xA7 => 1,
        0xA8 => 1,
        0xA9 => 1,
        0xAA => 1,
        0xAB => 1,
        0xAC => 1,
        0xAD => 1,
        0xAE => 2,
        0xAF => 1,
        0xB0 => 1,
        0xB1 => 1,
        0xB2 => 1,
        0xB3 => 1,
        0xB4 => 1,
        0xB5 => 1,
        0xB6 => 2,
        0xB7 => 1,
        0xB8 => 1,
        0xB9 => 1,
        0xBA => 1,
        0xBB => 1,
        0xBC => 1,
        0xBD => 1,
        0xBE => 2,
        0xBF => 1,
        0xC0 => 2,
        0xC1 => 3,
        0xC2 => 3,
        0xC3 => 4,
        0xC4 => 3,
        0xC5 => 4,
        0xC6 => 2,
        0xC7 => 4,
        0xC8 => 2,
        0xC9 => 4,
        0xCA => 3,
        0xCB => unimplemented!(),
        0xCC => 3,
        0xCD => 6,
        0xCE => 2,
        0xCF => 4,
        0xD0 => 2,
        0xD1 => 3,
        0xD2 => 3,
        0xD3 => unimplemented!(),
        0xD4 => 3,
        0xD5 => 4,
        0xD6 => 2,
        0xD7 => 4,
        0xD8 => 2,
        0xD9 => 4,
        0xDA => 3,
        0xDB => unimplemented!(),
        0xDC => 3,
        0xDD => unimplemented!(),
        0xDE => 2,
        0xDF => 4,
        0xE0 => 3,
        0xE1 => 3,
        0xE2 => 2,
        0xE3 => unimplemented!(),
        0xE4 => unimplemented!(),
        0xE5 => 4,
        0xE6 => 2,
        0xE7 => 4,
        0xE8 => 4,
        0xE9 => 1,
        0xEA => 4,
        0xEB => unimplemented!(),
        0xEC => unimplemented!(),
        0xED => unimplemented!(),
        0xEE => 2,
        0xEF => 4,
        0xF0 => 3,
        0xF1 => 3,
        0xF2 => 2,
        0xF3 => 1,
        0xF4 => unimplemented!(),
        0xF5 => 4,
        0xF6 => 2,
        0xF7 => 4,
        0xF8 => 3,
        0xF9 => 2,
        0xFA => 4,
        0xFB => 1,
        0xFC => unimplemented!(),
        0xFD => unimplemented!(),
        0xFE => 2,
        0xFF => 4,
    }
}

fn get_branched_opcode_delay(opcode: u8) -> u8 {
    match opcode {
        0x20 => 3,
        0x28 => 3,
        0x30 => 3,
        0x38 => 3,
        0xC0 => 5,
        0xC2 => 4,
        0xC4 => 6,
        0xC8 => 5,
        0xCA => 4,
        0xCC => 6,
        0xD0 => 5,
        0xD2 => 4,
        0xD4 => 6,
        0xD8 => 5,
        0xDA => 4,
        0xDC => 6,
        _ => unimplemented!(),
    }
}

fn get_cb_opcode_delay(opcode: u8) -> u8 {
    match opcode {
        0x00 => 2,
        0x01 => 2,
        0x02 => 2,
        0x03 => 2,
        0x04 => 2,
        0x05 => 2,
        0x06 => 4,
        0x07 => 2,
        0x08 => 2,
        0x09 => 2,
        0x0A => 2,
        0x0B => 2,
        0x0C => 2,
        0x0D => 2,
        0x0E => 4,
        0x0F => 2,
        0x10 => 2,
        0x11 => 2,
        0x12 => 2,
        0x13 => 2,
        0x14 => 2,
        0x15 => 2,
        0x16 => 4,
        0x17 => 2,
        0x18 => 2,
        0x19 => 2,
        0x1A => 2,
        0x1B => 2,
        0x1C => 2,
        0x1D => 2,
        0x1E => 4,
        0x1F => 2,
        0x20 => 2,
        0x21 => 2,
        0x22 => 2,
        0x23 => 2,
        0x24 => 2,
        0x25 => 2,
        0x26 => 4,
        0x27 => 2,
        0x28 => 2,
        0x29 => 2,
        0x2A => 2,
        0x2B => 2,
        0x2C => 2,
        0x2D => 2,
        0x2E => 4,
        0x2F => 2,
        0x30 => 2,
        0x31 => 2,
        0x32 => 2,
        0x33 => 2,
        0x34 => 2,
        0x35 => 2,
        0x36 => 4,
        0x37 => 2,
        0x38 => 2,
        0x39 => 2,
        0x3A => 2,
        0x3B => 2,
        0x3C => 2,
        0x3D => 2,
        0x3E => 4,
        0x3F => 2,
        0x40 => 2,
        0x41 => 2,
        0x42 => 2,
        0x43 => 2,
        0x44 => 2,
        0x45 => 2,
        0x46 => 3,
        0x47 => 2,
        0x48 => 2,
        0x49 => 2,
        0x4A => 2,
        0x4B => 2,
        0x4C => 2,
        0x4D => 2,
        0x4E => 3,
        0x4F => 2,
        0x50 => 2,
        0x51 => 2,
        0x52 => 2,
        0x53 => 2,
        0x54 => 2,
        0x55 => 2,
        0x56 => 3,
        0x57 => 2,
        0x58 => 2,
        0x59 => 2,
        0x5A => 2,
        0x5B => 2,
        0x5C => 2,
        0x5D => 2,
        0x5E => 3,
        0x5F => 2,
        0x60 => 2,
        0x61 => 2,
        0x62 => 2,
        0x63 => 2,
        0x64 => 2,
        0x65 => 2,
        0x66 => 3,
        0x67 => 2,
        0x68 => 2,
        0x69 => 2,
        0x6A => 2,
        0x6B => 2,
        0x6C => 2,
        0x6D => 2,
        0x6E => 3,
        0x6F => 2,
        0x70 => 2,
        0x71 => 2,
        0x72 => 2,
        0x73 => 2,
        0x74 => 2,
        0x75 => 2,
        0x76 => 3,
        0x77 => 2,
        0x78 => 2,
        0x79 => 2,
        0x7A => 2,
        0x7B => 2,
        0x7C => 2,
        0x7D => 2,
        0x7E => 3,
        0x7F => 2,
        0x80 => 2,
        0x81 => 2,
        0x82 => 2,
        0x83 => 2,
        0x84 => 2,
        0x85 => 2,
        0x86 => 4,
        0x87 => 2,
        0x88 => 2,
        0x89 => 2,
        0x8A => 2,
        0x8B => 2,
        0x8C => 2,
        0x8D => 2,
        0x8E => 4,
        0x8F => 2,
        0x90 => 2,
        0x91 => 2,
        0x92 => 2,
        0x93 => 2,
        0x94 => 2,
        0x95 => 2,
        0x96 => 4,
        0x97 => 2,
        0x98 => 2,
        0x99 => 2,
        0x9A => 2,
        0x9B => 2,
        0x9C => 2,
        0x9D => 2,
        0x9E => 4,
        0x9F => 2,
        0xA0 => 2,
        0xA1 => 2,
        0xA2 => 2,
        0xA3 => 2,
        0xA4 => 2,
        0xA5 => 2,
        0xA6 => 4,
        0xA7 => 2,
        0xA8 => 2,
        0xA9 => 2,
        0xAA => 2,
        0xAB => 2,
        0xAC => 2,
        0xAD => 2,
        0xAE => 4,
        0xAF => 2,
        0xB0 => 2,
        0xB1 => 2,
        0xB2 => 2,
        0xB3 => 2,
        0xB4 => 2,
        0xB5 => 2,
        0xB6 => 4,
        0xB7 => 2,
        0xB8 => 2,
        0xB9 => 2,
        0xBA => 2,
        0xBB => 2,
        0xBC => 2,
        0xBD => 2,
        0xBE => 4,
        0xBF => 2,
        0xC0 => 2,
        0xC1 => 2,
        0xC2 => 2,
        0xC3 => 2,
        0xC4 => 2,
        0xC5 => 2,
        0xC6 => 4,
        0xC7 => 2,
        0xC8 => 2,
        0xC9 => 2,
        0xCA => 2,
        0xCB => 2,
        0xCC => 2,
        0xCD => 2,
        0xCE => 4,
        0xCF => 2,
        0xD0 => 2,
        0xD1 => 2,
        0xD2 => 2,
        0xD3 => 2,
        0xD4 => 2,
        0xD5 => 2,
        0xD6 => 4,
        0xD7 => 2,
        0xD8 => 2,
        0xD9 => 2,
        0xDA => 2,
        0xDB => 2,
        0xDC => 2,
        0xDD => 2,
        0xDE => 4,
        0xDF => 2,
        0xE0 => 2,
        0xE1 => 2,
        0xE2 => 2,
        0xE3 => 2,
        0xE4 => 2,
        0xE5 => 2,
        0xE6 => 4,
        0xE7 => 2,
        0xE8 => 2,
        0xE9 => 2,
        0xEA => 2,
        0xEB => 2,
        0xEC => 2,
        0xED => 2,
        0xEE => 4,
        0xEF => 2,
        0xF0 => 2,
        0xF1 => 2,
        0xF2 => 2,
        0xF3 => 2,
        0xF4 => 2,
        0xF5 => 2,
        0xF6 => 4,
        0xF7 => 2,
        0xF8 => 2,
        0xF9 => 2,
        0xFA => 2,
        0xFB => 2,
        0xFC => 2,
        0xFD => 2,
        0xFE => 4,
        0xFF => 2,
    }
}

impl Cpu {
    fn test_flag(&self, flag: Flag) -> bool {
        match flag {
            Flag::Z => self.registers.f.zero,
            Flag::NZ => !self.registers.f.zero,
            Flag::C => self.registers.f.carry,
            Flag::NC => !self.registers.f.carry,
        }
    }

    pub fn execute_instruction(
        &mut self,
        memory_bus: &mut MemoryBus,
        instruction: Instruction,
    ) -> Result<BranchStatus> {
        //debug!("Executing instruction {}", instruction.as_ref());
        let mut branch_status = BranchStatus::NoBranch;
        match instruction {
            Instruction::INTERNAL_JUMP_INTERRUPT(address) => {
                // Push PC onto stack. LSB is last/top of the stack.
                let bytes = self.pc.to_le_bytes();
                self.push(memory_bus, bytes[1]).unwrap();
                self.push(memory_bus, bytes[0]).unwrap();

                // Jump to starting address of interrupt
                self.pc = address;
            }
            Instruction::LD(target, source) => {
                let value = source.get_u8(self, memory_bus)?;
                target.set_u8(self, memory_bus, value)?
            }
            Instruction::LD_16(target, source) => {
                let value = source.get_u16(self);
                target.set_u16(self, memory_bus, value)?
            }
            Instruction::LDHL_SP(signed_immediate) => {
                let sum = self
                    .sp
                    .wrapping_add_signed(i8::from(signed_immediate) as i16);
                self.set_word_register(WReg::HL, sum);

                self.registers.f.zero = false;
                self.registers.f.subtract = false;
                self.registers.f.half_carry =
                    (self.sp ^ i8::from(signed_immediate) as u16 ^ sum) & 0x10 == 0x10;
                self.registers.f.carry =
                    (self.sp ^ i8::from(signed_immediate) as u16 ^ sum) & 0x100 == 0x100;
            }
            Instruction::LDD_A_FROM_HL => {
                let value = InstrArgByte::AddressRegister(WReg::HL).get_u8(self, memory_bus)?;
                InstrArgByte::Register(Reg::A).set_u8(self, memory_bus, value)?;
                self.execute_instruction(
                    memory_bus,
                    Instruction::DEC_WORD(InstrArgWord::WordRegister(WReg::HL)),
                )?;
            }
            Instruction::LDD_A_INTO_HL => {
                let value = InstrArgByte::Register(Reg::A).get_u8(self, memory_bus)?;
                InstrArgByte::AddressRegister(WReg::HL).set_u8(self, memory_bus, value)?;
                self.execute_instruction(
                    memory_bus,
                    Instruction::DEC_WORD(InstrArgWord::WordRegister(WReg::HL)),
                )?;
            }
            Instruction::LDI_A_FROM_HL => {
                let value = InstrArgByte::AddressRegister(WReg::HL).get_u8(self, memory_bus)?;
                InstrArgByte::Register(Reg::A).set_u8(self, memory_bus, value)?;
                self.execute_instruction(
                    memory_bus,
                    Instruction::INC_WORD(InstrArgWord::WordRegister(WReg::HL)),
                )?;
            }
            Instruction::LDI_A_INTO_HL => {
                let value = InstrArgByte::Register(Reg::A).get_u8(self, memory_bus)?;
                InstrArgByte::AddressRegister(WReg::HL).set_u8(self, memory_bus, value)?;
                self.execute_instruction(
                    memory_bus,
                    Instruction::INC_WORD(InstrArgWord::WordRegister(WReg::HL)),
                )?;
            }
            Instruction::PUSH(pair) => {
                let reg_value = pair.get_u16(self);
                let bytes = reg_value.to_le_bytes();
                self.push(memory_bus, bytes[1])?;
                self.push(memory_bus, bytes[0])?;
            }
            Instruction::POP(pair) => {
                let bytes = [self.pop(memory_bus)?, self.pop(memory_bus)?];
                let value = u16::from_le_bytes(bytes);
                pair.set_u16(self, memory_bus, value)?;
            }

            /* Arithmetic */
            Instruction::ADD(source) => {
                let value = source.get_u8(self, memory_bus)?;
                let (sum, overflow) = self.registers.a.overflowing_add(value);
                // Set F register flags.
                self.registers.f.zero = sum == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry =
                    ((self.registers.a & 0xf) + (value & 0xf)) & 0x10 == 0x10;
                self.registers.f.carry = overflow;

                self.registers.a = sum;
            }
            Instruction::ADD_HL(word_reg) => {
                let value = word_reg.get_u16(self);
                let hl = self.registers.get_hl();

                let (sum, carry) = value.overflowing_add(hl);

                self.registers.f.subtract = false;
                self.registers.f.half_carry = ((hl & 0xfff) + (value & 0xfff)) & 0x1000 == 0x1000;
                self.registers.f.carry = carry;

                self.registers.set_hl(sum);
            }
            Instruction::ADD_SP(imm) => {
                let sp = self.get_word_register(WReg::SP);
                let imm: i16 = i8::from(imm).into();

                let sum = sp.wrapping_add_signed(imm);
                self.set_word_register(WReg::SP, sum);

                let unsigned_imm = imm as u16;

                self.registers.f.zero = false;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = (sp ^ unsigned_imm ^ (sum & 0xFFFF)) & 0x10 == 0x10;
                self.registers.f.carry = (sp ^ unsigned_imm ^ (sum & 0xFFFF)) & 0x100 == 0x100;
            }
            Instruction::ADC(arith_target) => {
                let value = arith_target.get_u8(self, memory_bus)?;
                let (partial_sum, overflow1) = self.registers.a.overflowing_add(value);
                let (sum, overflow2) = partial_sum.overflowing_add(self.registers.f.carry.into());
                let overflow = overflow1 || overflow2;

                // Set F register flags.
                self.registers.f.zero = sum == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = (self.registers.a ^ value ^ sum) & 0x10 == 0x10;
                self.registers.f.carry = overflow;

                self.registers.a = sum;
            }
            Instruction::SUB(arith_target) => {
                let value = arith_target.get_u8(self, memory_bus)?;
                let a = self.registers.a;
                let diff = a.wrapping_sub(value);

                // Set F register flags.
                self.registers.f.zero = diff == 0;
                self.registers.f.subtract = true;
                self.registers.f.half_carry = (a & 0xf) < (value & 0xf);
                self.registers.f.carry = a < value;

                self.registers.a = diff;
            }
            Instruction::SBC(arith_target) => {
                let value = arith_target.get_u8(self, memory_bus)?;
                let a = self.registers.a;
                let carry: u8 = self.registers.f.carry.into();

                let diff = a.wrapping_sub(value).wrapping_sub(carry);

                // Set F register flags.
                self.registers.f.zero = diff == 0;
                self.registers.f.subtract = true;
                self.registers.f.half_carry = (a & 0xf) < (value & 0xf) + carry;
                self.registers.f.carry = (a as u16) < value as u16 + carry as u16;

                self.registers.a = diff;
            }
            Instruction::AND(arith_target) => {
                let value = arith_target.get_u8(self, memory_bus)?;
                let result = self.registers.a & value;
                self.registers.a = result;

                self.registers.f.zero = result == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = true;
                self.registers.f.carry = false;
            }
            Instruction::OR(arith_target) => {
                let value = arith_target.get_u8(self, memory_bus)?;
                let result = self.registers.a | value;
                self.registers.a = result;

                self.registers.f.zero = result == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = false;
            }
            Instruction::XOR(arith_target) => {
                let value = arith_target.get_u8(self, memory_bus)?;
                let result = self.registers.a ^ value;
                self.registers.a = result;

                self.registers.f.zero = result == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = false;
            }
            Instruction::CP(arith_target) => {
                let value = arith_target.get_u8(self, memory_bus)?;
                let a = self.registers.a;

                self.registers.f.zero = a == value;
                self.registers.f.subtract = true;
                self.registers.f.half_carry = (a & 0xf) < (value & 0xf);
                self.registers.f.carry = a < value;
            }
            Instruction::INC(target) => {
                let value = target.get_u8(self, memory_bus)?;
                let incremented_value = value.wrapping_add(1);
                target.set_u8(self, memory_bus, incremented_value)?;

                self.registers.f.zero = incremented_value == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = ((value & 0xf) + 1) & 0x10 == 0x10;
            }
            Instruction::INC_WORD(word_reg) => {
                let value = word_reg.get_u16(self);
                let incremented_value = value.wrapping_add(1);
                word_reg.set_u16(self, memory_bus, incremented_value)?;
            }
            Instruction::DEC(target) => {
                let value = target.get_u8(self, memory_bus)?;
                let decremented_value = value.wrapping_sub(1);
                target.set_u8(self, memory_bus, decremented_value)?;

                self.registers.f.zero = decremented_value == 0;
                self.registers.f.subtract = true;
                self.registers.f.half_carry = (value & 0xf) < 1;
            }
            Instruction::DEC_WORD(word_reg) => {
                let value = word_reg.get_u16(self);
                let decremented_value = value.wrapping_sub(1);
                word_reg.set_u16(self, memory_bus, decremented_value)?;
            }

            /* Miscellaneous */
            Instruction::SWAP(target) => {
                let value = target.get_u8(self, memory_bus)?;
                let nibbles = [value & 0xf, value >> 4];
                let swapped_value = nibbles[0] << 4 | nibbles[1];
                target.set_u8(self, memory_bus, swapped_value)?;

                self.registers.f.zero = swapped_value == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = false;
            }
            Instruction::DAA => {
                if !self.registers.f.subtract {
                    // After an addition, adjust if (half-)carry occured or result is out of
                    // bounds.
                    if self.registers.f.carry || self.registers.a > 0x99 {
                        self.registers.a = self.registers.a.wrapping_add(0x60);
                        self.registers.f.carry = true;
                    }
                    if self.registers.f.half_carry || (self.registers.a & 0x0f) > 0x09 {
                        self.registers.a = self.registers.a.wrapping_add(0x06);
                    }
                } else {
                    // After a subtraction, only adjust if (half-)carry occured.
                    if self.registers.f.carry {
                        self.registers.a = self.registers.a.wrapping_sub(0x60);
                    }
                    if self.registers.f.half_carry {
                        self.registers.a = self.registers.a.wrapping_sub(0x06);
                    }
                }

                // These flags are always updated
                self.registers.f.zero = self.registers.a == 0;
                self.registers.f.half_carry = false;
            }
            Instruction::CPL => {
                self.registers.a = !self.registers.a;

                self.registers.f.subtract = true;
                self.registers.f.half_carry = true;
            }
            Instruction::CCF => {
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = !self.registers.f.carry;
            }
            Instruction::SCF => {
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = true;
            }
            Instruction::NOP => {}
            Instruction::HALT => {
                info!("Halting");
                let interrupt_pending =
                    memory_bus.read_u8(0xffff)? & memory_bus.read_u8(0xff0f)? & 0x1f != 0;
                self.halted = true;
                if !self.interrupt_enabled && interrupt_pending {
                    let byte = memory_bus.read_u8(self.pc.into())?;
                    info!("Performing halt bug with byte {:#04x}", byte);
                    println!("Performing halt bug with byte {:#04x}", byte);
                    self.halt_bug_on_next_opcode = true;
                }
            }
            Instruction::STOP => error!("STOP is not implemented"),
            Instruction::DI => self.interrupt_enabled = false,
            Instruction::EI => self.interrupt_enabled = true,

            /* Rotates & shifts */
            Instruction::RLC(target) => {
                let value = target.get_u8(self, memory_bus)?;

                let new_carry_flag = (value >> 7) & 1;
                let truncated_bit = (value >> 7) & 1;

                let result = (value << 1) | truncated_bit;

                self.registers.f.zero = result == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = new_carry_flag == 1;

                target.set_u8(self, memory_bus, result)?;
            }
            Instruction::RLCA => {
                self.execute_instruction(
                    memory_bus,
                    Instruction::RLC(InstrArgByte::Register(Reg::A)),
                )?;
                self.registers.f.zero = false;
            }
            Instruction::RL(target) => {
                let value = target.get_u8(self, memory_bus)?;
                let bit7 = value >> 7;
                let carry: u8 = self.registers.f.carry.into();

                let result = (value << 1) | carry;
                target.set_u8(self, memory_bus, result)?;

                self.registers.f.zero = result == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = bit7 == 1;
            }
            Instruction::RLA => {
                self.execute_instruction(
                    memory_bus,
                    Instruction::RL(InstrArgByte::Register(Reg::A)),
                )?;
                self.registers.f.zero = false;
            }
            Instruction::RRC(target) => {
                let value = target.get_u8(self, memory_bus)?;
                let bit0 = value & 0b1;

                let result = (value >> 1) | (bit0 << 7);
                target.set_u8(self, memory_bus, result)?;

                self.registers.f.zero = result == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = bit0 == 1;
            }
            Instruction::RRCA => {
                self.execute_instruction(
                    memory_bus,
                    Instruction::RRC(InstrArgByte::Register(Reg::A)),
                )?;
                self.registers.f.zero = false;
            }
            Instruction::RR(target) => {
                let value = target.get_u8(self, memory_bus)?;
                let bit0 = value & 0b1;
                let carry: u8 = self.registers.f.carry.into();

                let result = (value >> 1) | (carry << 7);
                target.set_u8(self, memory_bus, result)?;

                self.registers.f.zero = result == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = bit0 == 1;
            }
            Instruction::RRA => {
                self.execute_instruction(
                    memory_bus,
                    Instruction::RR(InstrArgByte::Register(Reg::A)),
                )?;
                self.registers.f.zero = false;
            }
            Instruction::SLA(target) => {
                let value = target.get_u8(self, memory_bus)?;
                let bit7 = value >> 7;

                let result = value << 1;
                target.set_u8(self, memory_bus, result)?;

                self.registers.f.zero = result == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = bit7 == 1;
            }
            Instruction::SRA(target) => {
                let value = target.get_u8(self, memory_bus)?;
                let bit0 = value & 0b1;
                let bit7 = value >> 7;

                let result = (value >> 1) | (bit7 << 7);
                target.set_u8(self, memory_bus, result)?;

                self.registers.f.zero = result == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = bit0 == 1;
            }
            Instruction::SRL(target) => {
                let value = target.get_u8(self, memory_bus)?;
                let bit0 = value & 0b1;

                let result = value >> 1;
                target.set_u8(self, memory_bus, result)?;

                self.registers.f.zero = result == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = bit0 == 1;
            }

            /* Bit opcodes */
            Instruction::BIT(bit, target) => {
                let bit: u8 = bit.into();
                let value = target.get_u8(self, memory_bus)?;

                let result = (value >> bit) & 0b1;

                self.registers.f.zero = result == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = true;
            }
            Instruction::SET(bit, target) => {
                let bit: u8 = bit.into();
                let value = target.get_u8(self, memory_bus)?;

                let result = value | (1 << bit);
                target.set_u8(self, memory_bus, result)?;
            }
            Instruction::RES(bit, target) => {
                let bit: u8 = bit.into();
                let value = target.get_u8(self, memory_bus)?;

                let result = value & !(1 << bit);
                target.set_u8(self, memory_bus, result)?;
            }

            /* Jumps */
            Instruction::JP(addr) => {
                self.pc = addr;
            }
            Instruction::JP_CONDITION(flag, addr) => {
                if self.test_flag(flag) {
                    self.execute_instruction(memory_bus, Instruction::JP(addr))?;
                    branch_status = BranchStatus::Branch;
                }
            }
            Instruction::JP_HL => {
                let addr: u16 = self.get_word_register(WReg::HL);
                self.pc = addr;
            }
            Instruction::JR(imm) => {
                let imm: i16 = i8::from(imm).into();
                let addr: u16 = self.pc.checked_add_signed(imm).unwrap();
                self.pc = addr;
            }
            Instruction::JR_CONDITION(flag, imm) => {
                if self.test_flag(flag) {
                    self.execute_instruction(memory_bus, Instruction::JR(imm))?;
                    branch_status = BranchStatus::Branch;
                }
            }

            /* Calls */
            Instruction::CALL(addr) => {
                // Save address of next instruction to stack
                let next_instr_addr = self.pc;
                let bytes = next_instr_addr.to_le_bytes();
                self.push(memory_bus, bytes[1])?;
                self.push(memory_bus, bytes[0])?;

                // Load addr into pc
                self.pc = addr;
            }
            Instruction::CALL_CONDITION(flag, addr) => {
                if self.test_flag(flag) {
                    self.execute_instruction(memory_bus, Instruction::CALL(addr))?;
                    branch_status = BranchStatus::Branch;
                }
            }

            /* Restarts */
            Instruction::RST(imm) => {
                let bytes = self.pc.to_le_bytes();
                self.push(memory_bus, bytes[1])?;
                self.push(memory_bus, bytes[0])?;

                self.pc = imm;
            }

            /* Returns */
            Instruction::RET => {
                let bytes = [self.pop(memory_bus)?, self.pop(memory_bus)?];
                self.pc = u16::from_le_bytes(bytes);
            }
            Instruction::RET_CONDITION(flag) => {
                if self.test_flag(flag) {
                    self.execute_instruction(memory_bus, Instruction::RET)?;
                    branch_status = BranchStatus::Branch;
                }
            }
            Instruction::RETI => {
                self.execute_instruction(memory_bus, Instruction::EI)?;
                self.execute_instruction(memory_bus, Instruction::RET)?;
            }
        }

        Ok(branch_status)
    }

    pub fn execute_regular_opcode(&mut self, memory_bus: &mut MemoryBus, opcode: u8) -> Result<u8> {
        let instruction = match opcode {
            0x00 => Instruction::NOP,
            0x10 => Instruction::STOP,
            0x20 => Instruction::JR_CONDITION(Flag::NZ, self.get_signed_byte_from_pc(memory_bus)?),
            0x30 => Instruction::JR_CONDITION(Flag::NC, self.get_signed_byte_from_pc(memory_bus)?),

            0x01 => Instruction::LD_16(
                InstrArgWord::WordRegister(WReg::BC),
                InstrArgWord::ImmediateWord(self.get_word_from_pc(memory_bus)?),
            ),
            0x11 => Instruction::LD_16(
                InstrArgWord::WordRegister(WReg::DE),
                InstrArgWord::ImmediateWord(self.get_word_from_pc(memory_bus)?),
            ),
            0x21 => Instruction::LD_16(
                InstrArgWord::WordRegister(WReg::HL),
                InstrArgWord::ImmediateWord(self.get_word_from_pc(memory_bus)?),
            ),
            0x31 => Instruction::LD_16(
                InstrArgWord::WordRegister(WReg::SP),
                InstrArgWord::ImmediateWord(self.get_word_from_pc(memory_bus)?),
            ),

            0x02 => Instruction::LD(
                InstrArgByte::AddressRegister(WReg::BC),
                InstrArgByte::Register(Reg::A),
            ),
            0x12 => Instruction::LD(
                InstrArgByte::AddressRegister(WReg::DE),
                InstrArgByte::Register(Reg::A),
            ),
            0x22 => Instruction::LDI_A_INTO_HL,
            0x32 => Instruction::LDD_A_INTO_HL,

            0x03 => Instruction::INC_WORD(InstrArgWord::WordRegister(WReg::BC)),
            0x13 => Instruction::INC_WORD(InstrArgWord::WordRegister(WReg::DE)),
            0x23 => Instruction::INC_WORD(InstrArgWord::WordRegister(WReg::HL)),
            0x33 => Instruction::INC_WORD(InstrArgWord::WordRegister(WReg::SP)),

            0x04 => Instruction::INC(InstrArgByte::Register(Reg::B)),
            0x14 => Instruction::INC(InstrArgByte::Register(Reg::D)),
            0x24 => Instruction::INC(InstrArgByte::Register(Reg::H)),
            0x34 => Instruction::INC(InstrArgByte::AddressRegister(WReg::HL)),

            0x05 => Instruction::DEC(InstrArgByte::Register(Reg::B)),
            0x15 => Instruction::DEC(InstrArgByte::Register(Reg::D)),
            0x25 => Instruction::DEC(InstrArgByte::Register(Reg::H)),
            0x35 => Instruction::DEC(InstrArgByte::AddressRegister(WReg::HL)),

            0x06 => Instruction::LD(
                InstrArgByte::Register(Reg::B),
                InstrArgByte::ImmediateByte(self.get_byte_from_pc(memory_bus)?),
            ),
            0x16 => Instruction::LD(
                InstrArgByte::Register(Reg::D),
                InstrArgByte::ImmediateByte(self.get_byte_from_pc(memory_bus)?),
            ),
            0x26 => Instruction::LD(
                InstrArgByte::Register(Reg::H),
                InstrArgByte::ImmediateByte(self.get_byte_from_pc(memory_bus)?),
            ),
            0x36 => Instruction::LD(
                InstrArgByte::AddressRegister(WReg::HL),
                InstrArgByte::ImmediateByte(self.get_byte_from_pc(memory_bus)?),
            ),

            0x07 => Instruction::RLCA,
            0x17 => Instruction::RLA,
            0x27 => Instruction::DAA,
            0x37 => Instruction::SCF,

            0x08 => Instruction::LD_16(
                InstrArgWord::AddressDirect(self.get_word_from_pc(memory_bus)?),
                InstrArgWord::WordRegister(WReg::SP),
            ),
            0x18 => Instruction::JR(self.get_signed_byte_from_pc(memory_bus)?),
            0x28 => Instruction::JR_CONDITION(Flag::Z, self.get_signed_byte_from_pc(memory_bus)?),
            0x38 => Instruction::JR_CONDITION(Flag::C, self.get_signed_byte_from_pc(memory_bus)?),

            0x09 => Instruction::ADD_HL(InstrArgWord::WordRegister(WReg::BC)),
            0x19 => Instruction::ADD_HL(InstrArgWord::WordRegister(WReg::DE)),
            0x29 => Instruction::ADD_HL(InstrArgWord::WordRegister(WReg::HL)),
            0x39 => Instruction::ADD_HL(InstrArgWord::WordRegister(WReg::SP)),

            0x0A => Instruction::LD(
                InstrArgByte::Register(Reg::A),
                InstrArgByte::AddressRegister(WReg::BC),
            ),
            0x1A => Instruction::LD(
                InstrArgByte::Register(Reg::A),
                InstrArgByte::AddressRegister(WReg::DE),
            ),
            0x2A => Instruction::LDI_A_FROM_HL,
            0x3A => Instruction::LDD_A_FROM_HL,

            0x0B => Instruction::DEC_WORD(InstrArgWord::WordRegister(WReg::BC)),
            0x1B => Instruction::DEC_WORD(InstrArgWord::WordRegister(WReg::DE)),
            0x2B => Instruction::DEC_WORD(InstrArgWord::WordRegister(WReg::HL)),
            0x3B => Instruction::DEC_WORD(InstrArgWord::WordRegister(WReg::SP)),

            0x0C => Instruction::INC(InstrArgByte::Register(Reg::C)),
            0x1C => Instruction::INC(InstrArgByte::Register(Reg::E)),
            0x2C => Instruction::INC(InstrArgByte::Register(Reg::L)),
            0x3C => Instruction::INC(InstrArgByte::Register(Reg::A)),

            0x0D => Instruction::DEC(InstrArgByte::Register(Reg::C)),
            0x1D => Instruction::DEC(InstrArgByte::Register(Reg::E)),
            0x2D => Instruction::DEC(InstrArgByte::Register(Reg::L)),
            0x3D => Instruction::DEC(InstrArgByte::Register(Reg::A)),

            0x0E => Instruction::LD(
                InstrArgByte::Register(Reg::C),
                InstrArgByte::ImmediateByte(self.get_byte_from_pc(memory_bus)?),
            ),
            0x1E => Instruction::LD(
                InstrArgByte::Register(Reg::E),
                InstrArgByte::ImmediateByte(self.get_byte_from_pc(memory_bus)?),
            ),
            0x2E => Instruction::LD(
                InstrArgByte::Register(Reg::L),
                InstrArgByte::ImmediateByte(self.get_byte_from_pc(memory_bus)?),
            ),
            0x3E => Instruction::LD(
                InstrArgByte::Register(Reg::A),
                InstrArgByte::ImmediateByte(self.get_byte_from_pc(memory_bus)?),
            ),

            0x0F => Instruction::RRCA,
            0x1F => Instruction::RRA,
            0x2F => Instruction::CPL,
            0x3F => Instruction::CCF,

            0x40 => Instruction::LD(
                InstrArgByte::Register(Reg::B),
                InstrArgByte::Register(Reg::B),
            ),
            0x41 => Instruction::LD(
                InstrArgByte::Register(Reg::B),
                InstrArgByte::Register(Reg::C),
            ),
            0x42 => Instruction::LD(
                InstrArgByte::Register(Reg::B),
                InstrArgByte::Register(Reg::D),
            ),
            0x43 => Instruction::LD(
                InstrArgByte::Register(Reg::B),
                InstrArgByte::Register(Reg::E),
            ),
            0x44 => Instruction::LD(
                InstrArgByte::Register(Reg::B),
                InstrArgByte::Register(Reg::H),
            ),
            0x45 => Instruction::LD(
                InstrArgByte::Register(Reg::B),
                InstrArgByte::Register(Reg::L),
            ),
            0x46 => Instruction::LD(
                InstrArgByte::Register(Reg::B),
                InstrArgByte::AddressRegister(WReg::HL),
            ),
            0x47 => Instruction::LD(
                InstrArgByte::Register(Reg::B),
                InstrArgByte::Register(Reg::A),
            ),

            0x48 => Instruction::LD(
                InstrArgByte::Register(Reg::C),
                InstrArgByte::Register(Reg::B),
            ),
            0x49 => Instruction::LD(
                InstrArgByte::Register(Reg::C),
                InstrArgByte::Register(Reg::C),
            ),
            0x4A => Instruction::LD(
                InstrArgByte::Register(Reg::C),
                InstrArgByte::Register(Reg::D),
            ),
            0x4B => Instruction::LD(
                InstrArgByte::Register(Reg::C),
                InstrArgByte::Register(Reg::E),
            ),
            0x4C => Instruction::LD(
                InstrArgByte::Register(Reg::C),
                InstrArgByte::Register(Reg::H),
            ),
            0x4D => Instruction::LD(
                InstrArgByte::Register(Reg::C),
                InstrArgByte::Register(Reg::L),
            ),
            0x4E => Instruction::LD(
                InstrArgByte::Register(Reg::C),
                InstrArgByte::AddressRegister(WReg::HL),
            ),
            0x4F => Instruction::LD(
                InstrArgByte::Register(Reg::C),
                InstrArgByte::Register(Reg::A),
            ),

            0x50 => Instruction::LD(
                InstrArgByte::Register(Reg::D),
                InstrArgByte::Register(Reg::B),
            ),
            0x51 => Instruction::LD(
                InstrArgByte::Register(Reg::D),
                InstrArgByte::Register(Reg::C),
            ),
            0x52 => Instruction::LD(
                InstrArgByte::Register(Reg::D),
                InstrArgByte::Register(Reg::D),
            ),
            0x53 => Instruction::LD(
                InstrArgByte::Register(Reg::D),
                InstrArgByte::Register(Reg::E),
            ),
            0x54 => Instruction::LD(
                InstrArgByte::Register(Reg::D),
                InstrArgByte::Register(Reg::H),
            ),
            0x55 => Instruction::LD(
                InstrArgByte::Register(Reg::D),
                InstrArgByte::Register(Reg::L),
            ),
            0x56 => Instruction::LD(
                InstrArgByte::Register(Reg::D),
                InstrArgByte::AddressRegister(WReg::HL),
            ),
            0x57 => Instruction::LD(
                InstrArgByte::Register(Reg::D),
                InstrArgByte::Register(Reg::A),
            ),

            0x58 => Instruction::LD(
                InstrArgByte::Register(Reg::E),
                InstrArgByte::Register(Reg::B),
            ),
            0x59 => Instruction::LD(
                InstrArgByte::Register(Reg::E),
                InstrArgByte::Register(Reg::C),
            ),
            0x5A => Instruction::LD(
                InstrArgByte::Register(Reg::E),
                InstrArgByte::Register(Reg::D),
            ),
            0x5B => Instruction::LD(
                InstrArgByte::Register(Reg::E),
                InstrArgByte::Register(Reg::E),
            ),
            0x5C => Instruction::LD(
                InstrArgByte::Register(Reg::E),
                InstrArgByte::Register(Reg::H),
            ),
            0x5D => Instruction::LD(
                InstrArgByte::Register(Reg::E),
                InstrArgByte::Register(Reg::L),
            ),
            0x5E => Instruction::LD(
                InstrArgByte::Register(Reg::E),
                InstrArgByte::AddressRegister(WReg::HL),
            ),
            0x5F => Instruction::LD(
                InstrArgByte::Register(Reg::E),
                InstrArgByte::Register(Reg::A),
            ),

            0x60 => Instruction::LD(
                InstrArgByte::Register(Reg::H),
                InstrArgByte::Register(Reg::B),
            ),
            0x61 => Instruction::LD(
                InstrArgByte::Register(Reg::H),
                InstrArgByte::Register(Reg::C),
            ),
            0x62 => Instruction::LD(
                InstrArgByte::Register(Reg::H),
                InstrArgByte::Register(Reg::D),
            ),
            0x63 => Instruction::LD(
                InstrArgByte::Register(Reg::H),
                InstrArgByte::Register(Reg::E),
            ),
            0x64 => Instruction::LD(
                InstrArgByte::Register(Reg::H),
                InstrArgByte::Register(Reg::H),
            ),
            0x65 => Instruction::LD(
                InstrArgByte::Register(Reg::H),
                InstrArgByte::Register(Reg::L),
            ),
            0x66 => Instruction::LD(
                InstrArgByte::Register(Reg::H),
                InstrArgByte::AddressRegister(WReg::HL),
            ),
            0x67 => Instruction::LD(
                InstrArgByte::Register(Reg::H),
                InstrArgByte::Register(Reg::A),
            ),

            0x68 => Instruction::LD(
                InstrArgByte::Register(Reg::L),
                InstrArgByte::Register(Reg::B),
            ),
            0x69 => Instruction::LD(
                InstrArgByte::Register(Reg::L),
                InstrArgByte::Register(Reg::C),
            ),
            0x6A => Instruction::LD(
                InstrArgByte::Register(Reg::L),
                InstrArgByte::Register(Reg::D),
            ),
            0x6B => Instruction::LD(
                InstrArgByte::Register(Reg::L),
                InstrArgByte::Register(Reg::E),
            ),
            0x6C => Instruction::LD(
                InstrArgByte::Register(Reg::L),
                InstrArgByte::Register(Reg::H),
            ),
            0x6D => Instruction::LD(
                InstrArgByte::Register(Reg::L),
                InstrArgByte::Register(Reg::L),
            ),
            0x6E => Instruction::LD(
                InstrArgByte::Register(Reg::L),
                InstrArgByte::AddressRegister(WReg::HL),
            ),
            0x6F => Instruction::LD(
                InstrArgByte::Register(Reg::L),
                InstrArgByte::Register(Reg::A),
            ),

            0x70 => Instruction::LD(
                InstrArgByte::AddressRegister(WReg::HL),
                InstrArgByte::Register(Reg::B),
            ),
            0x71 => Instruction::LD(
                InstrArgByte::AddressRegister(WReg::HL),
                InstrArgByte::Register(Reg::C),
            ),
            0x72 => Instruction::LD(
                InstrArgByte::AddressRegister(WReg::HL),
                InstrArgByte::Register(Reg::D),
            ),
            0x73 => Instruction::LD(
                InstrArgByte::AddressRegister(WReg::HL),
                InstrArgByte::Register(Reg::E),
            ),
            0x74 => Instruction::LD(
                InstrArgByte::AddressRegister(WReg::HL),
                InstrArgByte::Register(Reg::H),
            ),
            0x75 => Instruction::LD(
                InstrArgByte::AddressRegister(WReg::HL),
                InstrArgByte::Register(Reg::L),
            ),
            0x76 => Instruction::HALT,
            0x77 => Instruction::LD(
                InstrArgByte::AddressRegister(WReg::HL),
                InstrArgByte::Register(Reg::A),
            ),

            0x78 => Instruction::LD(
                InstrArgByte::Register(Reg::A),
                InstrArgByte::Register(Reg::B),
            ),
            0x79 => Instruction::LD(
                InstrArgByte::Register(Reg::A),
                InstrArgByte::Register(Reg::C),
            ),
            0x7A => Instruction::LD(
                InstrArgByte::Register(Reg::A),
                InstrArgByte::Register(Reg::D),
            ),
            0x7B => Instruction::LD(
                InstrArgByte::Register(Reg::A),
                InstrArgByte::Register(Reg::E),
            ),
            0x7C => Instruction::LD(
                InstrArgByte::Register(Reg::A),
                InstrArgByte::Register(Reg::H),
            ),
            0x7D => Instruction::LD(
                InstrArgByte::Register(Reg::A),
                InstrArgByte::Register(Reg::L),
            ),
            0x7E => Instruction::LD(
                InstrArgByte::Register(Reg::A),
                InstrArgByte::AddressRegister(WReg::HL),
            ),
            0x7F => Instruction::LD(
                InstrArgByte::Register(Reg::A),
                InstrArgByte::Register(Reg::A),
            ),

            0x80 => Instruction::ADD(InstrArgByte::Register(Reg::B)),
            0x81 => Instruction::ADD(InstrArgByte::Register(Reg::C)),
            0x82 => Instruction::ADD(InstrArgByte::Register(Reg::D)),
            0x83 => Instruction::ADD(InstrArgByte::Register(Reg::E)),
            0x84 => Instruction::ADD(InstrArgByte::Register(Reg::H)),
            0x85 => Instruction::ADD(InstrArgByte::Register(Reg::L)),
            0x86 => Instruction::ADD(InstrArgByte::AddressRegister(WReg::HL)),
            0x87 => Instruction::ADD(InstrArgByte::Register(Reg::A)),

            0x88 => Instruction::ADC(InstrArgByte::Register(Reg::B)),
            0x89 => Instruction::ADC(InstrArgByte::Register(Reg::C)),
            0x8A => Instruction::ADC(InstrArgByte::Register(Reg::D)),
            0x8B => Instruction::ADC(InstrArgByte::Register(Reg::E)),
            0x8C => Instruction::ADC(InstrArgByte::Register(Reg::H)),
            0x8D => Instruction::ADC(InstrArgByte::Register(Reg::L)),
            0x8E => Instruction::ADC(InstrArgByte::AddressRegister(WReg::HL)),
            0x8F => Instruction::ADC(InstrArgByte::Register(Reg::A)),

            0x90 => Instruction::SUB(InstrArgByte::Register(Reg::B)),
            0x91 => Instruction::SUB(InstrArgByte::Register(Reg::C)),
            0x92 => Instruction::SUB(InstrArgByte::Register(Reg::D)),
            0x93 => Instruction::SUB(InstrArgByte::Register(Reg::E)),
            0x94 => Instruction::SUB(InstrArgByte::Register(Reg::H)),
            0x95 => Instruction::SUB(InstrArgByte::Register(Reg::L)),
            0x96 => Instruction::SUB(InstrArgByte::AddressRegister(WReg::HL)),
            0x97 => Instruction::SUB(InstrArgByte::Register(Reg::A)),

            0x98 => Instruction::SBC(InstrArgByte::Register(Reg::B)),
            0x99 => Instruction::SBC(InstrArgByte::Register(Reg::C)),
            0x9A => Instruction::SBC(InstrArgByte::Register(Reg::D)),
            0x9B => Instruction::SBC(InstrArgByte::Register(Reg::E)),
            0x9C => Instruction::SBC(InstrArgByte::Register(Reg::H)),
            0x9D => Instruction::SBC(InstrArgByte::Register(Reg::L)),
            0x9E => Instruction::SBC(InstrArgByte::AddressRegister(WReg::HL)),
            0x9F => Instruction::SBC(InstrArgByte::Register(Reg::A)),

            0xA0 => Instruction::AND(InstrArgByte::Register(Reg::B)),
            0xA1 => Instruction::AND(InstrArgByte::Register(Reg::C)),
            0xA2 => Instruction::AND(InstrArgByte::Register(Reg::D)),
            0xA3 => Instruction::AND(InstrArgByte::Register(Reg::E)),
            0xA4 => Instruction::AND(InstrArgByte::Register(Reg::H)),
            0xA5 => Instruction::AND(InstrArgByte::Register(Reg::L)),
            0xA6 => Instruction::AND(InstrArgByte::AddressRegister(WReg::HL)),
            0xA7 => Instruction::AND(InstrArgByte::Register(Reg::A)),

            0xA8 => Instruction::XOR(InstrArgByte::Register(Reg::B)),
            0xA9 => Instruction::XOR(InstrArgByte::Register(Reg::C)),
            0xAA => Instruction::XOR(InstrArgByte::Register(Reg::D)),
            0xAB => Instruction::XOR(InstrArgByte::Register(Reg::E)),
            0xAC => Instruction::XOR(InstrArgByte::Register(Reg::H)),
            0xAD => Instruction::XOR(InstrArgByte::Register(Reg::L)),
            0xAE => Instruction::XOR(InstrArgByte::AddressRegister(WReg::HL)),
            0xAF => Instruction::XOR(InstrArgByte::Register(Reg::A)),

            0xB0 => Instruction::OR(InstrArgByte::Register(Reg::B)),
            0xB1 => Instruction::OR(InstrArgByte::Register(Reg::C)),
            0xB2 => Instruction::OR(InstrArgByte::Register(Reg::D)),
            0xB3 => Instruction::OR(InstrArgByte::Register(Reg::E)),
            0xB4 => Instruction::OR(InstrArgByte::Register(Reg::H)),
            0xB5 => Instruction::OR(InstrArgByte::Register(Reg::L)),
            0xB6 => Instruction::OR(InstrArgByte::AddressRegister(WReg::HL)),
            0xB7 => Instruction::OR(InstrArgByte::Register(Reg::A)),

            0xB8 => Instruction::CP(InstrArgByte::Register(Reg::B)),
            0xB9 => Instruction::CP(InstrArgByte::Register(Reg::C)),
            0xBA => Instruction::CP(InstrArgByte::Register(Reg::D)),
            0xBB => Instruction::CP(InstrArgByte::Register(Reg::E)),
            0xBC => Instruction::CP(InstrArgByte::Register(Reg::H)),
            0xBD => Instruction::CP(InstrArgByte::Register(Reg::L)),
            0xBE => Instruction::CP(InstrArgByte::AddressRegister(WReg::HL)),
            0xBF => Instruction::CP(InstrArgByte::Register(Reg::A)),

            0xC0 => Instruction::RET_CONDITION(Flag::NZ),
            0xD0 => Instruction::RET_CONDITION(Flag::NC),
            0xE0 => Instruction::LD(
                InstrArgByte::AddressDirect(self.get_byte_from_pc(memory_bus)? as u16 + 0xFF00),
                InstrArgByte::Register(Reg::A),
            ),
            0xF0 => Instruction::LD(
                InstrArgByte::Register(Reg::A),
                InstrArgByte::AddressDirect(self.get_byte_from_pc(memory_bus)? as u16 + 0xFF00),
            ),

            0xC1 => Instruction::POP(InstrArgWord::WordRegister(WReg::BC)),
            0xD1 => Instruction::POP(InstrArgWord::WordRegister(WReg::DE)),
            0xE1 => Instruction::POP(InstrArgWord::WordRegister(WReg::HL)),
            0xF1 => Instruction::POP(InstrArgWord::WordRegister(WReg::AF)),

            0xC2 => Instruction::JP_CONDITION(Flag::NZ, self.get_word_from_pc(memory_bus)?),
            0xD2 => Instruction::JP_CONDITION(Flag::NC, self.get_word_from_pc(memory_bus)?),
            0xE2 => Instruction::LD(InstrArgByte::Offset(Reg::C), InstrArgByte::Register(Reg::A)),
            0xF2 => Instruction::LD(InstrArgByte::Register(Reg::A), InstrArgByte::Offset(Reg::C)),

            0xC3 => Instruction::JP(self.get_word_from_pc(memory_bus)?),
            0xD3 => unimplemented!(),
            0xE3 => unimplemented!(),
            0xF3 => Instruction::DI,

            0xC4 => Instruction::CALL_CONDITION(Flag::NZ, self.get_word_from_pc(memory_bus)?),
            0xD4 => Instruction::CALL_CONDITION(Flag::NC, self.get_word_from_pc(memory_bus)?),
            0xE4 => unimplemented!(),
            0xF4 => unimplemented!(),

            0xC5 => Instruction::PUSH(InstrArgWord::WordRegister(WReg::BC)),
            0xD5 => Instruction::PUSH(InstrArgWord::WordRegister(WReg::DE)),
            0xE5 => Instruction::PUSH(InstrArgWord::WordRegister(WReg::HL)),
            0xF5 => Instruction::PUSH(InstrArgWord::WordRegister(WReg::AF)),

            0xC6 => Instruction::ADD(InstrArgByte::ImmediateByte(
                self.get_byte_from_pc(memory_bus)?,
            )),
            0xD6 => Instruction::SUB(InstrArgByte::ImmediateByte(
                self.get_byte_from_pc(memory_bus)?,
            )),
            0xE6 => Instruction::AND(InstrArgByte::ImmediateByte(
                self.get_byte_from_pc(memory_bus)?,
            )),
            0xF6 => Instruction::OR(InstrArgByte::ImmediateByte(
                self.get_byte_from_pc(memory_bus)?,
            )),

            0xC7 => Instruction::RST(0x00),
            0xD7 => Instruction::RST(0x10),
            0xE7 => Instruction::RST(0x20),
            0xF7 => Instruction::RST(0x30),

            0xC8 => Instruction::RET_CONDITION(Flag::Z),
            0xD8 => Instruction::RET_CONDITION(Flag::C),
            0xE8 => Instruction::ADD_SP(self.get_signed_byte_from_pc(memory_bus)?),
            0xF8 => Instruction::LDHL_SP(self.get_signed_byte_from_pc(memory_bus)?),

            0xC9 => Instruction::RET,
            0xD9 => Instruction::RETI,
            0xE9 => Instruction::JP_HL,
            0xF9 => Instruction::LD_16(
                InstrArgWord::WordRegister(WReg::SP),
                InstrArgWord::WordRegister(WReg::HL),
            ),

            0xCA => Instruction::JP_CONDITION(Flag::Z, self.get_word_from_pc(memory_bus)?),
            0xDA => Instruction::JP_CONDITION(Flag::C, self.get_word_from_pc(memory_bus)?),
            0xEA => Instruction::LD(
                InstrArgByte::AddressDirect(self.get_word_from_pc(memory_bus)?),
                InstrArgByte::Register(Reg::A),
            ),
            0xFA => Instruction::LD(
                InstrArgByte::Register(Reg::A),
                InstrArgByte::AddressDirect(self.get_word_from_pc(memory_bus)?),
            ),

            0xCB => unimplemented!(),
            0xDB => unimplemented!(),
            0xEB => unimplemented!(),
            0xFB => Instruction::EI,

            0xCC => Instruction::CALL_CONDITION(Flag::Z, self.get_word_from_pc(memory_bus)?),
            0xDC => Instruction::CALL_CONDITION(Flag::C, self.get_word_from_pc(memory_bus)?),
            0xEC => unimplemented!(),
            0xFC => unimplemented!(),

            0xCD => Instruction::CALL(self.get_word_from_pc(memory_bus)?),
            0xDD => unimplemented!(),
            0xED => unimplemented!(),
            0xFD => unimplemented!(),

            0xCE => Instruction::ADC(
                InstrArgByte::ImmediateByte(self.get_byte_from_pc(memory_bus)?).into(),
            ),
            0xDE => Instruction::SBC(InstrArgByte::ImmediateByte(
                self.get_byte_from_pc(memory_bus)?,
            )),
            0xEE => Instruction::XOR(InstrArgByte::ImmediateByte(
                self.get_byte_from_pc(memory_bus)?,
            )),
            0xFE => Instruction::CP(InstrArgByte::ImmediateByte(
                self.get_byte_from_pc(memory_bus)?,
            )),

            0xCF => Instruction::RST(0x08),
            0xDF => Instruction::RST(0x18),
            0xEF => Instruction::RST(0x28),
            0xFF => Instruction::RST(0x38),
        };

        let branch_status = self.execute_instruction(memory_bus, instruction)?;
        match branch_status {
            BranchStatus::NoBranch => Ok(get_opcode_delay(opcode)),
            BranchStatus::Branch => Ok(get_branched_opcode_delay(opcode)),
        }
    }

    pub fn execute_cb_opcode(&mut self, memory_bus: &mut MemoryBus, opcode: u8) -> Result<u8> {
        let instruction = match opcode {
            0x00 => Instruction::RLC(InstrArgByte::Register(Reg::B)),
            0x01 => Instruction::RLC(InstrArgByte::Register(Reg::C)),
            0x02 => Instruction::RLC(InstrArgByte::Register(Reg::D)),
            0x03 => Instruction::RLC(InstrArgByte::Register(Reg::E)),
            0x04 => Instruction::RLC(InstrArgByte::Register(Reg::H)),
            0x05 => Instruction::RLC(InstrArgByte::Register(Reg::L)),
            0x06 => Instruction::RLC(InstrArgByte::AddressRegister(WReg::HL)),
            0x07 => Instruction::RLC(InstrArgByte::Register(Reg::A)),

            0x08 => Instruction::RRC(InstrArgByte::Register(Reg::B)),
            0x09 => Instruction::RRC(InstrArgByte::Register(Reg::C)),
            0x0A => Instruction::RRC(InstrArgByte::Register(Reg::D)),
            0x0B => Instruction::RRC(InstrArgByte::Register(Reg::E)),
            0x0C => Instruction::RRC(InstrArgByte::Register(Reg::H)),
            0x0D => Instruction::RRC(InstrArgByte::Register(Reg::L)),
            0x0E => Instruction::RRC(InstrArgByte::AddressRegister(WReg::HL)),
            0x0F => Instruction::RRC(InstrArgByte::Register(Reg::A)),

            0x10 => Instruction::RL(InstrArgByte::Register(Reg::B)),
            0x11 => Instruction::RL(InstrArgByte::Register(Reg::C)),
            0x12 => Instruction::RL(InstrArgByte::Register(Reg::D)),
            0x13 => Instruction::RL(InstrArgByte::Register(Reg::E)),
            0x14 => Instruction::RL(InstrArgByte::Register(Reg::H)),
            0x15 => Instruction::RL(InstrArgByte::Register(Reg::L)),
            0x16 => Instruction::RL(InstrArgByte::AddressRegister(WReg::HL)),
            0x17 => Instruction::RL(InstrArgByte::Register(Reg::A)),

            0x18 => Instruction::RR(InstrArgByte::Register(Reg::B)),
            0x19 => Instruction::RR(InstrArgByte::Register(Reg::C)),
            0x1A => Instruction::RR(InstrArgByte::Register(Reg::D)),
            0x1B => Instruction::RR(InstrArgByte::Register(Reg::E)),
            0x1C => Instruction::RR(InstrArgByte::Register(Reg::H)),
            0x1D => Instruction::RR(InstrArgByte::Register(Reg::L)),
            0x1E => Instruction::RR(InstrArgByte::AddressRegister(WReg::HL)),
            0x1F => Instruction::RR(InstrArgByte::Register(Reg::A)),

            0x20 => Instruction::SLA(InstrArgByte::Register(Reg::B)),
            0x21 => Instruction::SLA(InstrArgByte::Register(Reg::C)),
            0x22 => Instruction::SLA(InstrArgByte::Register(Reg::D)),
            0x23 => Instruction::SLA(InstrArgByte::Register(Reg::E)),
            0x24 => Instruction::SLA(InstrArgByte::Register(Reg::H)),
            0x25 => Instruction::SLA(InstrArgByte::Register(Reg::L)),
            0x26 => Instruction::SLA(InstrArgByte::AddressRegister(WReg::HL)),
            0x27 => Instruction::SLA(InstrArgByte::Register(Reg::A)),

            0x28 => Instruction::SRA(InstrArgByte::Register(Reg::B)),
            0x29 => Instruction::SRA(InstrArgByte::Register(Reg::C)),
            0x2A => Instruction::SRA(InstrArgByte::Register(Reg::D)),
            0x2B => Instruction::SRA(InstrArgByte::Register(Reg::E)),
            0x2C => Instruction::SRA(InstrArgByte::Register(Reg::H)),
            0x2D => Instruction::SRA(InstrArgByte::Register(Reg::L)),
            0x2E => Instruction::SRA(InstrArgByte::AddressRegister(WReg::HL)),
            0x2F => Instruction::SRA(InstrArgByte::Register(Reg::A)),

            0x30 => Instruction::SWAP(InstrArgByte::Register(Reg::B)),
            0x31 => Instruction::SWAP(InstrArgByte::Register(Reg::C)),
            0x32 => Instruction::SWAP(InstrArgByte::Register(Reg::D)),
            0x33 => Instruction::SWAP(InstrArgByte::Register(Reg::E)),
            0x34 => Instruction::SWAP(InstrArgByte::Register(Reg::H)),
            0x35 => Instruction::SWAP(InstrArgByte::Register(Reg::L)),
            0x36 => Instruction::SWAP(InstrArgByte::AddressRegister(WReg::HL)),
            0x37 => Instruction::SWAP(InstrArgByte::Register(Reg::A)),

            0x38 => Instruction::SRL(InstrArgByte::Register(Reg::B)),
            0x39 => Instruction::SRL(InstrArgByte::Register(Reg::C)),
            0x3A => Instruction::SRL(InstrArgByte::Register(Reg::D)),
            0x3B => Instruction::SRL(InstrArgByte::Register(Reg::E)),
            0x3C => Instruction::SRL(InstrArgByte::Register(Reg::H)),
            0x3D => Instruction::SRL(InstrArgByte::Register(Reg::L)),
            0x3E => Instruction::SRL(InstrArgByte::AddressRegister(WReg::HL)),
            0x3F => Instruction::SRL(InstrArgByte::Register(Reg::A)),

            0x40 => Instruction::BIT(0, InstrArgByte::Register(Reg::B)),
            0x41 => Instruction::BIT(0, InstrArgByte::Register(Reg::C)),
            0x42 => Instruction::BIT(0, InstrArgByte::Register(Reg::D)),
            0x43 => Instruction::BIT(0, InstrArgByte::Register(Reg::E)),
            0x44 => Instruction::BIT(0, InstrArgByte::Register(Reg::H)),
            0x45 => Instruction::BIT(0, InstrArgByte::Register(Reg::L)),
            0x46 => Instruction::BIT(0, InstrArgByte::AddressRegister(WReg::HL)),
            0x47 => Instruction::BIT(0, InstrArgByte::Register(Reg::A)),

            0x48 => Instruction::BIT(1, InstrArgByte::Register(Reg::B)),
            0x49 => Instruction::BIT(1, InstrArgByte::Register(Reg::C)),
            0x4A => Instruction::BIT(1, InstrArgByte::Register(Reg::D)),
            0x4B => Instruction::BIT(1, InstrArgByte::Register(Reg::E)),
            0x4C => Instruction::BIT(1, InstrArgByte::Register(Reg::H)),
            0x4D => Instruction::BIT(1, InstrArgByte::Register(Reg::L)),
            0x4E => Instruction::BIT(1, InstrArgByte::AddressRegister(WReg::HL)),
            0x4F => Instruction::BIT(1, InstrArgByte::Register(Reg::A)),

            0x50 => Instruction::BIT(2, InstrArgByte::Register(Reg::B)),
            0x51 => Instruction::BIT(2, InstrArgByte::Register(Reg::C)),
            0x52 => Instruction::BIT(2, InstrArgByte::Register(Reg::D)),
            0x53 => Instruction::BIT(2, InstrArgByte::Register(Reg::E)),
            0x54 => Instruction::BIT(2, InstrArgByte::Register(Reg::H)),
            0x55 => Instruction::BIT(2, InstrArgByte::Register(Reg::L)),
            0x56 => Instruction::BIT(2, InstrArgByte::AddressRegister(WReg::HL)),
            0x57 => Instruction::BIT(2, InstrArgByte::Register(Reg::A)),

            0x58 => Instruction::BIT(3, InstrArgByte::Register(Reg::B)),
            0x59 => Instruction::BIT(3, InstrArgByte::Register(Reg::C)),
            0x5A => Instruction::BIT(3, InstrArgByte::Register(Reg::D)),
            0x5B => Instruction::BIT(3, InstrArgByte::Register(Reg::E)),
            0x5C => Instruction::BIT(3, InstrArgByte::Register(Reg::H)),
            0x5D => Instruction::BIT(3, InstrArgByte::Register(Reg::L)),
            0x5E => Instruction::BIT(3, InstrArgByte::AddressRegister(WReg::HL)),
            0x5F => Instruction::BIT(3, InstrArgByte::Register(Reg::A)),

            0x60 => Instruction::BIT(4, InstrArgByte::Register(Reg::B)),
            0x61 => Instruction::BIT(4, InstrArgByte::Register(Reg::C)),
            0x62 => Instruction::BIT(4, InstrArgByte::Register(Reg::D)),
            0x63 => Instruction::BIT(4, InstrArgByte::Register(Reg::E)),
            0x64 => Instruction::BIT(4, InstrArgByte::Register(Reg::H)),
            0x65 => Instruction::BIT(4, InstrArgByte::Register(Reg::L)),
            0x66 => Instruction::BIT(4, InstrArgByte::AddressRegister(WReg::HL)),
            0x67 => Instruction::BIT(4, InstrArgByte::Register(Reg::A)),

            0x68 => Instruction::BIT(5, InstrArgByte::Register(Reg::B)),
            0x69 => Instruction::BIT(5, InstrArgByte::Register(Reg::C)),
            0x6A => Instruction::BIT(5, InstrArgByte::Register(Reg::D)),
            0x6B => Instruction::BIT(5, InstrArgByte::Register(Reg::E)),
            0x6C => Instruction::BIT(5, InstrArgByte::Register(Reg::H)),
            0x6D => Instruction::BIT(5, InstrArgByte::Register(Reg::L)),
            0x6E => Instruction::BIT(5, InstrArgByte::AddressRegister(WReg::HL)),
            0x6F => Instruction::BIT(5, InstrArgByte::Register(Reg::A)),

            0x70 => Instruction::BIT(6, InstrArgByte::Register(Reg::B)),
            0x71 => Instruction::BIT(6, InstrArgByte::Register(Reg::C)),
            0x72 => Instruction::BIT(6, InstrArgByte::Register(Reg::D)),
            0x73 => Instruction::BIT(6, InstrArgByte::Register(Reg::E)),
            0x74 => Instruction::BIT(6, InstrArgByte::Register(Reg::H)),
            0x75 => Instruction::BIT(6, InstrArgByte::Register(Reg::L)),
            0x76 => Instruction::BIT(6, InstrArgByte::AddressRegister(WReg::HL)),
            0x77 => Instruction::BIT(6, InstrArgByte::Register(Reg::A)),

            0x78 => Instruction::BIT(7, InstrArgByte::Register(Reg::B)),
            0x79 => Instruction::BIT(7, InstrArgByte::Register(Reg::C)),
            0x7A => Instruction::BIT(7, InstrArgByte::Register(Reg::D)),
            0x7B => Instruction::BIT(7, InstrArgByte::Register(Reg::E)),
            0x7C => Instruction::BIT(7, InstrArgByte::Register(Reg::H)),
            0x7D => Instruction::BIT(7, InstrArgByte::Register(Reg::L)),
            0x7E => Instruction::BIT(7, InstrArgByte::AddressRegister(WReg::HL)),
            0x7F => Instruction::BIT(7, InstrArgByte::Register(Reg::A)),

            0x80 => Instruction::RES(0, InstrArgByte::Register(Reg::B)),
            0x81 => Instruction::RES(0, InstrArgByte::Register(Reg::C)),
            0x82 => Instruction::RES(0, InstrArgByte::Register(Reg::D)),
            0x83 => Instruction::RES(0, InstrArgByte::Register(Reg::E)),
            0x84 => Instruction::RES(0, InstrArgByte::Register(Reg::H)),
            0x85 => Instruction::RES(0, InstrArgByte::Register(Reg::L)),
            0x86 => Instruction::RES(0, InstrArgByte::AddressRegister(WReg::HL)),
            0x87 => Instruction::RES(0, InstrArgByte::Register(Reg::A)),

            0x88 => Instruction::RES(1, InstrArgByte::Register(Reg::B)),
            0x89 => Instruction::RES(1, InstrArgByte::Register(Reg::C)),
            0x8A => Instruction::RES(1, InstrArgByte::Register(Reg::D)),
            0x8B => Instruction::RES(1, InstrArgByte::Register(Reg::E)),
            0x8C => Instruction::RES(1, InstrArgByte::Register(Reg::H)),
            0x8D => Instruction::RES(1, InstrArgByte::Register(Reg::L)),
            0x8E => Instruction::RES(1, InstrArgByte::AddressRegister(WReg::HL)),
            0x8F => Instruction::RES(1, InstrArgByte::Register(Reg::A)),

            0x90 => Instruction::RES(2, InstrArgByte::Register(Reg::B)),
            0x91 => Instruction::RES(2, InstrArgByte::Register(Reg::C)),
            0x92 => Instruction::RES(2, InstrArgByte::Register(Reg::D)),
            0x93 => Instruction::RES(2, InstrArgByte::Register(Reg::E)),
            0x94 => Instruction::RES(2, InstrArgByte::Register(Reg::H)),
            0x95 => Instruction::RES(2, InstrArgByte::Register(Reg::L)),
            0x96 => Instruction::RES(2, InstrArgByte::AddressRegister(WReg::HL)),
            0x97 => Instruction::RES(2, InstrArgByte::Register(Reg::A)),

            0x98 => Instruction::RES(3, InstrArgByte::Register(Reg::B)),
            0x99 => Instruction::RES(3, InstrArgByte::Register(Reg::C)),
            0x9A => Instruction::RES(3, InstrArgByte::Register(Reg::D)),
            0x9B => Instruction::RES(3, InstrArgByte::Register(Reg::E)),
            0x9C => Instruction::RES(3, InstrArgByte::Register(Reg::H)),
            0x9D => Instruction::RES(3, InstrArgByte::Register(Reg::L)),
            0x9E => Instruction::RES(3, InstrArgByte::AddressRegister(WReg::HL)),
            0x9F => Instruction::RES(3, InstrArgByte::Register(Reg::A)),

            0xA0 => Instruction::RES(4, InstrArgByte::Register(Reg::B)),
            0xA1 => Instruction::RES(4, InstrArgByte::Register(Reg::C)),
            0xA2 => Instruction::RES(4, InstrArgByte::Register(Reg::D)),
            0xA3 => Instruction::RES(4, InstrArgByte::Register(Reg::E)),
            0xA4 => Instruction::RES(4, InstrArgByte::Register(Reg::H)),
            0xA5 => Instruction::RES(4, InstrArgByte::Register(Reg::L)),
            0xA6 => Instruction::RES(4, InstrArgByte::AddressRegister(WReg::HL)),
            0xA7 => Instruction::RES(4, InstrArgByte::Register(Reg::A)),

            0xA8 => Instruction::RES(5, InstrArgByte::Register(Reg::B)),
            0xA9 => Instruction::RES(5, InstrArgByte::Register(Reg::C)),
            0xAA => Instruction::RES(5, InstrArgByte::Register(Reg::D)),
            0xAB => Instruction::RES(5, InstrArgByte::Register(Reg::E)),
            0xAC => Instruction::RES(5, InstrArgByte::Register(Reg::H)),
            0xAD => Instruction::RES(5, InstrArgByte::Register(Reg::L)),
            0xAE => Instruction::RES(5, InstrArgByte::AddressRegister(WReg::HL)),
            0xAF => Instruction::RES(5, InstrArgByte::Register(Reg::A)),

            0xB0 => Instruction::RES(6, InstrArgByte::Register(Reg::B)),
            0xB1 => Instruction::RES(6, InstrArgByte::Register(Reg::C)),
            0xB2 => Instruction::RES(6, InstrArgByte::Register(Reg::D)),
            0xB3 => Instruction::RES(6, InstrArgByte::Register(Reg::E)),
            0xB4 => Instruction::RES(6, InstrArgByte::Register(Reg::H)),
            0xB5 => Instruction::RES(6, InstrArgByte::Register(Reg::L)),
            0xB6 => Instruction::RES(6, InstrArgByte::AddressRegister(WReg::HL)),
            0xB7 => Instruction::RES(6, InstrArgByte::Register(Reg::A)),

            0xB8 => Instruction::RES(7, InstrArgByte::Register(Reg::B)),
            0xB9 => Instruction::RES(7, InstrArgByte::Register(Reg::C)),
            0xBA => Instruction::RES(7, InstrArgByte::Register(Reg::D)),
            0xBB => Instruction::RES(7, InstrArgByte::Register(Reg::E)),
            0xBC => Instruction::RES(7, InstrArgByte::Register(Reg::H)),
            0xBD => Instruction::RES(7, InstrArgByte::Register(Reg::L)),
            0xBE => Instruction::RES(7, InstrArgByte::AddressRegister(WReg::HL)),
            0xBF => Instruction::RES(7, InstrArgByte::Register(Reg::A)),

            0xC0 => Instruction::SET(0, InstrArgByte::Register(Reg::B)),
            0xC1 => Instruction::SET(0, InstrArgByte::Register(Reg::C)),
            0xC2 => Instruction::SET(0, InstrArgByte::Register(Reg::D)),
            0xC3 => Instruction::SET(0, InstrArgByte::Register(Reg::E)),
            0xC4 => Instruction::SET(0, InstrArgByte::Register(Reg::H)),
            0xC5 => Instruction::SET(0, InstrArgByte::Register(Reg::L)),
            0xC6 => Instruction::SET(0, InstrArgByte::AddressRegister(WReg::HL)),
            0xC7 => Instruction::SET(0, InstrArgByte::Register(Reg::A)),

            0xC8 => Instruction::SET(1, InstrArgByte::Register(Reg::B)),
            0xC9 => Instruction::SET(1, InstrArgByte::Register(Reg::C)),
            0xCA => Instruction::SET(1, InstrArgByte::Register(Reg::D)),
            0xCB => Instruction::SET(1, InstrArgByte::Register(Reg::E)),
            0xCC => Instruction::SET(1, InstrArgByte::Register(Reg::H)),
            0xCD => Instruction::SET(1, InstrArgByte::Register(Reg::L)),
            0xCE => Instruction::SET(1, InstrArgByte::AddressRegister(WReg::HL)),
            0xCF => Instruction::SET(1, InstrArgByte::Register(Reg::A)),

            0xD0 => Instruction::SET(2, InstrArgByte::Register(Reg::B)),
            0xD1 => Instruction::SET(2, InstrArgByte::Register(Reg::C)),
            0xD2 => Instruction::SET(2, InstrArgByte::Register(Reg::D)),
            0xD3 => Instruction::SET(2, InstrArgByte::Register(Reg::E)),
            0xD4 => Instruction::SET(2, InstrArgByte::Register(Reg::H)),
            0xD5 => Instruction::SET(2, InstrArgByte::Register(Reg::L)),
            0xD6 => Instruction::SET(2, InstrArgByte::AddressRegister(WReg::HL)),
            0xD7 => Instruction::SET(2, InstrArgByte::Register(Reg::A)),

            0xD8 => Instruction::SET(3, InstrArgByte::Register(Reg::B)),
            0xD9 => Instruction::SET(3, InstrArgByte::Register(Reg::C)),
            0xDA => Instruction::SET(3, InstrArgByte::Register(Reg::D)),
            0xDB => Instruction::SET(3, InstrArgByte::Register(Reg::E)),
            0xDC => Instruction::SET(3, InstrArgByte::Register(Reg::H)),
            0xDD => Instruction::SET(3, InstrArgByte::Register(Reg::L)),
            0xDE => Instruction::SET(3, InstrArgByte::AddressRegister(WReg::HL)),
            0xDF => Instruction::SET(3, InstrArgByte::Register(Reg::A)),

            0xE0 => Instruction::SET(4, InstrArgByte::Register(Reg::B)),
            0xE1 => Instruction::SET(4, InstrArgByte::Register(Reg::C)),
            0xE2 => Instruction::SET(4, InstrArgByte::Register(Reg::D)),
            0xE3 => Instruction::SET(4, InstrArgByte::Register(Reg::E)),
            0xE4 => Instruction::SET(4, InstrArgByte::Register(Reg::H)),
            0xE5 => Instruction::SET(4, InstrArgByte::Register(Reg::L)),
            0xE6 => Instruction::SET(4, InstrArgByte::AddressRegister(WReg::HL)),
            0xE7 => Instruction::SET(4, InstrArgByte::Register(Reg::A)),

            0xE8 => Instruction::SET(5, InstrArgByte::Register(Reg::B)),
            0xE9 => Instruction::SET(5, InstrArgByte::Register(Reg::C)),
            0xEA => Instruction::SET(5, InstrArgByte::Register(Reg::D)),
            0xEB => Instruction::SET(5, InstrArgByte::Register(Reg::E)),
            0xEC => Instruction::SET(5, InstrArgByte::Register(Reg::H)),
            0xED => Instruction::SET(5, InstrArgByte::Register(Reg::L)),
            0xEE => Instruction::SET(5, InstrArgByte::AddressRegister(WReg::HL)),
            0xEF => Instruction::SET(5, InstrArgByte::Register(Reg::A)),

            0xF0 => Instruction::SET(6, InstrArgByte::Register(Reg::B)),
            0xF1 => Instruction::SET(6, InstrArgByte::Register(Reg::C)),
            0xF2 => Instruction::SET(6, InstrArgByte::Register(Reg::D)),
            0xF3 => Instruction::SET(6, InstrArgByte::Register(Reg::E)),
            0xF4 => Instruction::SET(6, InstrArgByte::Register(Reg::H)),
            0xF5 => Instruction::SET(6, InstrArgByte::Register(Reg::L)),
            0xF6 => Instruction::SET(6, InstrArgByte::AddressRegister(WReg::HL)),
            0xF7 => Instruction::SET(6, InstrArgByte::Register(Reg::A)),

            0xF8 => Instruction::SET(7, InstrArgByte::Register(Reg::B)),
            0xF9 => Instruction::SET(7, InstrArgByte::Register(Reg::C)),
            0xFA => Instruction::SET(7, InstrArgByte::Register(Reg::D)),
            0xFB => Instruction::SET(7, InstrArgByte::Register(Reg::E)),
            0xFC => Instruction::SET(7, InstrArgByte::Register(Reg::H)),
            0xFD => Instruction::SET(7, InstrArgByte::Register(Reg::L)),
            0xFE => Instruction::SET(7, InstrArgByte::AddressRegister(WReg::HL)),
            0xFF => Instruction::SET(7, InstrArgByte::Register(Reg::A)),
        };

        self.execute_instruction(memory_bus, instruction)?;
        Ok(get_cb_opcode_delay(opcode))
    }
}
