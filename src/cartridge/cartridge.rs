use crate::register::Register;
use log::{debug, error, info};

pub type Address = usize;

#[derive(Debug)]
pub struct AddressingError(pub Address);

pub trait Cartridge: Send {
    fn mbc_controller_type(&self) -> MBCControllerType;
    fn read(&self, address: Address) -> Result<u8, AddressingError>;
    fn write(&mut self, address: Address, value: u8) -> Result<(), AddressingError>;
}

impl std::fmt::Debug for dyn Cartridge {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{:?}", self.mbc_controller_type())
    }
}

/// Examines cartridge data (the header) to get the size of the rom located
/// on the cartridge.
fn get_rom_size(data: &[u8]) -> usize {
    match data[0x148] {
        0..=8 => 32 * 1024 * (1 << data[0x148]),
        _ => unimplemented!(
            "rom size indicated by value of {:#x} is unsupported",
            data[0x148]
        ),
    }
}

/// Examines cartridge data (the header) to get the size of the ram located
/// on the cartridge.
fn get_ram_size(data: &[u8]) -> usize {
    match data[0x149] {
        0 => 0,
        2 => 8 * 1024,
        3 => 32 * 1024,
        4 => 128 * 1024,
        5 => 64 * 1024,
        _ => unimplemented!(
            "ram size indicated by value of {:#x} is unsupported",
            data[0x149]
        ),
    }
}

/// A Gameboy cartridge that only has a single ROM bank, with no switching.
#[derive(Debug)]
struct RomOnlyCartridge {
    rom: Vec<u8>,
}

impl RomOnlyCartridge {
    fn new(data: &[u8]) -> Self {
        let mut rom = vec![0; 0x8000];
        rom[..data.len()].clone_from_slice(data);
        Self { rom }
    }
}

impl Cartridge for RomOnlyCartridge {
    fn mbc_controller_type(&self) -> MBCControllerType {
        MBCControllerType::RomOnly
    }

    fn read(&self, address: Address) -> Result<u8, AddressingError> {
        if let Some(&value) = self.rom.get(address) {
            Ok(value)
        } else {
            Err(AddressingError(address))
        }
    }

    fn write(&mut self, address: Address, value: u8) -> Result<(), AddressingError> {
        if let Some(elem) = self.rom.get_mut(address) {
            *elem = value;
            Ok(())
        } else {
            Err(AddressingError(address))
        }
    }
}

#[derive(Debug)]
struct MBC1Cartridge {
    ram_gate: Register<u8>,
    bank_register_1: Register<u8>,
    bank_register_2: Register<u8>,
    mode_register: Register<u8>,
    rom: Vec<u8>,
    ram: Vec<u8>,
}

impl MBC1Cartridge {
    fn new(data: &[u8]) -> Self {
        let mut rom = vec![0; get_rom_size(data)];
        rom[..].clone_from_slice(data);
        Self {
            ram_gate: Register::from(0),
            bank_register_1: Register::from(1),
            bank_register_2: Register::from(0),
            mode_register: Register::from(0),
            rom,
            ram: vec![0; get_ram_size(data)],
        }
    }

    fn bank_number(&self, address: Address) -> usize {
        match address {
            0..=0x3fff if self.mode_register.as_value() == 0 => 0,
            0..=0x3fff if self.mode_register.as_value() != 0 => {
                (self.bank_register_2.as_value() << 5).into()
            }
            0x4000..=0x7fff => {
                (self.bank_register_2.as_value() << 5 | self.bank_register_1.as_value()).into()
            }
            _ => panic!(),
        }
    }
}

impl Cartridge for MBC1Cartridge {
    fn mbc_controller_type(&self) -> MBCControllerType {
        MBCControllerType::MBC1
    }

    fn read(&self, address: Address) -> Result<u8, AddressingError> {
        let bank_number = self.bank_number(address);
        let rom_address = bank_number << 14 | address & 0x3fff;
        match self.rom.get(rom_address) {
            Some(value) => Ok(*value),
            None => {
                error!("rom address {:#x} is out of bounds", rom_address);
                Err(AddressingError(address))
            }
        }
    }

