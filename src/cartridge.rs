use crate::bit_field::BitField;
use log::*;

pub type Address = usize;

#[derive(Debug)]
pub struct AddressingError(pub Address);

pub struct Cartridge {
    mbc: Box<dyn MemoryBankController + Send>,
    rom: Vec<u8>,
    ram: Vec<u8>,
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
        }
    }
}

impl std::fmt::Debug for Cartridge {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{:?}", self.mbc.get_type())
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

struct Mbc1 {
    ram_gate: BitField<u8>,
    bank_register_1: BitField<u8>,
    bank_register_2: BitField<u8>,
    mode_register: BitField<u8>,
}

impl Default for Mbc1 {
    fn default() -> Self {
        Self {
            ram_gate: BitField::from(0),
            bank_register_1: BitField::from(1),
            bank_register_2: BitField::from(0),
            mode_register: BitField::from(0),
        }
    }
}

impl Mbc1 {
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

    fn read_banked_rom(&self, address: Address, rom: &[u8]) -> Result<u8, AddressingError> {
        let bank_number = self.bank_number(address);
        let rom_address = bank_number << 14 | address & 0x3fff;

        if let Some(value) = rom.get(rom_address) {
            Ok(*value)
        } else {
            Err(AddressingError(address))
        }
    }

    fn read_banked_ram(&self, _address: Address, _ram: &[u8]) -> Result<u8, AddressingError> {
        todo!("Mbc1 ram is not yet implemented")
    }
}

impl MemoryBankController for Mbc1 {
    fn read(&self, address: Address, rom: &[u8], ram: &[u8]) -> Result<u8, AddressingError> {
        match address {
            0x0000..=0x7fff => self.read_banked_rom(address, rom),
            0xa000..=0xbfff => self.read_banked_ram(address, ram),
            _ => Err(AddressingError(address)),
        }
    }

    fn write(
        &mut self,
        address: Address,
        mut value: u8,
        _rom: &mut [u8],
        _ram: &mut [u8],
    ) -> Result<(), AddressingError> {
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
            _ => panic!("Address {:#x} is out of bounds for rom", address),
        }
    }

    fn get_type(&self) -> MbcType {
        MbcType::Mbc1
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CartridgeType {
    mbc_controller_type: MbcType,
    has_ram: bool,
    has_battery: bool,
    has_timer: bool,
    has_rumble: bool,
    rom_size: usize,
    ram_size: usize,
}

impl CartridgeType {
    fn from_data(data: &[u8]) -> Option<Self> {
        debug!("cartridge type byte: {:#x}", data[0x0147]);
        let rom_size = get_rom_size(data);
        let ram_size = get_ram_size(data);
        let cartridge_type = match data[0x0147] {
            0x00 => CartridgeType {
                mbc_controller_type: MbcType::RomOnly,
                has_ram: false,
                has_battery: false,
                has_timer: false,
                has_rumble: false,
                rom_size,
                ram_size,
            },
            0x01 => CartridgeType {
                mbc_controller_type: MbcType::Mbc1,
                has_ram: false,
                has_battery: false,
                has_timer: false,
                has_rumble: false,
                rom_size,
                ram_size,
            },
            0x02 => CartridgeType {
                mbc_controller_type: MbcType::Mbc1,
                has_ram: true,
                has_battery: false,
                has_timer: false,
                has_rumble: false,
                rom_size,
                ram_size,
            },
            0x03 => CartridgeType {
                mbc_controller_type: MbcType::Mbc1,
                has_ram: true,
                has_battery: true,
                has_timer: false,
                has_rumble: false,
                rom_size,
                ram_size,
            },
            _ => {
                warn!("catridge indicated by {:#x} is not supported", data[0x0147]);
                return None;
            }
        };
        Some(cartridge_type)
    }

    fn build(&self, rom_data: &[u8]) -> Cartridge {
        let mbc_controller: Box<dyn MemoryBankController + Send> = match self.mbc_controller_type {
            MbcType::RomOnly => Box::new(NoMbc::default()),
            MbcType::Mbc1 => Box::new(Mbc1::default()),
        };
        let mut rom = vec![0; self.rom_size];
        // Copy provided data into rom. Panics if the provided data exceeds the rom's size.
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
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MbcType {
    RomOnly,
    Mbc1,
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
            mbc: Box::new(Mbc1::default()),
            rom: rom_bytes,
            ram: Vec::new(),
        };

        // Store 0b00100 into bank 1, 0b10 into bank 2, and 0b0 into mode
        cartridge.write(0x2000, 0b00100).unwrap();
        cartridge.write(0x4000, 0b10).unwrap();
        cartridge.write(0x6000, 0).unwrap();

        // Now a read at 0x72a7 should produce the rom value at 0x1132a7, which we set to be 0xff
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
