pub mod mbc1;
pub mod mbc3;
use std::fmt::Display;

use log::*;

use self::{
    mbc1::{Mbc1, Mbc1Mode},
    mbc3::Mbc3,
};

pub type Address = usize;

#[derive(Debug)]
pub struct AddressingError(pub Address);

pub struct Cartridge {
    mbc: Box<dyn MemoryBankController + Send>,
    rom: Vec<u8>,
    pub(crate) ram: Vec<u8>,
    pub(crate) cartridge_type: Option<CartridgeType>,
}

impl Cartridge {
    pub fn read(&self, address: Address) -> Result<u8, AddressingError> {
        self.mbc.read(address, &self.rom, &self.ram)
    }
    pub fn write(&mut self, address: Address, value: u8) -> Result<(), AddressingError> {
        self.mbc.write(address, value, &mut self.rom, &mut self.ram)
    }
    pub fn cartridge_from_data(data: &[u8]) -> Option<Cartridge> {
        let cartridge_type = CartridgeType::from_data(data)?;
        Some(cartridge_type.build(data))
    }
    pub fn mock() -> Self {
        Cartridge {
            mbc: Box::new(NoMbc {}),
            rom: vec![0; 0x8000],
            ram: vec![0; 0x8000],
            cartridge_type: None,
        }
    }
}

impl std::fmt::Debug for Cartridge {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{:?}", self.mbc.get_type())
    }
}

impl Display for Cartridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "ram size: {}", self.ram.len())
    }
}

trait MemoryBankController {
    fn read(&self, address: Address, rom: &[u8], ram: &[u8]) -> Result<u8, AddressingError>;
    fn write(
        &mut self,
        address: Address,
        value: u8,
        rom: &mut [u8],
        ram: &mut [u8],
    ) -> Result<(), AddressingError>;
    fn get_type(&self) -> MbcType;
}

