use crate::cpu::CPU;

#[allow(non_camel_case_types)]
pub enum Instruction {
    /* LD nn,n */
    LD_IMMEDIATE(Register, Immediate),
    LD_IMMEDIATE_PAIR(RegisterPair, Immediate),
    LD_IMMEDIATE_16(WordRegister, Immediate16),

    /* LD r1,r2 */
    LD_REG_REG(Register, Register),
    LD_REG_PAIR(Register, RegisterPair),
    LD_PAIR_REG(RegisterPair, Register),
    LD_REG_ADDRESS(Register, Address16),
    LD_ADDRESS_REG(Address16, Register),

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
    ADD_SP(Immediate),

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

#[derive(Clone, Copy)]
pub enum WordRegister {
    AF,
    BC,
    DE,
    HL,
    SP,
    PC,
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

#[derive(Clone, Copy)]
pub struct Immediate(u8);
#[derive(Clone, Copy)]
pub struct Immediate16(u16);
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

#[derive(Clone)]
pub enum RegisterTarget {
    Register(Register),
    HL,
}

#[derive(Clone, Copy)]
pub struct Bit(u8);

impl From<Bit> for u8 {
    fn from(bit: Bit) -> u8 {
        bit.0
    }
}

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
            Instruction::LD_IMMEDIATE(reg, imm) => self.set_register(reg, imm.into()),
            Instruction::LD_IMMEDIATE_PAIR(pair, imm) => {
                self.set_word_register(pair.into(), imm.into())
            }
            Instruction::LD_IMMEDIATE_16(word_reg, imm) => {
                self.set_word_register(word_reg, imm.into())
            }
            Instruction::LD_REG_REG(reg1, reg2) => self.set_register(reg1, self.get_register(reg2)),
            Instruction::LD_REG_PAIR(reg1, pair2) => {
                let mem_addr: usize = self.get_word_register(pair2.into()).into();
                let mem_value = self.get_memory_value(mem_addr);
                self.set_register(reg1, mem_value);
            }
            Instruction::LD_PAIR_REG(pair1, reg2) => {
                let mem_addr: usize = self.get_word_register(pair1.into()).into();
                let mem_value = self.get_register(reg2);
                self.set_memory_value(mem_addr, mem_value);
            }
            Instruction::LD_REG_ADDRESS(reg, addr) => {
                let mem_addr: usize = addr.into();
                let mem_value = self.get_memory_value(mem_addr);
                self.set_register(reg, mem_value);
            }
            Instruction::LD_ADDRESS_REG(addr, reg) => {
                let mem_addr: usize = addr.into();
                let reg_value = self.get_register(reg);
                self.set_memory_value(mem_addr, reg_value);
            }
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
                self.execute(Instruction::LD_REG_PAIR(Register::A, RegisterPair::HL));
                self.execute(Instruction::DEC_WORD(WordRegister::HL));
            }
            Instruction::LDD_A_INTO_HL => {
                self.execute(Instruction::LD_PAIR_REG(RegisterPair::HL, Register::A));
                self.execute(Instruction::DEC_WORD(WordRegister::HL));
            }
            Instruction::LDI_A_FROM_HL => {
                self.execute(Instruction::LD_REG_PAIR(Register::A, RegisterPair::HL));
                self.execute(Instruction::INC_WORD(WordRegister::HL));
            }
            Instruction::LDI_A_INTO_HL => {
                self.execute(Instruction::LD_PAIR_REG(RegisterPair::HL, Register::A));
                self.execute(Instruction::INC_WORD(WordRegister::HL));
            }
            Instruction::LDH_A_INTO_OFFSET(imm) => {
                let mem_addr = Address16(0xff00 + u8::from(imm) as u16);
                self.execute(Instruction::LD_ADDRESS_REG(mem_addr, Register::A));
            }
            Instruction::LDH_A_FROM_OFFSET(imm) => {
                let mem_addr = Address16(0xff00 + u8::from(imm) as u16);
                self.execute(Instruction::LD_REG_ADDRESS(Register::A, mem_addr));
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
                let imm: u16 = u8::from(imm).into();

                let (sum, carry) = sp.overflowing_add(imm);

                self.registers.f.zero = false;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = ((sp & 0xfff) + (imm & 0xfff)) & 0x1000 == 0x1000;
                self.registers.f.carry = carry;
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
                let (diff, overflow) = self.registers.a.overflowing_sub(value);
                // Set F register flags.
                self.registers.f.zero = diff == 0;
                self.registers.f.subtract = true;
                self.registers.f.half_carry = (self.registers.a & 0xf) < (value & 0xf);
                self.registers.f.carry = !overflow;

                self.registers.a = diff;
            }
            Instruction::SBC(arith_target) => {
                let value = self.get_arithmetic_value(&arith_target);
                let (partial_diff, overflow1) = self.registers.a.overflowing_sub(value);
                let (diff, overflow2) = partial_diff.overflowing_sub(self.registers.f.carry.into());
                // Set F register flags.
                self.registers.f.zero = diff == 0;
                self.registers.f.subtract = true;
                self.registers.f.half_carry = (self.registers.a & 0xf) < (value & 0xf);
                self.registers.f.carry = !(overflow1 || overflow2);

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
                todo!()
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
            Instruction::DI => todo!(),
            Instruction::EI => todo!(),

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
                let next_instr_addr = self.pc + 2;
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
            Instruction::RETI => todo!(),
        }
    }
}
