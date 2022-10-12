use log::{error,debug};
use crate::cpu::CPU;

#[allow(non_camel_case_types)]
pub enum Instruction {
    /* LD nn,n */
    LD(Box<dyn CPUWritable<u8>>, Box<dyn CPUReadable<u8>>),
    LD_16(Box<dyn CPUWritable<u16>>, Box<dyn CPUReadable<u16>>),

    /* LD SP,HL */
    LDHL_SP(SignedImmediate),

    /* LDD */
    LDD_A_FROM_HL,
    LDD_A_INTO_HL,

    /* LDI */
    LDI_A_FROM_HL,
    LDI_A_INTO_HL,

    /* LDH */
    LDH_A_INTO_OFFSET(Immediate),
    LDH_A_FROM_OFFSET(Immediate),

    PUSH(RegisterPair),
    POP(RegisterPair),

    /* ADD */
    ADD(ArithmeticTarget),
    ADD_HL(WordRegister),
    ADD_SP(SignedImmediate),

    /* ADC */
    ADC(ArithmeticTarget),

    /* SUB */
    SUB(ArithmeticTarget),

    /* SBC */
    SBC(ArithmeticTarget),

    AND(ArithmeticTarget),

    OR(ArithmeticTarget),

    XOR(ArithmeticTarget),

    CP(ArithmeticTarget),

    INC(RegisterTarget),
    INC_WORD(WordRegister),

    DEC(RegisterTarget),
    DEC_WORD(WordRegister),

    SWAP(RegisterTarget),

    DAA,

    CPL,

    CCF,
    SCF,

    NOP,

    HALT,

    STOP,

    DI,

    EI,

    RLC(RegisterTarget),
    RL(RegisterTarget),
    RRC(RegisterTarget),
    RR(RegisterTarget),

    SLA(RegisterTarget),
    SRA(RegisterTarget),
    SRL(RegisterTarget),

    BIT(Bit, RegisterTarget),
    SET(Bit, RegisterTarget),
    RES(Bit, RegisterTarget),

    JP(Address16),
    JP_CONDITION(Flag, Address16),
    JP_HL,

    JR(SignedImmediate),
    JR_CONDITION(Flag, SignedImmediate),

    CALL(Address16),
    CALL_CONDITION(Flag, Address16),

    RST(Immediate),

    RET,
    RET_CONDITION(Flag),

    RETI,
}

pub trait CPUReadable<T> {
    fn get(&self, cpu: &CPU) -> T;
}

pub trait CPUWritable<T> {
    fn set(&self, cpu: &mut CPU, value: T);
}

