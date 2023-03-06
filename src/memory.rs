/*!
 * The memory bus holds ownership of the ppu and cartridge.
 * This structure makes it easy to delegate reads/writes to the corresponding memory-mapped component.
 */
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::Sender;

use crate::cartridge::Cartridge;
use crate::component::{Address, Addressable};
use crate::emulator::events::EmulationEvent;
use crate::error::Result;
use crate::gameboy::Interrupt;
use crate::joypad::Joypad;
use crate::ppu::Ppu;
use crate::timer::Timer;
use log::debug;

/// Mock memory bus
pub struct MemoryBus {
    cartridge: Option<Cartridge>,
    ppu: Rc<RefCell<dyn Ppu>>,
    joypad: Rc<RefCell<Joypad>>,
    timer: Rc<RefCell<Timer>>,
    pub data: [u8; 0x10000],
    pub serial_port_data: Vec<u8>,
    emulation_event_sender: Sender<EmulationEvent>
}

impl MemoryBus {
    pub fn new(
        ppu: Rc<RefCell<dyn Ppu>>,
        joypad: Rc<RefCell<Joypad>>,
        timer: Rc<RefCell<Timer>>,
        emulation_event_sender: Sender<EmulationEvent>
    ) -> Self {
        let memory_bus = Self {
            cartridge: None,
            ppu,
            joypad,
            timer,
            data: [0; 0x10000],
            serial_port_data: Vec::new(),
            emulation_event_sender,
        };

        memory_bus
    }

    fn _read(&mut self, address: Address) -> Result<u8> {
        if address == 0x8ce0 {
            println!("Reading correct tile");
        }

        match address {
            0..=0x7fff => {
                let cartridge = self.cartridge.as_ref().expect("No cartridge inserted");
                let value = cartridge.read(address).expect("Error reading cartridge");
                Ok(value)
            }
            0x8000..=0x97ff => self.ppu.borrow_mut().read_u8(address),
            0x9800..=0x9bff => self.ppu.borrow_mut().read_u8(address),
            // OAM
            0xfe00..=0xfe9f => self.ppu.borrow_mut().read_u8(address),
            // Joypad
            0xff00 => self.joypad.borrow_mut().read_u8(address),
            // Timer
            0xff04..=0xff07 => self.timer.borrow_mut().read_u8(address),
            // IF register always has top 3 bits high
            0xff0f => Ok(self.data[address] | 0xe0),
            // LCD Control register (LCDC)
            0xff40 => self.ppu.borrow_mut().read_u8(address),
            0xff44 => self.ppu.borrow_mut().read_u8(address),
            0xff4d => Ok(0xff),
            _ => Ok(self.data[address]),
        }
    }

    fn _write(&mut self, address: Address, value: u8) -> Result<()> {
        if address == 0xFF02 && value == 0x81 {
            self.serial_port_data.push(self.data[0xFF01]);
        }

        if address == 0x8000 {
            self.emulation_event(EmulationEvent::MemoryWrite { address: address, value: value });
        }

        match address {
            0..=0x7fff => {
                let cartridge = self.cartridge.as_mut().expect("No cartridge inserted");
                cartridge
                    .write(address, value)
                    .expect("Error reading cartridge");
            }
            0x8000..=0x97ff => self.ppu.borrow_mut().write_u8(address, value)?,
            0x9800..=0x9bff => self.ppu.borrow_mut().write_u8(address, value)?,
            // OAM
            0xfe00..=0xfe9f => self.ppu.borrow_mut().write_u8(address, value)?,
            // Joypad
            0xff00 => self.joypad.borrow_mut().write_u8(address, value)?,
            // Timer
            0xff04..=0xff07 => self.timer.borrow_mut().write_u8(address, value)?,
            // LCD Control register (LCDC)
            0xff40 => self.ppu.borrow_mut().write_u8(address, value)?,
            0xff46 => self.oam_transfer(value)?,
            // Write to VRAM tile data
            _ => self.data[address] = value,
        }

        Ok(())
    }
    
    pub fn emulation_event(&self, event: EmulationEvent) {
        self.emulation_event_sender.send(event);
    }

    // Initiate an OAM transfer
    fn oam_transfer(&mut self, value: u8) -> Result<()> {
        let mut data = vec![0; 0xa0];
        self.read(usize::from(value) * 0x100, &mut data)?;
        self.write(0xfe00, &data)?;
        Ok(())
    }

    pub fn interrupt(&mut self, interrupt: Interrupt) -> Result<()> {
        debug!("Interrupting");
        let bit = match interrupt {
            Interrupt::VBlank => 0,
            Interrupt::Stat => 1,
            Interrupt::Timer => 2,
            Interrupt::Joypad => 4,
        };
        let mut interrupt_flag = self.read_u8(0xFF0F)?;
        interrupt_flag |= 1 << bit;
        self.write_u8(0xFF0F, interrupt_flag)?;
        Ok(())
    }

    pub fn insert_cartridge(&mut self, cartridge: Cartridge) {
        self.cartridge = Some(cartridge);
    }

    pub fn remove_cartridge(&mut self) -> Option<Cartridge> {
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
