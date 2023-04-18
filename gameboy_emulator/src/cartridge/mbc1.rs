use log::info;

use crate::bit_field::BitField;

use super::{Address, AddressingError, MbcType, MemoryBankController};

pub enum Mbc1Mode {
    Default,
    Alternative,
}

pub struct Mbc1 {
    ram_enable: u8,
    bank_register_1: BitField<u8>,
    bank_register_2: BitField<u8>,
    banking_mode_register: BitField<u8>,
    ram_bank_register: u8,
    mode: Mbc1Mode,
}

impl Mbc1 {
    pub fn new(mode: Mbc1Mode) -> Self {
        Self {
            ram_enable: 0,
            bank_register_1: BitField::from(1),
            bank_register_2: BitField::from(0),
            banking_mode_register: BitField::from(0),
            ram_bank_register: 0,
            mode,
        }
    }

    fn bank_number(&self, address: Address) -> usize {
        match address {
            0..=0x3fff if self.banking_mode_register.as_value() == 0 => 0,
            0..=0x3fff if self.banking_mode_register.as_value() != 0 => {
                (self.bank_register_2.as_value() << 5).into()
            }
            0x4000..=0x7fff => {
                (self.bank_register_2.as_value() << 5 | self.bank_register_1.as_value()).into()
            }
            _ => panic!(),
        }
    }

    fn read_banked_rom(&self, address: Address, rom: &[u8]) -> Result<u8, AddressingError> {
        let bank_number = self.bank_number(address);
        let rom_address = bank_number << 14 | address & 0x3fff;

        if let Some(value) = rom.get(rom_address) {
            Ok(*value)
        } else {
            Err(AddressingError(address))
        }
    }

    fn read_banked_ram(&self, mut address: Address, ram: &[u8]) -> Result<u8, AddressingError> {
        if !self.is_ram_enabled() {
            return Ok(0xff);
        }
        if self.banking_mode_register.as_value() == 0 {
            address = address & 0x1fff
        } else {
            address = ((self.ram_bank_register as usize) << 13) | address & 0x1fff;
            assert!(address <= 0x7fff);
        }
        Ok(ram[address])
    }

    fn is_ram_enabled(&self) -> bool {
        self.ram_enable & 0xf == 0xa
    }
}

impl MemoryBankController for Mbc1 {
    fn read(&self, address: Address, rom: &[u8], ram: &[u8]) -> Result<u8, AddressingError> {
        match address {
            0x0000..=0x3fff => match self.mode {
                Mbc1Mode::Default => Ok(rom[address]),
                Mbc1Mode::Alternative => todo!("alternative mode not implemented"),
            },
            0x0000..=0x7fff => match self.mode {
                Mbc1Mode::Default => self.read_banked_rom(address, rom),
                Mbc1Mode::Alternative => todo!("alternative mode not implemented"),
            },
            0xa000..=0xbfff => match self.mode {
                Mbc1Mode::Default => self.read_banked_ram(address, ram),
                Mbc1Mode::Alternative => todo!("alternative mode not implemented"),
            },
            _ => Err(AddressingError(address)),
        }
    }

    fn write(
        &mut self,
        address: Address,
        mut value: u8,
        _rom: &mut [u8],
        ram: &mut [u8],
    ) -> Result<(), AddressingError> {
        match address {
            0..=0x1fff => {
                self.ram_enable = value;
                Ok(())
            }
            0x2000..=0x3fff => {
                // Write lower 5 bits to bank register 1, first replacing values of 0 with 1 as
                // specified by technical documentation
                value &= 0x1F;
                if value == 0 {
                    value = 1;
                }
                self.bank_register_1.set_range_value(0..=4, value);
                info!("Switched to bank {}", self.bank_number(0x4000));
                Ok(())
            }
            0x4000..=0x5fff => {
                match self.mode {
                    Mbc1Mode::Default => {
                        // use only lower 2 bits
                        value &= 0x3;
                        self.ram_bank_register = value;
                        Ok(())
                    }
                    Mbc1Mode::Alternative => todo!("alternative mode not implemented"),
                }
            }
            // banking mode select
            0x6000..=0x7fff => {
                // Write lowest bit to mode register
                value &= 0x1;
                self.banking_mode_register.set_range_value(0..=0, value);
                Ok(())
            }
            0xa000..=0xbfff => {
                if let Some(entry) = ram.get_mut(address - 0xa000) {
                    *entry = value;
                }
                Ok(())
            }
            _ => panic!("Address {:#x} is out of bounds for rom", address),
        }
    }

    fn get_type(&self) -> MbcType {
        MbcType::Mbc1
    }
}
