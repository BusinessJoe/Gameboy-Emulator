/// The eight 8-bit CPU registers. Does not include the 16-bit SP and PC registers.
/// Some registers can be paired up and treated as 16-bit registers.
#[derive(Default, Debug)]
pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: FlagRegister,
    pub h: u8,
    pub l: u8,
}

/// Macro to generate a function that gets the value in a join register.
macro_rules! get_joint_register {
    ($name:ident, $first:ident, $second:ident) => {
        #[doc = concat!("Gets the joint register ", stringify!($first), stringify!($second), ".")]
        pub fn $name(&self) -> u16 {
            (u8::from(self.$first) as u16) << 8 | (u8::from(self.$second) as u16)
        }
    };
}

/// Macro to generate a function that sets the value in a join register.
macro_rules! set_joint_register {
    ($name:ident, $first:ident, $second:ident) => {
        #[doc = concat!("Sets the joint register ", stringify!($first), stringify!($second), ".")]
        pub fn $name(&mut self, value: u16) {
            self.$first = (((value >> 8) & 0xff) as u8).into();
            self.$second = ((value & 0xff) as u8).into();
        }
    };
}

impl Registers {
    // AF
    get_joint_register!(get_af, a, f);
    set_joint_register!(set_af, a, f);

    // BC
    get_joint_register!(get_bc, b, c);
    set_joint_register!(set_bc, b, c);

    // DE
    get_joint_register!(get_de, d, e);
    set_joint_register!(set_de, d, e);

    // HL
    get_joint_register!(get_hl, h, l);
    set_joint_register!(set_hl, h, l);
}

/// The flag register has meanings assigned to its bits.
#[derive(Debug, Default, Clone, Copy)]
pub struct FlagRegister {
    /// This bit is set when the result of a math op is zero or two values match when using the CP
    /// instruction.
    pub zero: bool,

    /// This bit is set if a subtraction was performed in the last math operation.
    pub subtract: bool,

    /// This bit is set if a carry occurred from the lower nibble in the last math operation.
    pub half_carry: bool,

    /// This bit is set if a carry occurred from the last math operation or if register A is the
    /// smaller value when executing the CP instruction.
    pub carry: bool,
}

const ZERO_FLAG_BYTE_POSITION: u8 = 7;
const SUBTRACT_FLAG_BYTE_POSITION: u8 = 6;
const HALF_CARRY_FLAG_BYTE_POSITION: u8 = 5;
const CARRY_FLAG_BYTE_POSITION: u8 = 4;

impl std::convert::From<FlagRegister> for u8 {
    fn from(flag: FlagRegister) -> u8 {
        u8::from(flag.zero) << ZERO_FLAG_BYTE_POSITION
            | u8::from(flag.subtract) << SUBTRACT_FLAG_BYTE_POSITION
            | u8::from(flag.half_carry) << HALF_CARRY_FLAG_BYTE_POSITION
            | u8::from(flag.carry) << CARRY_FLAG_BYTE_POSITION
    }
}

impl std::convert::From<u8> for FlagRegister {
    fn from(byte: u8) -> Self {
        let zero = ((byte >> ZERO_FLAG_BYTE_POSITION) & 0b1) == 1;
        let subtract = ((byte >> SUBTRACT_FLAG_BYTE_POSITION) & 0b1) == 1;
        let half_carry = ((byte >> HALF_CARRY_FLAG_BYTE_POSITION) & 0b1) == 1;
        let carry = ((byte >> CARRY_FLAG_BYTE_POSITION) & 0b1) == 1;

        Self {
            zero,
            subtract,
            half_carry,
            carry,
        }
    }
}
