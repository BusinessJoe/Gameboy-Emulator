/*!
 * The memory bus holds ownership of the ppu and cartridge.
 * This structure makes it easy to delegate reads/writes to the corresponding memory-mapped component.
 */
use std::cell::RefCell;
use std::rc::Rc;

use crate::apu::Apu;
use crate::cartridge::Cartridge;
use crate::component::{Address, Addressable};
use crate::emulator::events::EmulationEvent;
use crate::error::Result;
use crate::gameboy::Interrupt;
use crate::joypad::Joypad;
use crate::ppu::BasePpu;
use crate::timer::Timer;
use log::debug;

/// Mock memory bus
pub struct MemoryBus {
    pub(crate) cartridge: Option<Cartridge>,
    ppu: Rc<RefCell<BasePpu>>,
    apu: Rc<RefCell<Apu>>,
    joypad: Rc<RefCell<Joypad>>,
    timer: Rc<RefCell<Timer>>,
    pub data: [u8; 0x10000],
    pub serial_port_data: Vec<u8>,
}

impl MemoryBus {
    pub fn new(
        ppu: Rc<RefCell<BasePpu>>,
        apu: Rc<RefCell<Apu>>,
        joypad: Rc<RefCell<Joypad>>,
        timer: Rc<RefCell<Timer>>,
    ) -> Self {
        let memory_bus = Self {
            cartridge: None,
            ppu,
            apu,
            joypad,
            timer,
            data: [0; 0x10000],
            serial_port_data: Vec::new(),
        };

        memory_bus
    }

    pub fn get_serial_port_data(&self) -> &[u8] {
        &self.serial_port_data
    }

    pub fn emulation_event(&self, event: EmulationEvent) {
        // if let Some(sender) = &self.emulation_event_sender {
        //     sender.send(event).unwrap();
        // }
    }

    // Initiate an OAM transfer
    fn oam_transfer(&mut self, value: u8) -> Result<()> {
        let read_base_address = usize::from(value) * 0x100;
        let mut data = vec![0; 0xa0];

        for i in 0..0xa0 {
            data[i] = self.read_u8(read_base_address + i)?;
        }

        self.ppu.borrow_mut().oam_transfer(&data);

        // for i in 0 .. 0xa0 {
        //     let data = self.read_u8(read_base_address + i)?;
        //     self.ppu.borrow_mut().
        //     self.write_u8(0xfe00 + i, data)?;
        // }
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
    fn read_u8(&mut self, address: Address) -> Result<u8> {
        let byte = match address {
            0..=0x7fff => {
                let cartridge = self.cartridge.as_ref().expect("No cartridge inserted");
                let value = cartridge.read(address).expect("Error reading cartridge");
                Ok(value)
            }
            0x8000..=0x9fff => self.ppu.borrow_mut().read_u8(address),
            // Cartridge RAM
            0xa000..=0xbfff => {
                let cartridge = self.cartridge.as_ref().expect("No cartridge inserted");
                let value = cartridge.read(address).expect("Error reading cartridge");
                Ok(value)
            }
            // OAM
            0xfe00..=0xfe9f => self.ppu.borrow_mut().read_u8(address),
            // Joypad
            0xff00 => Ok(self.joypad.borrow().read()),
            // Timer
            0xff04..=0xff07 => self.timer.borrow_mut().read_u8(address),
            // IF register always has top 3 bits high
            0xff0f => Ok(self.data[address] | 0xe0),
            0xff10..=0xff3f => self.apu.borrow_mut().read_u8(address),
            // PPU mappings
            0xff40..=0xff45 => self.ppu.borrow_mut().read_u8(address),
            0xff47..=0xff4b => self.ppu.borrow_mut().read_u8(address),
            0xff4d => Ok(0xff),
            _ => match self.data.get(address) {
                Some(byte) => Ok(*byte),
                None => Ok(0xff),
            },
        };

        debug_assert!(byte.is_ok());

        byte
    }

    fn write_u8(&mut self, address: Address, value: u8) -> Result<()> {
        if address == 0xFF02 && value == 0x81 {
            self.serial_port_data.push(self.data[0xFF01]);
        }

        match address {
            0..=0x7fff => {
                let cartridge = self.cartridge.as_mut().expect("No cartridge inserted");
                cartridge
                    .write(address, value)
                    .expect("Error reading cartridge");
            }
            0x8000..=0x9fff => self.ppu.borrow_mut().write_u8(address, value)?,
            // Cartridge RAM
            0xa000..=0xbfff => {
                let cartridge = self.cartridge.as_mut().expect("No cartridge inserted");
                cartridge
                    .write(address, value)
                    .expect("Error reading cartridge");
            }
            // OAM
            0xfe00..=0xfe9f => self.ppu.borrow_mut().write_u8(address, value)?,
            // Joypad
            0xff00 => self.joypad.borrow_mut().write(value),
            // Timer
            0xff04..=0xff07 => self.timer.borrow_mut().write_u8(address, value)?,
            0xff10..=0xff3f => self.apu.borrow_mut().write_u8(address, value)?,
            // PPU mappings
            0xff40..=0xff44 => self.ppu.borrow_mut().write_u8(address, value)?,
            // lyc write, we need to check if that triggers a stat interrupt
            0xff45 => {
                let result = {
                    let mut ppu = self.ppu.borrow_mut();
                    ppu.write_u8(address, value)?;
                    ppu.state.lcd.check_ly_equals_lyc()
                };
                if let Some(interrupt) = result {
                    self.interrupt(interrupt)?;
                }
            }
            0xff47..=0xff4b => self.ppu.borrow_mut().write_u8(address, value)?,
            0xff46 => self.oam_transfer(value)?,
            // Write to VRAM tile data
            _ => match self.data.get_mut(address) {
                Some(entry) => *entry = value,
                None => (),
            },
        }

        Ok(())
    }
}