/// Examines cartridge data (the header) to get the size of the rom located
/// on the cartridge.
fn get_rom_size(data: &[u8]) -> usize {
    dbg!(data[0x148]);
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
    dbg!(data[0x149]);
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

/// Reads bytes 0x134 ..= 0x143 into a string
fn get_title(data: &[u8]) -> String {
    String::from_utf8(data[0x134..=0x143].to_vec()).unwrap_or_else(|_| "Unknown".to_string())
}

/// A Gameboy cartridge that only has a single ROM bank, with no switching.
#[derive(Default)]
struct NoMbc {}
impl MemoryBankController for NoMbc {
    fn read(&self, address: Address, rom: &[u8], ram: &[u8]) -> Result<u8, AddressingError> {
        rom.get(address).ok_or(AddressingError(address)).copied()
    }

    fn write(
        &mut self,
        address: Address,
        value: u8,
        rom: &mut [u8],
        ram: &mut [u8],
    ) -> Result<(), AddressingError> {
        if let Some(elem) = rom.get_mut(address) {
            // ignore writes to rom
            Ok(())
        } else {
            Err(AddressingError(address))
        }
    }

    fn get_type(&self) -> MbcType {
        MbcType::RomOnly
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CartridgeType {
    mbc_controller_type: MbcType,
    pub has_ram: bool,
    pub has_battery: bool,
    pub has_timer: bool,
    pub has_rumble: bool,
    pub rom_size: usize,
    pub ram_size: usize,
    pub title: String,
}

impl CartridgeType {
    fn from_data(data: &[u8]) -> Option<Self> {
        debug!("cartridge type byte: {:#x}", data[0x0147]);
        let rom_size = get_rom_size(data);
        let ram_size = get_ram_size(data);
        println!("rom size: {:#x} bytes", rom_size);
        println!("ram size: {:#x} bytes", ram_size);
        let title = get_title(data);
        let cartridge_type = match data[0x0147] {
            0x00 => CartridgeType {
                mbc_controller_type: MbcType::RomOnly,
                has_ram: false,
                has_battery: false,
                has_timer: false,
                has_rumble: false,
                rom_size,
                ram_size,
                title,
            },
            0x01 => CartridgeType {
                mbc_controller_type: MbcType::Mbc1,
                has_ram: false,
                has_battery: false,
                has_timer: false,
                has_rumble: false,
                rom_size,
                ram_size,
                title,
            },
            0x02 => CartridgeType {
                mbc_controller_type: MbcType::Mbc1,
                has_ram: true,
                has_battery: false,
                has_timer: false,
                has_rumble: false,
                rom_size,
                ram_size,
                title,
            },
            0x03 => CartridgeType {
                mbc_controller_type: MbcType::Mbc1,
                has_ram: true,
                has_battery: true,
                has_timer: false,
                has_rumble: false,
                rom_size,
                ram_size,
                title,
            },
            0x0f => CartridgeType {
                mbc_controller_type: MbcType::Mbc3,
                has_ram: false,
                has_battery: true,
                has_timer: true,
                has_rumble: false,
                rom_size,
                ram_size,
                title,
            },
            0x10 => CartridgeType {
                mbc_controller_type: MbcType::Mbc3,
                has_ram: true,
                has_battery: true,
                has_timer: true,
                has_rumble: false,
                rom_size,
                ram_size,
                title,
            },
            0x11 => CartridgeType {
                mbc_controller_type: MbcType::Mbc3,
                has_ram: false,
                has_battery: false,
                has_timer: false,
                has_rumble: false,
                rom_size,
                ram_size,
                title,
            },
            0x12 => CartridgeType {
                mbc_controller_type: MbcType::Mbc3,
                has_ram: true,
                has_battery: false,
                has_timer: false,
                has_rumble: false,
                rom_size,
                ram_size,
                title,
            },
            0x13 => CartridgeType {
                mbc_controller_type: MbcType::Mbc3,
                has_ram: true,
                has_battery: true,
                has_timer: false,
                has_rumble: false,
                rom_size,
                ram_size,
                title,
            },
            _ => {
                eprintln!("catridge indicated by {:#x} is not supported", data[0x0147]);
                return None;
            }
        };
        Some(cartridge_type)
    }

    fn build(&self, rom_data: &[u8]) -> Cartridge {
        let mbc_controller: Box<dyn MemoryBankController + Send> = match self.mbc_controller_type {
            MbcType::RomOnly => Box::new(NoMbc::default()),
            MbcType::Mbc1 => {
                if self.rom_size <= 512 * 1024 && self.ram_size <= 32 * 1024 {
                    Box::new(Mbc1::new(Mbc1Mode::Default))
                } else if self.rom_size > 512 * 1024
                    && self.rom_size < 8 * 1024 * 1024
                    && self.ram_size <= 8 * 1024
                {
                    Box::new(Mbc1::new(Mbc1Mode::Alternative))
                } else {
                    panic!("invalid rom/ram sizes")
                }
            }
            MbcType::Mbc3 => Box::new(Mbc3::new()),
        };
        let mut rom = vec![0; self.rom_size];
        // Copy provided data into rom. Panics if the provided data exceeds the rom's size.
        println!("loading {:#x} bytes into rom", rom_data.len());
        rom[0..rom_data.len()].copy_from_slice(rom_data);
        let ram = vec![0; self.ram_size];

        if self.has_battery {
            warn!(
                "batteries not included (cartridge requires a battery which isn't implemented yet)"
            );
        }
        if self.has_timer {
            warn!("cartridge requires a timer which isn't implemented yet");
        }
        if self.has_rumble {
            warn!("cartridge requires rumble which isn't implemented yet");
        }

        Cartridge {
            mbc: mbc_controller,
            rom,
            ram,
            cartridge_type: Some(self.clone()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MbcType {
    RomOnly,
    Mbc1,
    Mbc3,
}

fn cartridge_from_data(data: &[u8]) -> Option<Cartridge> {
    let cartridge_type = CartridgeType::from_data(data)?;
    Some(cartridge_type.build(data))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mbc1_memory_banks_swap() {
        let mut rom_bytes = vec![0; 0x200000];
        rom_bytes[0x1132a7] = 0xfe;
        let mut cartridge = Cartridge {
            mbc: Box::new(Mbc1::new(Mbc1Mode::Default)),
            rom: rom_bytes,
            ram: Vec::new(),
            cartridge_type: None,
        };

        // Store 0b00100 into bank 1, 0b10 into bank 2, and 0b0 into mode
        cartridge.write(0x2000, 0b00100).unwrap();
        cartridge.write(0x4000, 0b10).unwrap();
        cartridge.write(0x6000, 0).unwrap();

        // Now a read at 0x72a7 should produce the rom value at 0x1132a7, which we set to be 0xfe
        assert_eq!(0xfe, cartridge.read(0x72a7).unwrap());
    }

    #[test]
    fn test_cartridge_builder_correct_mbc_type() {
        let bytes = [0; 32_000];
        assert_eq!(
            MbcType::RomOnly,
            cartridge_from_data(&bytes).unwrap().mbc.get_type()
        );

        let mut bytes = vec![0; 128 * 0x4000];
        bytes[0x0147] = 1;
        bytes[0x0148] = 0x6;
        assert_eq!(
            MbcType::Mbc1,
            cartridge_from_data(&bytes).unwrap().mbc.get_type()
        );
    }

    #[test]
    #[should_panic]
    fn test_cartridge_builder_panics_with_large_data() {
        let mut bytes = vec![0; 128 * 0x4000 + 1];
        bytes[0x0147] = 1;
        cartridge_from_data(&bytes);
    }
}