#[derive(Clone, Copy)]
pub enum Register {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

impl CPUReadable<u8> for Register {
    fn get(&self, cpu: &CPU) -> u8 {
        match self {
            Register::A => cpu.registers.a,
            Register::B => cpu.registers.b,
            Register::C => cpu.registers.c,
            Register::D => cpu.registers.d,
            Register::E => cpu.registers.e,
            Register::H => cpu.registers.h,
            Register::L => cpu.registers.l,
        }
    }
}

impl CPUWritable<u8> for Register {
    fn set(&self, cpu: &mut CPU, value: u8) {
        match self {
            Register::A => cpu.registers.a = value,
            Register::B => cpu.registers.b = value,
            Register::C => cpu.registers.c = value,
            Register::D => cpu.registers.d = value,
            Register::E => cpu.registers.e = value,
            Register::H => cpu.registers.h = value,
            Register::L => cpu.registers.l = value,
        }
    }
}

#[derive(Clone, Copy)]
pub enum WordRegister {
    AF,
    BC,
    DE,
    HL,
    SP,
    PC,
}

impl CPUReadable<u16> for WordRegister {
    fn get(&self, cpu: &CPU) -> u16 {
        match self {
            WordRegister::AF => cpu.registers.get_af(),
            WordRegister::BC => cpu.registers.get_bc(),
            WordRegister::DE => cpu.registers.get_de(),
            WordRegister::HL => cpu.registers.get_hl(),
            WordRegister::SP => cpu.sp,
            WordRegister::PC => cpu.pc,
        }
    }
}

impl CPUWritable<u16> for WordRegister {
    fn set(&self, cpu: &mut CPU, value: u16) {
        match self {
            WordRegister::AF => cpu.registers.set_af(value),
            WordRegister::BC => cpu.registers.set_bc(value),
            WordRegister::DE => cpu.registers.set_de(value),
            WordRegister::HL => cpu.registers.set_hl(value),
            WordRegister::SP => cpu.sp = value,
            WordRegister::PC => cpu.pc = value,
        }
    }
}

impl WordRegister {
    fn into_address(self) -> GoodAddress {
        GoodAddress::WordRegister(self)
    }
}

#[derive(Clone, Copy)]
pub enum RegisterPair {
    AF,
    BC,
    DE,
    HL,
}

impl From<RegisterPair> for WordRegister {
    fn from(pair: RegisterPair) -> Self {
        match pair {
            RegisterPair::AF => WordRegister::AF,
            RegisterPair::BC => WordRegister::BC,
            RegisterPair::DE => WordRegister::DE,
            RegisterPair::HL => WordRegister::HL,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Address(u8);
#[derive(Clone, Copy)]
pub struct Address16(u16);

pub enum GoodAddress {
    Direct(u16),
    WordRegister(WordRegister),
}

impl From<u16> for GoodAddress {
    fn from(value: u16) -> Self {
        GoodAddress::Direct(value)
    }
}

impl From<Address> for usize {
    fn from(addr: Address) -> usize {
        addr.0.into()
    }
}

impl From<Address16> for usize {
    fn from(addr: Address16) -> usize {
        addr.0.into()
    }
}

impl From<Address16> for u16 {
    fn from(addr: Address16) -> u16 {
        addr.0
    }
}

impl CPUReadable<u8> for GoodAddress {
    fn get(&self, cpu: &CPU) -> u8 {
        match *self {
            GoodAddress::Direct(addr) => cpu.get_memory_value(addr as usize),
            GoodAddress::WordRegister(word_reg) => {
                let addr: u16 = word_reg.get(cpu);
                cpu.get_memory_value(addr as usize)
            }
        }
    }
}

impl CPUWritable<u8> for GoodAddress {
    fn set(&self, cpu: &mut CPU, value: u8) {
        match *self {
            GoodAddress::Direct(addr) => cpu.set_memory_value(addr as usize, value),
            GoodAddress::WordRegister(word_reg) => {
                let addr: u16 = word_reg.get(cpu);
                cpu.set_memory_value(addr as usize, value)
            }
        }
    }
}

impl CPUWritable<u16> for GoodAddress {
    fn set(&self, cpu: &mut CPU, value: u16) {
        let addr: usize = match *self {
            GoodAddress::Direct(addr) => addr.into(),
            GoodAddress::WordRegister(word_reg) => {
                word_reg.get(cpu).into()
            }
        };
    
        let bytes = value.to_le_bytes();
        cpu.set_memory_value(addr, bytes[0]);
        cpu.set_memory_value(addr + 1, bytes[1]);
    }
}

#[derive(Clone, Copy)]
pub struct Immediate(u8);

impl CPUReadable<u8> for Immediate {
    fn get(&self, _cpu: &CPU) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy)]
pub struct Immediate16(u16);

impl CPUReadable<u16> for Immediate16 {
    fn get(&self, _cpu: &CPU) -> u16 {
        self.0
    }
}

#[derive(Clone, Copy)]
pub struct SignedImmediate(i8);

impl From<Immediate> for u8 {
    fn from(imm: Immediate) -> u8 {
        imm.0
    }
}

impl From<Immediate> for u16 {
    fn from(imm: Immediate) -> u16 {
        imm.0.into()
    }
}

impl From<Immediate16> for u16 {
    fn from(imm: Immediate16) -> u16 {
        imm.0
    }
}

impl From<SignedImmediate> for i8 {
    fn from(imm: SignedImmediate) -> i8 {
        imm.0
    }
}

pub enum ArithmeticTarget {
    Register(Register),
    RegisterPair(RegisterPair),
    Immediate(Immediate),
}

impl From<Register> for ArithmeticTarget {
    fn from(reg: Register) -> Self {
        ArithmeticTarget::Register(reg)
    }
}

impl From<RegisterPair> for ArithmeticTarget {
    fn from(pair: RegisterPair) -> Self {
        ArithmeticTarget::RegisterPair(pair)
    }
}

impl From<Immediate> for ArithmeticTarget {
    fn from(imm: Immediate) -> Self {
        ArithmeticTarget::Immediate(imm)
    }
}

#[derive(Clone)]
pub enum RegisterTarget {
    Register(Register),
    HL,
}

impl From<Register> for RegisterTarget {
    fn from(reg: Register) -> Self {
        RegisterTarget::Register(reg)
    }
}

struct Offset(Register);

impl CPUReadable<u8> for Offset {
    fn get(&self, cpu: &CPU) -> u8 {
        let addr = 0xff00 + self.0.get(cpu) as u16;
        cpu.get_memory_value(addr.into())
    }
}

impl CPUWritable<u8> for Offset {
    fn set(&self, cpu: &mut CPU, value: u8) {
        let addr = 0xff00 + self.0.get(cpu) as u16;
        cpu.set_memory_value(addr.into(), value);
    }
}

type Bit = u8;

pub enum Flag {
    NZ,
    Z,
    NC,
    C,
}

impl CPU {
    fn get_arithmetic_value(&self, arith_target: &ArithmeticTarget) -> u8 {
        match *arith_target {
            ArithmeticTarget::Register(reg) => self.get_register(reg),
            ArithmeticTarget::RegisterPair(pair) => {
                let mem_addr = self.get_word_register(pair.into());
                self.get_memory_value(mem_addr.into())
            }
            ArithmeticTarget::Immediate(imm) => imm.into(),
        }
    }

    fn get_register_target(&self, reg_target: &RegisterTarget) -> u8 {
        match reg_target {
            RegisterTarget::Register(reg) => self.get_register(*reg),
            RegisterTarget::HL => {
                let mem_addr = self.get_word_register(WordRegister::HL);
                self.get_memory_value(mem_addr.into())
            }
        }
    }

    fn set_register_target(&mut self, reg_target: &RegisterTarget, value: u8) {
        match reg_target {
            RegisterTarget::Register(reg) => self.set_register(*reg, value),
            RegisterTarget::HL => {
                let mem_addr = self.get_word_register(WordRegister::HL);
                self.set_memory_value(mem_addr.into(), value)
            }
        }
    }

    fn test_flag(&self, flag: Flag) -> bool {
        match flag {
            Flag::Z => self.registers.f.zero,
            Flag::NZ => !self.registers.f.zero,
            Flag::C => self.registers.f.carry,
            Flag::NC => !self.registers.f.carry,
        }
    }

    fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::LD(target, source) => target.set(self, source.get(self)),
            Instruction::LD_16(target, source) => target.set(self, source.get(self)),
            // TODO: check validity of F flags
            Instruction::LDHL_SP(signed_immediate) => {
                let signed_immediate: i8 = signed_immediate.into();
                let sum = self.sp.wrapping_add_signed(signed_immediate.into());
                self.set_word_register(WordRegister::HL, sum);
                self.registers.f.zero = false;
                self.registers.f.subtract = false;
                self.registers.f.half_carry =
                    (self.sp ^ i8::from(signed_immediate) as u16 ^ sum) & 0x10 == 0x10;
                self.registers.f.carry =
                    (self.sp ^ i8::from(signed_immediate) as u16 ^ sum) & 0x100 == 0x100;
            }
            Instruction::LDD_A_FROM_HL => {
                Register::A.set(self, WordRegister::HL.into_address().get(self));
                self.execute(Instruction::DEC_WORD(WordRegister::HL));
            }
            Instruction::LDD_A_INTO_HL => {
                WordRegister::HL.into_address().set(self, Register::A.get(self));
                self.execute(Instruction::DEC_WORD(WordRegister::HL));
            }
            Instruction::LDI_A_FROM_HL => {
                Register::A.set(self, WordRegister::HL.into_address().get(self));
                self.execute(Instruction::INC_WORD(WordRegister::HL));
            }
            Instruction::LDI_A_INTO_HL => {
                WordRegister::HL.into_address().set(self, Register::A.get(self));
                self.execute(Instruction::INC_WORD(WordRegister::HL));
            }
            Instruction::LDH_A_INTO_OFFSET(imm) => {
                let mem_addr = GoodAddress::Direct(0xff00 + u8::from(imm) as u16);
                mem_addr.set(self, Register::A.get(self));
            }
            Instruction::LDH_A_FROM_OFFSET(imm) => {
                let mem_addr = GoodAddress::Direct(0xff00 + u8::from(imm) as u16);
                Register::A.set(self, mem_addr.get(self));
            }
            Instruction::PUSH(pair) => {
                let reg_value = self.get_word_register(pair.into());
                let bytes = reg_value.to_le_bytes();
                self.push(bytes[1]);
                self.push(bytes[0]);
            }
            Instruction::POP(pair) => {
                let bytes = [self.pop(), self.pop()];
                let value = u16::from_le_bytes(bytes);
                self.set_word_register(pair.into(), value);
            }

            /* Arithmetic */
            Instruction::ADD(arith_target) => {
                let value = self.get_arithmetic_value(&arith_target);
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
                let value = self.get_word_register(word_reg);
                let hl = self.registers.get_hl();

                let (sum, carry) = value.overflowing_add(hl);

                self.registers.f.subtract = false;
                self.registers.f.half_carry = ((hl & 0xfff) + (value & 0xfff)) & 0x1000 == 0x1000;
                self.registers.f.carry = carry;

                self.registers.set_hl(sum);
            }
            Instruction::ADD_SP(imm) => {
                let sp = self.get_word_register(WordRegister::SP);
                let imm: i16 = i8::from(imm).into();

                let sum = sp.wrapping_add_signed(imm);
                self.set_word_register(WordRegister::SP, sum);

                let unsigned_imm = imm as u16;

                self.registers.f.zero = false;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = (sp ^ unsigned_imm ^ (sum & 0xFFFF)) & 0x10 == 0x10;
                self.registers.f.carry = (sp ^ unsigned_imm ^ (sum & 0xFFFF)) & 0x100 == 0x100;
            }
            Instruction::ADC(arith_target) => {
                let value = self.get_arithmetic_value(&arith_target);
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
                let value = self.get_arithmetic_value(&arith_target);
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
                let value = self.get_arithmetic_value(&arith_target);
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
                let value = self.get_arithmetic_value(&arith_target);
                let result = self.registers.a & value;
                self.registers.a = result;

                self.registers.f.zero = result == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = true;
                self.registers.f.carry = false;
            }
            Instruction::OR(arith_target) => {
                let value = self.get_arithmetic_value(&arith_target);
                let result = self.registers.a | value;
                self.registers.a = result;

                self.registers.f.zero = result == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = false;
            }
            Instruction::XOR(arith_target) => {
                let value = self.get_arithmetic_value(&arith_target);
                let result = self.registers.a ^ value;
                self.registers.a = result;

                self.registers.f.zero = result == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = false;
            }
            Instruction::CP(arith_target) => {
                let value = self.get_arithmetic_value(&arith_target);
                let a = self.registers.a;

                self.registers.f.zero = a == value;
                self.registers.f.subtract = true;
                self.registers.f.half_carry = (a & 0xf) < (value & 0xf);
                self.registers.f.carry = a < value;
            }
            Instruction::INC(reg_target) => {
                let value = self.get_register_target(&reg_target);
                let incremented_value = value.wrapping_add(1);
                self.set_register_target(&reg_target, incremented_value);

                self.registers.f.zero = incremented_value == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = ((value & 0xf) + 1) & 0x10 == 0x10;
            }
            Instruction::INC_WORD(word_reg) => {
                let value = self.get_word_register(word_reg);
                let incremented_value = value.wrapping_add(1);
                self.set_word_register(word_reg, incremented_value);
            }
            Instruction::DEC(reg_target) => {
                let value = self.get_register_target(&reg_target);
                let decremented_value = value.wrapping_sub(1);
                self.set_register_target(&reg_target, decremented_value);

                self.registers.f.zero = decremented_value == 0;
                self.registers.f.subtract = true;
                self.registers.f.half_carry = (value & 0xf) < 1;
            }
            Instruction::DEC_WORD(word_reg) => {
                let value = self.get_word_register(word_reg);
                let decremented_value = value.wrapping_sub(1);
                self.set_word_register(word_reg, decremented_value);
            }

            /* Miscellaneous */
            Instruction::SWAP(reg_target) => {
                let value = self.get_register_target(&reg_target);
                let nibbles = [value & 0xf, value >> 4];
                let swapped_value = nibbles[0] << 4 | nibbles[1];
                self.set_register_target(&reg_target, swapped_value);

                self.registers.f.zero = swapped_value == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = false;
            }
            Instruction::DAA => {
                error!("DAA is not implemented");
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
            Instruction::HALT => todo!(),
            Instruction::STOP => todo!(),
            Instruction::DI => self.interrupt_enabled = false,
            Instruction::EI => self.interrupt_enabled = true,

            /* Rotates & shifts */
            Instruction::RLC(reg_target) => {
                let value = self.get_register_target(&reg_target);
                let bit7 = value >> 7;

                let result = (value << 1) | bit7;
                self.set_register_target(&reg_target, result);

                self.registers.f.zero = result == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = bit7 == 1;
            }
            Instruction::RL(reg_target) => {
                let value = self.get_register_target(&reg_target);
                let bit7 = value >> 7;
                let carry: u8 = self.registers.f.carry.into();

                let result = (value << 1) | carry;
                self.set_register_target(&reg_target, result);

                self.registers.f.zero = result == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = bit7 == 1;
            }
            Instruction::RRC(reg_target) => {
                let value = self.get_register_target(&reg_target);
                let bit0 = value & 0b1;

                let result = (value >> 1) | (bit0 << 7);
                self.set_register_target(&reg_target, result);

                self.registers.f.zero = result == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = bit0 == 1;
            }
            Instruction::RR(reg_target) => {
                let value = self.get_register_target(&reg_target);
                let bit0 = value & 0b1;
                let carry: u8 = self.registers.f.carry.into();

                let result = (value >> 1) | (carry << 7);
                self.set_register_target(&reg_target, result);

                self.registers.f.zero = result == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = bit0 == 1;
            }
            Instruction::SLA(reg_target) => {
                let value = self.get_register_target(&reg_target);
                let bit7 = value >> 7;

                let result = value << 1;
                self.set_register_target(&reg_target, result);

                self.registers.f.zero = result == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = bit7 == 1;
            }
            Instruction::SRA(reg_target) => {
                let value = self.get_register_target(&reg_target);
                let bit0 = value & 0b1;
                let bit7 = value >> 7;

                let result = (value >> 1) | (bit7 << 7);
                self.set_register_target(&reg_target, result);

                self.registers.f.zero = result == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = bit0 == 1;
            }
            Instruction::SRL(reg_target) => {
                let value = self.get_register_target(&reg_target);
                let bit0 = value & 0b1;

                let result = value >> 1;
                self.set_register_target(&reg_target, result);

                self.registers.f.zero = result == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = bit0 == 1;
            }

            /* Bit opcodes */
            Instruction::BIT(bit, reg_target) => {
                let bit: u8 = bit.into();
                let value = self.get_register_target(&reg_target);

                let result = (value >> bit) & 0b1;

                self.registers.f.zero = result == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = true;
            }
            Instruction::SET(bit, reg_target) => {
                let bit: u8 = bit.into();
                let value = self.get_register_target(&reg_target);

                let result = value | (1 << bit);
                self.set_register_target(&reg_target, result);
            }
            Instruction::RES(bit, reg_target) => {
                let bit: u8 = bit.into();
                let value = self.get_register_target(&reg_target);

                let result = value & !(1 << bit);
                self.set_register_target(&reg_target, result);
            }

            /* Jumps */
            Instruction::JP(addr) => {
                let addr: u16 = addr.into();
                self.pc = addr;
            }
            Instruction::JP_CONDITION(flag, addr) => {
                if self.test_flag(flag) {
                    self.execute(Instruction::JP(addr));
                }
            }
            Instruction::JP_HL => {
                let addr: u16 = self.get_word_register(WordRegister::HL);
                self.pc = addr;
            }
            Instruction::JR(imm) => {
                let imm: i8 = imm.into();
                let addr: u16 = self.pc.checked_add_signed(imm.into()).unwrap();
                self.pc = addr;
            }
            Instruction::JR_CONDITION(flag, imm) => {
                if self.test_flag(flag) {
                    self.execute(Instruction::JR(imm));
                }
            }

            /* Calls */
            Instruction::CALL(addr) => {
                // Save address of next instruction to stack
                let next_instr_addr = self.pc;
                let bytes = next_instr_addr.to_le_bytes();
                self.push(bytes[1]);
                self.push(bytes[0]);

                // Load addr into pc
                let addr: u16 = addr.into();
                self.pc = addr;
            }
            Instruction::CALL_CONDITION(flag, addr) => {
                if self.test_flag(flag) {
                    self.execute(Instruction::CALL(addr));
                }
            }

            /* Restarts */
            Instruction::RST(imm) => {
                let bytes = self.pc.to_le_bytes();
                self.push(bytes[1]);
                self.push(bytes[0]);

                self.pc = imm.into();
            }

            /* Returns */
            Instruction::RET => {
                let bytes = [self.pop(), self.pop()];
                self.pc = u16::from_le_bytes(bytes);
            }
            Instruction::RET_CONDITION(flag) => {
                if self.test_flag(flag) {
                    self.execute(Instruction::RET);
                }
            }
            Instruction::RETI => {
                error!("this is sus");
                self.execute(Instruction::EI);
                self.execute(Instruction::RET);
            },
        }
    }

    pub fn execute_regular_opcode(&mut self, opcode: u8) {
        let instruction = match opcode {
            0x00 => Instruction::NOP,
            0x10 => Instruction::STOP,
            0x20 => Instruction::JR_CONDITION(Flag::NZ, SignedImmediate(self.get_signed_byte_from_pc())),
            0x30 => Instruction::JR_CONDITION(Flag::NC, SignedImmediate(self.get_signed_byte_from_pc())),

            0x01 => Instruction::LD_16(Box::new(WordRegister::BC), Box::new(Immediate16(self.get_word_from_pc()))),
            0x11 => Instruction::LD_16(Box::new(WordRegister::DE), Box::new(Immediate16(self.get_word_from_pc()))),
            0x21 => Instruction::LD_16(Box::new(WordRegister::HL), Box::new(Immediate16(self.get_word_from_pc()))),
            0x31 => Instruction::LD_16(Box::new(WordRegister::SP), Box::new(Immediate16(self.get_word_from_pc()))),

            0x02 => Instruction::LD(Box::new(WordRegister::BC.into_address()), Box::new(Register::A)),
            0x12 => Instruction::LD(Box::new(WordRegister::DE.into_address()), Box::new(Register::A)),
            0x22 => Instruction::LDI_A_INTO_HL,
            0x32 => Instruction::LDD_A_INTO_HL,

            0x03 => Instruction::INC_WORD(WordRegister::BC),
            0x13 => Instruction::INC_WORD(WordRegister::DE),
            0x23 => Instruction::INC_WORD(WordRegister::HL),
            0x33 => Instruction::INC_WORD(WordRegister::SP),

            0x04 => Instruction::INC(RegisterTarget::Register(Register::B)),
            0x14 => Instruction::INC(RegisterTarget::Register(Register::D)),
            0x24 => Instruction::INC(RegisterTarget::Register(Register::H)),
            0x34 => Instruction::INC(RegisterTarget::HL),

            0x05 => Instruction::DEC(RegisterTarget::Register(Register::B)),
            0x15 => Instruction::DEC(RegisterTarget::Register(Register::D)),
            0x25 => Instruction::DEC(RegisterTarget::Register(Register::H)),
            0x35 => Instruction::DEC(RegisterTarget::HL),

            0x06 => Instruction::LD(Box::new(Register::B), Box::new(Immediate(self.get_byte_from_pc()))),
            0x16 => Instruction::LD(Box::new(Register::D), Box::new(Immediate(self.get_byte_from_pc()))),
            0x26 => Instruction::LD(Box::new(Register::H), Box::new(Immediate(self.get_byte_from_pc()))),
            0x36 => Instruction::LD(Box::new(WordRegister::HL.into_address()), Box::new(Immediate(self.get_byte_from_pc()))),

            0x07 => Instruction::RLC(RegisterTarget::Register(Register::A)),
            0x17 => Instruction::RL(RegisterTarget::Register(Register::A)),
            0x27 => Instruction::DAA,
            0x37 => Instruction::SCF,

            0x08 => Instruction::LD_16(Box::new(GoodAddress::from(self.get_word_from_pc())), Box::new(WordRegister::SP)),
            0x18 => Instruction::JR(SignedImmediate(self.get_signed_byte_from_pc())),
            0x28 => Instruction::JR_CONDITION(Flag::Z, SignedImmediate(self.get_signed_byte_from_pc())),
            0x38 => Instruction::JR_CONDITION(Flag::C, SignedImmediate(self.get_signed_byte_from_pc())),

            0x09 => Instruction::ADD_HL(WordRegister::BC),
            0x19 => Instruction::ADD_HL(WordRegister::DE),
            0x29 => Instruction::ADD_HL(WordRegister::HL),
            0x39 => Instruction::ADD_HL(WordRegister::SP),

            0x0A => Instruction::LD(Box::new(Register::A), Box::new(WordRegister::BC.into_address())),
            0x1A => Instruction::LD(Box::new(Register::A), Box::new(WordRegister::DE.into_address())),
            0x2A => Instruction::LDI_A_FROM_HL,
            0x3A => Instruction::LDD_A_FROM_HL,

            0x0B => Instruction::DEC_WORD(WordRegister::BC),
            0x1B => Instruction::DEC_WORD(WordRegister::DE),
            0x2B => Instruction::DEC_WORD(WordRegister::HL),
            0x3B => Instruction::DEC_WORD(WordRegister::SP),

            0x0C => Instruction::INC(Register::C.into()),
            0x1C => Instruction::INC(Register::E.into()),
            0x2C => Instruction::INC(Register::L.into()),
            0x3C => Instruction::INC(Register::A.into()),

            0x0D => Instruction::DEC(Register::C.into()),
            0x1D => Instruction::DEC(Register::E.into()),
            0x2D => Instruction::DEC(Register::L.into()),
            0x3D => Instruction::DEC(Register::A.into()),

            0x0E => Instruction::LD(Box::new(Register::C), Box::new(Immediate(self.get_byte_from_pc()))),
            0x1E => Instruction::LD(Box::new(Register::E), Box::new(Immediate(self.get_byte_from_pc()))),
            0x2E => Instruction::LD(Box::new(Register::L), Box::new(Immediate(self.get_byte_from_pc()))),
            0x3E => Instruction::LD(Box::new(Register::A), Box::new(Immediate(self.get_byte_from_pc()))),

            0x0F => Instruction::RRC(Register::A.into()),
            0x1F => Instruction::RR(Register::A.into()),
            0x2F => Instruction::CPL,
            0x3F => Instruction::CCF,

            0x40 => Instruction::LD(Box::new(Register::B), Box::new(Register::B)),
            0x41 => Instruction::LD(Box::new(Register::B), Box::new(Register::C)),
            0x42 => Instruction::LD(Box::new(Register::B), Box::new(Register::D)),
            0x43 => Instruction::LD(Box::new(Register::B), Box::new(Register::E)),
            0x44 => Instruction::LD(Box::new(Register::B), Box::new(Register::H)),
            0x45 => Instruction::LD(Box::new(Register::B), Box::new(Register::L)),
            0x46 => Instruction::LD(Box::new(Register::B), Box::new(WordRegister::HL.into_address())),
            0x47 => Instruction::LD(Box::new(Register::B), Box::new(Register::A)),

            0x48 => Instruction::LD(Box::new(Register::C), Box::new(Register::B)),
            0x49 => Instruction::LD(Box::new(Register::C), Box::new(Register::C)),
            0x4A => Instruction::LD(Box::new(Register::C), Box::new(Register::D)),
            0x4B => Instruction::LD(Box::new(Register::C), Box::new(Register::E)),
            0x4C => Instruction::LD(Box::new(Register::C), Box::new(Register::H)),
            0x4D => Instruction::LD(Box::new(Register::C), Box::new(Register::L)),
            0x4E => Instruction::LD(Box::new(Register::C), Box::new(WordRegister::HL.into_address())),
            0x4F => Instruction::LD(Box::new(Register::C), Box::new(Register::A)),
                
            0x50 => Instruction::LD(Box::new(Register::D), Box::new(Register::B)),
            0x51 => Instruction::LD(Box::new(Register::D), Box::new(Register::C)),
            0x52 => Instruction::LD(Box::new(Register::D), Box::new(Register::D)),
            0x53 => Instruction::LD(Box::new(Register::D), Box::new(Register::E)),
            0x54 => Instruction::LD(Box::new(Register::D), Box::new(Register::H)),
            0x55 => Instruction::LD(Box::new(Register::D), Box::new(Register::L)),
            0x56 => Instruction::LD(Box::new(Register::D), Box::new(WordRegister::HL.into_address())),
            0x57 => Instruction::LD(Box::new(Register::D), Box::new(Register::A)),

            0x58 => Instruction::LD(Box::new(Register::E), Box::new(Register::B)),
            0x59 => Instruction::LD(Box::new(Register::E), Box::new(Register::C)),
            0x5A => Instruction::LD(Box::new(Register::E), Box::new(Register::D)),
            0x5B => Instruction::LD(Box::new(Register::E), Box::new(Register::E)),
            0x5C => Instruction::LD(Box::new(Register::E), Box::new(Register::H)),
            0x5D => Instruction::LD(Box::new(Register::E), Box::new(Register::L)),
            0x5E => Instruction::LD(Box::new(Register::E), Box::new(WordRegister::HL.into_address())),
            0x5F => Instruction::LD(Box::new(Register::E), Box::new(Register::A)),
            
            0x60 => Instruction::LD(Box::new(Register::H), Box::new(Register::B)),
            0x61 => Instruction::LD(Box::new(Register::H), Box::new(Register::C)),
            0x62 => Instruction::LD(Box::new(Register::H), Box::new(Register::D)),
            0x63 => Instruction::LD(Box::new(Register::H), Box::new(Register::E)),
            0x64 => Instruction::LD(Box::new(Register::H), Box::new(Register::H)),
            0x65 => Instruction::LD(Box::new(Register::H), Box::new(Register::L)),
            0x66 => Instruction::LD(Box::new(Register::H), Box::new(WordRegister::HL.into_address())),
            0x67 => Instruction::LD(Box::new(Register::H), Box::new(Register::A)),

            0x68 => Instruction::LD(Box::new(Register::L), Box::new(Register::B)),
            0x69 => Instruction::LD(Box::new(Register::L), Box::new(Register::C)),
            0x6A => Instruction::LD(Box::new(Register::L), Box::new(Register::D)),
            0x6B => Instruction::LD(Box::new(Register::L), Box::new(Register::E)),
            0x6C => Instruction::LD(Box::new(Register::L), Box::new(Register::H)),
            0x6D => Instruction::LD(Box::new(Register::L), Box::new(Register::L)),
            0x6E => Instruction::LD(Box::new(Register::L), Box::new(WordRegister::HL.into_address())),
            0x6F => Instruction::LD(Box::new(Register::L), Box::new(Register::A)),

            0x70 => Instruction::LD(Box::new(WordRegister::HL.into_address()), Box::new(Register::B)),
            0x71 => Instruction::LD(Box::new(WordRegister::HL.into_address()), Box::new(Register::C)),
            0x72 => Instruction::LD(Box::new(WordRegister::HL.into_address()), Box::new(Register::D)),
            0x73 => Instruction::LD(Box::new(WordRegister::HL.into_address()), Box::new(Register::E)),
            0x74 => Instruction::LD(Box::new(WordRegister::HL.into_address()), Box::new(Register::H)),
            0x75 => Instruction::LD(Box::new(WordRegister::HL.into_address()), Box::new(Register::L)),
            0x76 => Instruction::HALT,
            0x77 => Instruction::LD(Box::new(WordRegister::HL.into_address()), Box::new(Register::A)),

            0x78 => Instruction::LD(Box::new(Register::A), Box::new(Register::B)),
            0x79 => Instruction::LD(Box::new(Register::A), Box::new(Register::C)),
            0x7A => Instruction::LD(Box::new(Register::A), Box::new(Register::D)),
            0x7B => Instruction::LD(Box::new(Register::A), Box::new(Register::E)),
            0x7C => Instruction::LD(Box::new(Register::A), Box::new(Register::H)),
            0x7D => Instruction::LD(Box::new(Register::A), Box::new(Register::L)),
            0x7E => Instruction::LD(Box::new(Register::A), Box::new(WordRegister::HL.into_address())),
            0x7F => Instruction::LD(Box::new(Register::A), Box::new(Register::A)),

            0x80 => Instruction::ADD(Register::B.into()),
            0x81 => Instruction::ADD(Register::C.into()),
            0x82 => Instruction::ADD(Register::D.into()),
            0x83 => Instruction::ADD(Register::E.into()),
            0x84 => Instruction::ADD(Register::H.into()),
            0x85 => Instruction::ADD(Register::L.into()),
            0x86 => Instruction::ADD(RegisterPair::HL.into()),
            0x87 => Instruction::ADD(Register::A.into()),

            0x88 => Instruction::ADC(Register::B.into()),
            0x89 => Instruction::ADC(Register::C.into()),
            0x8A => Instruction::ADC(Register::D.into()),
            0x8B => Instruction::ADC(Register::E.into()),
            0x8C => Instruction::ADC(Register::H.into()),
            0x8D => Instruction::ADC(Register::L.into()),
            0x8E => Instruction::ADC(RegisterPair::HL.into()),
            0x8F => Instruction::ADC(Register::A.into()),

            0x90 => Instruction::SUB(Register::B.into()),
            0x91 => Instruction::SUB(Register::C.into()),
            0x92 => Instruction::SUB(Register::D.into()),
            0x93 => Instruction::SUB(Register::E.into()),
            0x94 => Instruction::SUB(Register::H.into()),
            0x95 => Instruction::SUB(Register::L.into()),
            0x96 => Instruction::SUB(RegisterPair::HL.into()),
            0x97 => Instruction::SUB(Register::A.into()),

            0x98 => Instruction::SBC(Register::B.into()),
            0x99 => Instruction::SBC(Register::C.into()),
            0x9A => Instruction::SBC(Register::D.into()),
            0x9B => Instruction::SBC(Register::E.into()),
            0x9C => Instruction::SBC(Register::H.into()),
            0x9D => Instruction::SBC(Register::L.into()),
            0x9E => Instruction::SBC(RegisterPair::HL.into()),
            0x9F => Instruction::SBC(Register::A.into()),

            0xA0 => Instruction::AND(Register::B.into()),
            0xA1 => Instruction::AND(Register::C.into()),
            0xA2 => Instruction::AND(Register::D.into()),
            0xA3 => Instruction::AND(Register::E.into()),
            0xA4 => Instruction::AND(Register::H.into()),
            0xA5 => Instruction::AND(Register::L.into()),
            0xA6 => Instruction::AND(RegisterPair::HL.into()),
            0xA7 => Instruction::AND(Register::A.into()),

            0xA8 => Instruction::XOR(Register::B.into()),
            0xA9 => Instruction::XOR(Register::C.into()),
            0xAA => Instruction::XOR(Register::D.into()),
            0xAB => Instruction::XOR(Register::E.into()),
            0xAC => Instruction::XOR(Register::H.into()),
            0xAD => Instruction::XOR(Register::L.into()),
            0xAE => Instruction::XOR(RegisterPair::HL.into()),
            0xAF => Instruction::XOR(Register::A.into()),

            0xB0 => Instruction::OR(Register::B.into()),
            0xB1 => Instruction::OR(Register::C.into()),
            0xB2 => Instruction::OR(Register::D.into()),
            0xB3 => Instruction::OR(Register::E.into()),
            0xB4 => Instruction::OR(Register::H.into()),
            0xB5 => Instruction::OR(Register::L.into()),
            0xB6 => Instruction::OR(RegisterPair::HL.into()),
            0xB7 => Instruction::OR(Register::A.into()),

            0xB8 => Instruction::CP(Register::B.into()),
            0xB9 => Instruction::CP(Register::C.into()),
            0xBA => Instruction::CP(Register::D.into()),
            0xBB => Instruction::CP(Register::E.into()),
            0xBC => Instruction::CP(Register::H.into()),
            0xBD => Instruction::CP(Register::L.into()),
            0xBE => Instruction::CP(RegisterPair::HL.into()),
            0xBF => Instruction::CP(Register::A.into()),

            0xC0 => Instruction::RET_CONDITION(Flag::NZ),
            0xD0 => Instruction::RET_CONDITION(Flag::NC),
            0xE0 => Instruction::LD(Box::new(GoodAddress::from(self.get_byte_from_pc() as u16 + 0xFF00)), Box::new(Register::A)),
            0xF0 => Instruction::LD(Box::new(Register::A), Box::new(GoodAddress::from(self.get_byte_from_pc() as u16 + 0xFF00))),

            0xC1 => Instruction::POP(RegisterPair::BC),
            0xD1 => Instruction::POP(RegisterPair::DE),
            0xE1 => Instruction::POP(RegisterPair::HL),
            0xF1 => Instruction::POP(RegisterPair::AF),

            0xC2 => Instruction::JP_CONDITION(Flag::NZ, Address16(self.get_word_from_pc())),
            0xD2 => Instruction::JP_CONDITION(Flag::NC, Address16(self.get_word_from_pc())),
            0xE2 => Instruction::LD(Box::new(Offset(Register::C)), Box::new(Register::A)),
            0xF2 => Instruction::LD(Box::new(Register::A), Box::new(Offset(Register::C))),

            0xC3 => Instruction::JP(Address16(self.get_word_from_pc())),
            0xD3 => unimplemented!(),
            0xE3 => unimplemented!(),
            0xF3 => Instruction::DI,

            0xC4 => Instruction::CALL_CONDITION(Flag::NZ, Address16(self.get_word_from_pc())),
            0xD4 => Instruction::CALL_CONDITION(Flag::NC, Address16(self.get_word_from_pc())),
            0xE4 => unimplemented!(),
            0xF4 => unimplemented!(),

            0xC5 => Instruction::PUSH(RegisterPair::BC),
            0xD5 => Instruction::PUSH(RegisterPair::DE),
            0xE5 => Instruction::PUSH(RegisterPair::HL),
            0xF5 => Instruction::PUSH(RegisterPair::AF),

            0xC6 => Instruction::ADD(Immediate(self.get_byte_from_pc()).into()),
            0xD6 => Instruction::SUB(Immediate(self.get_byte_from_pc()).into()),
            0xE6 => Instruction::AND(Immediate(self.get_byte_from_pc()).into()),
            0xF6 => Instruction::OR(Immediate(self.get_byte_from_pc()).into()),

            0xC7 => Instruction::RST(Immediate(0x00)),
            0xD7 => Instruction::RST(Immediate(0x10)),
            0xE7 => Instruction::RST(Immediate(0x20)),
            0xF7 => Instruction::RST(Immediate(0x30)),

            0xC8 => Instruction::RET_CONDITION(Flag::Z),
            0xD8 => Instruction::RET_CONDITION(Flag::C),
            0xE8 => Instruction::ADD_SP(SignedImmediate(self.get_signed_byte_from_pc())),
            0xF8 => Instruction::LDHL_SP(SignedImmediate(self.get_signed_byte_from_pc())),

            0xC9 => Instruction::RET,
            0xD9 => Instruction::RETI,
            0xE9 => Instruction::JP_HL,
            0xF9 => Instruction::LD_16(Box::new(WordRegister::SP), Box::new(WordRegister::HL)),

            0xCA => Instruction::JP_CONDITION(Flag::Z, Address16(self.get_word_from_pc())),
            0xDA => Instruction::JP_CONDITION(Flag::C, Address16(self.get_word_from_pc())),
            0xEA => Instruction::LD(Box::new(GoodAddress::from(self.get_word_from_pc())), Box::new(Register::A)),
            0xFA => Instruction::LD(Box::new(Register::A), Box::new(GoodAddress::from(self.get_word_from_pc()))),

            0xCB => unimplemented!(),
            0xDB => unimplemented!(),
            0xEB => unimplemented!(),
            0xFB => Instruction::EI,

            0xCC => Instruction::CALL_CONDITION(Flag::Z, Address16(self.get_word_from_pc())),
            0xDC => Instruction::CALL_CONDITION(Flag::C, Address16(self.get_word_from_pc())),
            0xEC => unimplemented!(),
            0xFC => unimplemented!(),

            0xCD => Instruction::CALL(Address16(self.get_word_from_pc())),
            0xDD => unimplemented!(),
            0xED => unimplemented!(),
            0xFD => unimplemented!(),

            0xCE => Instruction::ADC(Immediate(self.get_byte_from_pc()).into()),
            0xDE => Instruction::SBC(Immediate(self.get_byte_from_pc()).into()),
            0xEE => Instruction::XOR(Immediate(self.get_byte_from_pc()).into()),
            0xFE => Instruction::CP(Immediate(self.get_byte_from_pc()).into()),

            0xCF => Instruction::RST(Immediate(0x08)),
            0xDF => Instruction::RST(Immediate(0x18)),
            0xEF => Instruction::RST(Immediate(0x28)),
            0xFF => Instruction::RST(Immediate(0x38)),
        };

        self.execute(instruction);
    }

    pub fn execute_cb_opcode(&mut self, opcode: u8) {
        let instruction = match opcode {
            0x00 => Instruction::RLC(Register::B.into()),
            0x01 => Instruction::RLC(Register::C.into()),
            0x02 => Instruction::RLC(Register::D.into()),
            0x03 => Instruction::RLC(Register::E.into()),
            0x04 => Instruction::RLC(Register::H.into()),
            0x05 => Instruction::RLC(Register::L.into()),
            0x06 => Instruction::RLC(RegisterTarget::HL),
            0x07 => Instruction::RLC(Register::A.into()),

            0x08 => Instruction::RRC(Register::B.into()),
            0x09 => Instruction::RRC(Register::C.into()),
            0x0A => Instruction::RRC(Register::D.into()),
            0x0B => Instruction::RRC(Register::E.into()),
            0x0C => Instruction::RRC(Register::H.into()),
            0x0D => Instruction::RRC(Register::L.into()),
            0x0E => Instruction::RRC(RegisterTarget::HL),
            0x0F => Instruction::RRC(Register::A.into()),

            0x10 => Instruction::RL(Register::B.into()),
            0x11 => Instruction::RL(Register::C.into()),
            0x12 => Instruction::RL(Register::D.into()),
            0x13 => Instruction::RL(Register::E.into()),
            0x14 => Instruction::RL(Register::H.into()),
            0x15 => Instruction::RL(Register::L.into()),
            0x16 => Instruction::RL(RegisterTarget::HL),
            0x17 => Instruction::RL(Register::A.into()),

            0x18 => Instruction::RR(Register::B.into()),
            0x19 => Instruction::RR(Register::C.into()),
            0x1A => Instruction::RR(Register::D.into()),
            0x1B => Instruction::RR(Register::E.into()),
            0x1C => Instruction::RR(Register::H.into()),
            0x1D => Instruction::RR(Register::L.into()),
            0x1E => Instruction::RR(RegisterTarget::HL),
            0x1F => Instruction::RR(Register::A.into()),

            0x20 => Instruction::SLA(Register::B.into()),
            0x21 => Instruction::SLA(Register::C.into()),
            0x22 => Instruction::SLA(Register::D.into()),
            0x23 => Instruction::SLA(Register::E.into()),
            0x24 => Instruction::SLA(Register::H.into()),
            0x25 => Instruction::SLA(Register::L.into()),
            0x26 => Instruction::SLA(RegisterTarget::HL),
            0x27 => Instruction::SLA(Register::A.into()),

            0x28 => Instruction::SRA(Register::B.into()),
            0x29 => Instruction::SRA(Register::C.into()),
            0x2A => Instruction::SRA(Register::D.into()),
            0x2B => Instruction::SRA(Register::E.into()),
            0x2C => Instruction::SRA(Register::H.into()),
            0x2D => Instruction::SRA(Register::L.into()),
            0x2E => Instruction::SRA(RegisterTarget::HL),
            0x2F => Instruction::SRA(Register::A.into()),
                
            0x30 => Instruction::SWAP(Register::B.into()),
            0x31 => Instruction::SWAP(Register::C.into()),
            0x32 => Instruction::SWAP(Register::D.into()),
            0x33 => Instruction::SWAP(Register::E.into()),
            0x34 => Instruction::SWAP(Register::H.into()),
            0x35 => Instruction::SWAP(Register::L.into()),
            0x36 => Instruction::SWAP(RegisterTarget::HL),
            0x37 => Instruction::SWAP(Register::A.into()),

            0x38 => Instruction::SRL(Register::B.into()),
            0x39 => Instruction::SRL(Register::C.into()),
            0x3A => Instruction::SRL(Register::D.into()),
            0x3B => Instruction::SRL(Register::E.into()),
            0x3C => Instruction::SRL(Register::H.into()),
            0x3D => Instruction::SRL(Register::L.into()),
            0x3E => Instruction::SRL(RegisterTarget::HL),
            0x3F => Instruction::SRL(Register::A.into()),

            0x40 => Instruction::BIT(0, Register::B.into()),
            0x41 => Instruction::BIT(0, Register::C.into()),
            0x42 => Instruction::BIT(0, Register::D.into()),
            0x43 => Instruction::BIT(0, Register::E.into()),
            0x44 => Instruction::BIT(0, Register::H.into()),
            0x45 => Instruction::BIT(0, Register::L.into()),
            0x46 => Instruction::BIT(0, RegisterTarget::HL),
            0x47 => Instruction::BIT(0, Register::A.into()),

            0x48 => Instruction::BIT(1, Register::B.into()),
            0x49 => Instruction::BIT(1, Register::C.into()),
            0x4A => Instruction::BIT(1, Register::D.into()),
            0x4B => Instruction::BIT(1, Register::E.into()),
            0x4C => Instruction::BIT(1, Register::H.into()),
            0x4D => Instruction::BIT(1, Register::L.into()),
            0x4E => Instruction::BIT(1, RegisterTarget::HL),
            0x4F => Instruction::BIT(1, Register::A.into()),

            0x50 => Instruction::BIT(2, Register::B.into()),
            0x51 => Instruction::BIT(2, Register::C.into()),
            0x52 => Instruction::BIT(2, Register::D.into()),
            0x53 => Instruction::BIT(2, Register::E.into()),
            0x54 => Instruction::BIT(2, Register::H.into()),
            0x55 => Instruction::BIT(2, Register::L.into()),
            0x56 => Instruction::BIT(2, RegisterTarget::HL),
            0x57 => Instruction::BIT(2, Register::A.into()),

            0x58 => Instruction::BIT(3, Register::B.into()),
            0x59 => Instruction::BIT(3, Register::C.into()),
            0x5A => Instruction::BIT(3, Register::D.into()),
            0x5B => Instruction::BIT(3, Register::E.into()),
            0x5C => Instruction::BIT(3, Register::H.into()),
            0x5D => Instruction::BIT(3, Register::L.into()),
            0x5E => Instruction::BIT(3, RegisterTarget::HL),
            0x5F => Instruction::BIT(3, Register::A.into()),

            0x60 => Instruction::BIT(4, Register::B.into()),
            0x61 => Instruction::BIT(4, Register::C.into()),
            0x62 => Instruction::BIT(4, Register::D.into()),
            0x63 => Instruction::BIT(4, Register::E.into()),
            0x64 => Instruction::BIT(4, Register::H.into()),
            0x65 => Instruction::BIT(4, Register::L.into()),
            0x66 => Instruction::BIT(4, RegisterTarget::HL),
            0x67 => Instruction::BIT(4, Register::A.into()),

            0x68 => Instruction::BIT(5, Register::B.into()),
            0x69 => Instruction::BIT(5, Register::C.into()),
            0x6A => Instruction::BIT(5, Register::D.into()),
            0x6B => Instruction::BIT(5, Register::E.into()),
            0x6C => Instruction::BIT(5, Register::H.into()),
            0x6D => Instruction::BIT(5, Register::L.into()),
            0x6E => Instruction::BIT(5, RegisterTarget::HL),
            0x6F => Instruction::BIT(5, Register::A.into()),

            0x70 => Instruction::BIT(6, Register::B.into()),
            0x71 => Instruction::BIT(6, Register::C.into()),
            0x72 => Instruction::BIT(6, Register::D.into()),
            0x73 => Instruction::BIT(6, Register::E.into()),
            0x74 => Instruction::BIT(6, Register::H.into()),
            0x75 => Instruction::BIT(6, Register::L.into()),
            0x76 => Instruction::BIT(6, RegisterTarget::HL),
            0x77 => Instruction::BIT(6, Register::A.into()),

            0x78 => Instruction::BIT(7, Register::B.into()),
            0x79 => Instruction::BIT(7, Register::C.into()),
            0x7A => Instruction::BIT(7, Register::D.into()),
            0x7B => Instruction::BIT(7, Register::E.into()),
            0x7C => Instruction::BIT(7, Register::H.into()),
            0x7D => Instruction::BIT(7, Register::L.into()),
            0x7E => Instruction::BIT(7, RegisterTarget::HL),
            0x7F => Instruction::BIT(7, Register::A.into()),

            0x80 => Instruction::RES(0, Register::B.into()),
            0x81 => Instruction::RES(0, Register::C.into()),
            0x82 => Instruction::RES(0, Register::D.into()),
            0x83 => Instruction::RES(0, Register::E.into()),
            0x84 => Instruction::RES(0, Register::H.into()),
            0x85 => Instruction::RES(0, Register::L.into()),
            0x86 => Instruction::RES(0, RegisterTarget::HL),
            0x87 => Instruction::RES(0, Register::A.into()),

            0x88 => Instruction::RES(1, Register::B.into()),
            0x89 => Instruction::RES(1, Register::C.into()),
            0x8A => Instruction::RES(1, Register::D.into()),
            0x8B => Instruction::RES(1, Register::E.into()),
            0x8C => Instruction::RES(1, Register::H.into()),
            0x8D => Instruction::RES(1, Register::L.into()),
            0x8E => Instruction::RES(1, RegisterTarget::HL),
            0x8F => Instruction::RES(1, Register::A.into()),

            0x90 => Instruction::RES(2, Register::B.into()),
            0x91 => Instruction::RES(2, Register::C.into()),
            0x92 => Instruction::RES(2, Register::D.into()),
            0x93 => Instruction::RES(2, Register::E.into()),
            0x94 => Instruction::RES(2, Register::H.into()),
            0x95 => Instruction::RES(2, Register::L.into()),
            0x96 => Instruction::RES(2, RegisterTarget::HL),
            0x97 => Instruction::RES(2, Register::A.into()),

            0x98 => Instruction::RES(3, Register::B.into()),
            0x99 => Instruction::RES(3, Register::C.into()),
            0x9A => Instruction::RES(3, Register::D.into()),
            0x9B => Instruction::RES(3, Register::E.into()),
            0x9C => Instruction::RES(3, Register::H.into()),
            0x9D => Instruction::RES(3, Register::L.into()),
            0x9E => Instruction::RES(3, RegisterTarget::HL),
            0x9F => Instruction::RES(3, Register::A.into()),

            0xA0 => Instruction::RES(4, Register::B.into()),
            0xA1 => Instruction::RES(4, Register::C.into()),
            0xA2 => Instruction::RES(4, Register::D.into()),
            0xA3 => Instruction::RES(4, Register::E.into()),
            0xA4 => Instruction::RES(4, Register::H.into()),
            0xA5 => Instruction::RES(4, Register::L.into()),
            0xA6 => Instruction::RES(4, RegisterTarget::HL),
            0xA7 => Instruction::RES(4, Register::A.into()),

            0xA8 => Instruction::RES(5, Register::B.into()),
            0xA9 => Instruction::RES(5, Register::C.into()),
            0xAA => Instruction::RES(5, Register::D.into()),
            0xAB => Instruction::RES(5, Register::E.into()),
            0xAC => Instruction::RES(5, Register::H.into()),
            0xAD => Instruction::RES(5, Register::L.into()),
            0xAE => Instruction::RES(5, RegisterTarget::HL),
            0xAF => Instruction::RES(5, Register::A.into()),

            0xB0 => Instruction::RES(6, Register::B.into()),
            0xB1 => Instruction::RES(6, Register::C.into()),
            0xB2 => Instruction::RES(6, Register::D.into()),
            0xB3 => Instruction::RES(6, Register::E.into()),
            0xB4 => Instruction::RES(6, Register::H.into()),
            0xB5 => Instruction::RES(6, Register::L.into()),
            0xB6 => Instruction::RES(6, RegisterTarget::HL),
            0xB7 => Instruction::RES(6, Register::A.into()),

            0xB8 => Instruction::RES(7, Register::B.into()),
            0xB9 => Instruction::RES(7, Register::C.into()),
            0xBA => Instruction::RES(7, Register::D.into()),
            0xBB => Instruction::RES(7, Register::E.into()),
            0xBC => Instruction::RES(7, Register::H.into()),
            0xBD => Instruction::RES(7, Register::L.into()),
            0xBE => Instruction::RES(7, RegisterTarget::HL),
            0xBF => Instruction::RES(7, Register::A.into()),

            0xC0 => Instruction::SET(0, Register::B.into()),
            0xC1 => Instruction::SET(0, Register::C.into()),
            0xC2 => Instruction::SET(0, Register::D.into()),
            0xC3 => Instruction::SET(0, Register::E.into()),
            0xC4 => Instruction::SET(0, Register::H.into()),
            0xC5 => Instruction::SET(0, Register::L.into()),
            0xC6 => Instruction::SET(0, RegisterTarget::HL),
            0xC7 => Instruction::SET(0, Register::A.into()),

            0xC8 => Instruction::SET(1, Register::B.into()),
            0xC9 => Instruction::SET(1, Register::C.into()),
            0xCA => Instruction::SET(1, Register::D.into()),
            0xCB => Instruction::SET(1, Register::E.into()),
            0xCC => Instruction::SET(1, Register::H.into()),
            0xCD => Instruction::SET(1, Register::L.into()),
            0xCE => Instruction::SET(1, RegisterTarget::HL),
            0xCF => Instruction::SET(1, Register::A.into()),

            0xD0 => Instruction::SET(2, Register::B.into()),
            0xD1 => Instruction::SET(2, Register::C.into()),
            0xD2 => Instruction::SET(2, Register::D.into()),
            0xD3 => Instruction::SET(2, Register::E.into()),
            0xD4 => Instruction::SET(2, Register::H.into()),
            0xD5 => Instruction::SET(2, Register::L.into()),
            0xD6 => Instruction::SET(2, RegisterTarget::HL),
            0xD7 => Instruction::SET(2, Register::A.into()),

            0xD8 => Instruction::SET(3, Register::B.into()),
            0xD9 => Instruction::SET(3, Register::C.into()),
            0xDA => Instruction::SET(3, Register::D.into()),
            0xDB => Instruction::SET(3, Register::E.into()),
            0xDC => Instruction::SET(3, Register::H.into()),
            0xDD => Instruction::SET(3, Register::L.into()),
            0xDE => Instruction::SET(3, RegisterTarget::HL),
            0xDF => Instruction::SET(3, Register::A.into()),

            0xE0 => Instruction::SET(4, Register::B.into()),
            0xE1 => Instruction::SET(4, Register::C.into()),
            0xE2 => Instruction::SET(4, Register::D.into()),
            0xE3 => Instruction::SET(4, Register::E.into()),
            0xE4 => Instruction::SET(4, Register::H.into()),
            0xE5 => Instruction::SET(4, Register::L.into()),
            0xE6 => Instruction::SET(4, RegisterTarget::HL),
            0xE7 => Instruction::SET(4, Register::A.into()),

            0xE8 => Instruction::SET(5, Register::B.into()),
            0xE9 => Instruction::SET(5, Register::C.into()),
            0xEA => Instruction::SET(5, Register::D.into()),
            0xEB => Instruction::SET(5, Register::E.into()),
            0xEC => Instruction::SET(5, Register::H.into()),
            0xED => Instruction::SET(5, Register::L.into()),
            0xEE => Instruction::SET(5, RegisterTarget::HL),
            0xEF => Instruction::SET(5, Register::A.into()),

            0xF0 => Instruction::SET(6, Register::B.into()),
            0xF1 => Instruction::SET(6, Register::C.into()),
            0xF2 => Instruction::SET(6, Register::D.into()),
            0xF3 => Instruction::SET(6, Register::E.into()),
            0xF4 => Instruction::SET(6, Register::H.into()),
            0xF5 => Instruction::SET(6, Register::L.into()),
            0xF6 => Instruction::SET(6, RegisterTarget::HL),
            0xF7 => Instruction::SET(6, Register::A.into()),

            0xF8 => Instruction::SET(7, Register::B.into()),
            0xF9 => Instruction::SET(7, Register::C.into()),
            0xFA => Instruction::SET(7, Register::D.into()),
            0xFB => Instruction::SET(7, Register::E.into()),
            0xFC => Instruction::SET(7, Register::H.into()),
            0xFD => Instruction::SET(7, Register::L.into()),
            0xFE => Instruction::SET(7, RegisterTarget::HL),
            0xFF => Instruction::SET(7, Register::A.into()),
        };

        self.execute(instruction);
    }
}
