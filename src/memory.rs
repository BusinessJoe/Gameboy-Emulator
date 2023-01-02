/*!
 * The memory bus holds ownership of the ppu and cartridge.
 * This structure makes it easy to delegate reads/writes to the corresponding memory-mapped component.
 */
use std::sync::{Arc, Mutex, Weak};

use crate::cartridge::{self, Cartridge};
use crate::component::{Address, Addressable};
use crate::cpu::CPU;
use crate::error::Result;
use crate::gameboy::Interrupt;
use crate::ppu::PPU;
use log::debug;

/// Mock memory bus
#[derive(Debug)]
pub struct MemoryBus {
    cartridge: Option<Box<dyn Cartridge>>,
    ppu: Arc<Mutex<PPU>>,
    pub data: [u8; 0x10000],
    pub serial_port_data: Vec<char>,
}

impl MemoryBus {
    pub fn new() -> Self {
        let memory_bus = Self {
            cartridge: None,
            ppu: Arc::new(Mutex::new(PPU::new())),
            data: [0; 0x10000],
            serial_port_data: Vec::new(),
        };

        memory_bus
    }

    pub fn upgrade(memory_bus: Weak<Mutex<MemoryBus>>) -> Arc<Mutex<MemoryBus>> {
        memory_bus.upgrade().expect("invalid memory bus reference")
    }

    fn _read(&mut self, address: Address) -> Result<u8> {
        let value = match address {
            0..=0x7fff => {
                let cartridge = self.cartridge.as_ref().expect("No cartridge inserted");
                cartridge.read(address).expect("Error reading cartridge")
            }
            _ => self.data[address],
        };

        Ok(value)
    }

    fn _write(&mut self, address: Address, value: u8) -> Result<()> {
        if address == 0xFF02 && value == 0x81 {
            let chr = char::from_u32(self.data[0xFF01] as u32).unwrap();
            self.serial_port_data.push(chr);
        }
        match address {
            0..=0x7fff => {
                let cartridge = self.cartridge.as_mut().expect("No cartridge inserted");
                cartridge
                    .write(address, value)
                    .expect("Error reading cartridge");
            }
            _ => self.data[address] = value,
        }

        Ok(())
    }

    pub fn interrupt(&mut self, interrupt: Interrupt) -> Result<()> {
        debug!("Interrupting");
        let bit = match interrupt {
            Interrupt::Timer => 2,
        };
        let mut interrupt_flag = self.read_u8(0xFF0F)?;
        interrupt_flag |= 1 << bit;
        self.write_u8(0xFF0F, interrupt_flag)?;
        Ok(())
    }

    pub fn insert_cartridge(&mut self, cartridge: Box<dyn Cartridge>) {
        self.cartridge = Some(cartridge);
    }

    pub fn remove_cartridge(&mut self) -> Option<Box<dyn Cartridge>> {
        self.cartridge.take()
    }
}

impl Addressable for MemoryBus {
    fn read(&mut self, address: Address, data: &mut [u8]) -> Result<()> {
        for (offset, byte) in data.iter_mut().enumerate() {
            *byte = self._read(address + offset)?;
        }

        Ok(())
    }

    fn write(&mut self, address: Address, data: &[u8]) -> Result<()> {
        for (offset, byte) in data.iter().enumerate() {
            self._write(address + offset, *byte)?;
        }

        Ok(())
    }
}
