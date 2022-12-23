use crate::gameboy::Interrupt;
use crate::cartridge::{Cartridge, self};
use log::debug;

/// Mock memory bus
#[derive(Debug)]
pub struct MemoryBus {
    cartridge: Option<Box<dyn Cartridge>>,
    pub data: [u8; 0x10000],
    pub output_string: String,
}

impl MemoryBus {
    pub fn new() -> Self {
        Self {
            cartridge: None,
            data: [0; 0x10000],
            output_string: String::new(),
        }
    }

    pub fn get(&self, address: usize) -> u8 {
        match address {
            0..=0x7fff => {
                let cartridge = self.cartridge.as_ref().expect("No cartridge inserted");
                cartridge.read(address).expect("Error reading cartridge")
            }
            _ => self.data[address]
        }
    }

    pub fn set(&mut self, address: usize, value: u8) {
        if address == 0xFF02 && value == 0x81 {
            let chr = char::from_u32(self.data[0xFF01] as u32).unwrap();
            self.output_string.push(chr);
        }
        match address {
            0..=0x7fff => {
                let cartridge = self.cartridge.as_mut().expect("No cartridge inserted");
                cartridge.write(address, value).expect("Error reading cartridge")
            },
            _ => self.data[address] = value,
        }
    }

    pub fn interrupt(&mut self, interrupt: Interrupt) {
        debug!("Interrupting");
        let bit = match interrupt {
            Interrupt::Timer => 2,
        };
        let mut interrupt_flag = self.get(0xFF0F);
        interrupt_flag |= 1 << bit;
        self.set(0xFF0F, interrupt_flag);
    }

    pub fn insert_cartridge(&mut self, cartridge: Box<dyn Cartridge>) {
        self.cartridge = Some(cartridge);
    }

    pub fn remove_cartridge(&mut self) -> Option<Box<dyn Cartridge>> {
        self.cartridge.take()
    }
}