    fn write(&mut self, address: Address, mut value: u8) -> Result<(), AddressingError> {
        match address {
            0..=0x1fff => {
                // Write lower 4 bits to ram gate register
                value &= 0xF;
                self.ram_gate.set_range_value(0..=3, value);
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
                // Write lower 2 bits to bank register 2
                value &= 0x3;
                self.bank_register_2.set_range_value(0..=1, value);
                info!("Switched to bank {}", self.bank_number(0x4000));
                Ok(())
            }
            0x6000..=0x7fff => {
                // Write lowest bit to mode register
                value &= 0x1;
                self.mode_register.set_range_value(0..=0, value);
                Ok(())
            }
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CartridgeType {
    mbc_controller_type: MBCControllerType,
    ram: bool,
    battery: bool,
    timer: bool,
    rumble: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MBCControllerType {
    RomOnly,
    MBC1,
}

pub fn build_cartridge(data: &[u8]) -> Option<Box<dyn Cartridge>> {
    match cartridge_type_from_data(data) {
        Some(CartridgeType {
            mbc_controller_type: MBCControllerType::RomOnly,
            ..
        }) => Some(Box::new(RomOnlyCartridge::new(data))),
        Some(CartridgeType {
            mbc_controller_type: MBCControllerType::MBC1,
            ..
        }) => Some(Box::new(MBC1Cartridge::new(data))),
        _ => None,
    }
}

fn cartridge_type_from_data(data: &[u8]) -> Option<CartridgeType> {
    debug!("{}", data[0x0147]);
    let cartridge_type = match data[0x0147] {
        0x00 => CartridgeType {
            mbc_controller_type: MBCControllerType::RomOnly,
            ram: false,
            battery: false,
            timer: false,
            rumble: false,
        },
        0x01 => CartridgeType {
            mbc_controller_type: MBCControllerType::MBC1,
            ram: false,
            battery: false,
            timer: false,
            rumble: false,
        },
        0x02 => CartridgeType {
            mbc_controller_type: MBCControllerType::MBC1,
            ram: true,
            battery: false,
            timer: false,
            rumble: false,
        },
        0x03 => CartridgeType {
            mbc_controller_type: MBCControllerType::MBC1,
            ram: true,
            battery: true,
            timer: false,
            rumble: false,
        },
        _ => unimplemented!("catridge indicated by {:#x} is not supported", data[0x0147]),
    };

    Some(cartridge_type)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mbc1_memory_banks_swap() {
        let mut bytes = vec![0; 0x200000];
        bytes[0x1132a7] = 0xff;
        let mut cartridge = MBC1Cartridge::new(&bytes);

        // Store 0b00100 into bank 1, 0b10 into bank 2, and 0b0 into mode
        cartridge.write(0x2000, 0b00100).unwrap();
        cartridge.write(0x4000, 0b10).unwrap();
        cartridge.write(0x6000, 0).unwrap();

        // Now a read at 0x72a7 should produce the rom value at 0x1132a7, which we set to be 0xff
        assert_eq!(0xff, cartridge.read(0x72a7).unwrap());
    }

    #[test]
    fn test_cartridge_builder() {
        let bytes = [0; 32_000];
        assert_eq!(
            MBCControllerType::RomOnly,
            build_cartridge(&bytes).unwrap().mbc_controller_type()
        );

        let mut bytes = vec![0; 128 * 0x4000];
        bytes[0x0147] = 1;
        assert_eq!(
            MBCControllerType::MBC1,
            build_cartridge(&bytes).unwrap().mbc_controller_type()
        );
    }

    #[test]
    #[should_panic]
    fn test_cartridge_builder_panics_with_large_data() {
        let mut bytes = vec![0; 128 * 0x4000 + 1];
        bytes[0x0147] = 1;
        build_cartridge(&bytes);
    }
}
