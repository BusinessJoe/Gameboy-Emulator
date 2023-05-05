/*!
 * The memory bus holds ownership of the non-cpu components of the gameboy.
 * This structure makes it easy to delegate reads/writes to the corresponding memory-mapped component.
 */

use crate::apu::Apu;
use crate::cartridge::Cartridge;
use crate::component::{Address, Addressable, BatchSteppable, Steppable, TickCount};
use crate::error::Result;
use crate::interrupt::InterruptRegs;
use crate::joypad::Joypad;
use crate::ppu::BasePpu;
use crate::timer::Timer;

/// Mock memory bus
pub struct MemoryBus {
    pub(crate) cartridge: Option<Cartridge>,
    pub(crate) ppu: BasePpu,
    pub(crate) apu: Apu,
    pub(crate) joypad: Joypad,
    pub(crate) timer: Timer,
    pub(crate) interrupt_regs: InterruptRegs,
    pub data: [u8; 0x10000],
    pub serial_port_data: Vec<u8>,

    tick_count: TickCount,
}

impl MemoryBus {
    pub fn new(ppu: BasePpu, apu: Apu, joypad: Joypad, timer: Timer) -> Self {
        let memory_bus = Self {
            cartridge: None,
            ppu,
            apu,
            joypad,
            timer,
            interrupt_regs: InterruptRegs::new(),
            data: [0; 0x10000],
            serial_port_data: Vec::new(),
            tick_count: 0,
        };

        memory_bus
    }

    pub fn increment_tick_counter(&mut self, increment: u128) {
        self.tick_count += increment;
    }

    pub fn fast_forward_timer(&mut self) -> Result<()> {
        self.timer
            .fast_forward(&mut self.interrupt_regs, self.tick_count)
    }

    pub fn get_serial_port_data(&self) -> &[u8] {
        &self.serial_port_data
    }

    // Initiate an OAM transfer
    fn oam_transfer(&mut self, value: u8) -> Result<()> {
        let read_base_address = usize::from(value) * 0x100;
        let mut data = vec![0; 0xa0];

        for i in 0..0xa0 {
            data[i] = self.read_u8(read_base_address + i)?;
        }

        self.ppu.oam_transfer(&data);

        // for i in 0 .. 0xa0 {
        //     let data = self.read_u8(read_base_address + i)?;
        //     self.ppu.borrow_mut().
        //     self.write_u8(0xfe00 + i, data)?;
        // }
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
            0x8000..=0x9fff => self.ppu.read_u8(address),
            // Cartridge RAM
            0xa000..=0xbfff => {
                let cartridge = self.cartridge.as_ref().expect("No cartridge inserted");
                let value = cartridge.read(address).expect("Error reading cartridge");
                Ok(value)
            }
            // OAM
            0xfe00..=0xfe9f => self.ppu.read_u8(address),
            // Joypad
            0xff00 => Ok(self.joypad.read()),
            // Timer
            0xff04..=0xff07 => {
                self.timer
                    .fast_forward(&mut self.interrupt_regs, self.tick_count)?;
                self.timer.read_u8(address)
            }
            // IF register always has top 3 bits high
            0xff0f => Ok(self.interrupt_regs.interrupt_flag | 0xe0),
            0xff10..=0xff3f => self.apu.read_u8(address),
            // PPU mappings
            0xff40..=0xff45 => self.ppu.read_u8(address),
            0xff47..=0xff4b => self.ppu.read_u8(address),
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
            0x8000..=0x9fff => self.ppu.write_u8(address, value)?,
            // Cartridge RAM
            0xa000..=0xbfff => {
                let cartridge = self.cartridge.as_mut().expect("No cartridge inserted");
                cartridge
                    .write(address, value)
                    .expect("Error reading cartridge");
            }
            // OAM
            0xfe00..=0xfe9f => self.ppu.write_u8(address, value)?,
            // Joypad
            0xff00 => self.joypad.write(value),
            // Timer
            0xff04..=0xff07 => {
                self.timer
                    .fast_forward(&mut self.interrupt_regs, self.tick_count)?;
                self.timer.write_u8(address, value)?
            }
            0xff0f => self.interrupt_regs.interrupt_flag = value,
            0xff10..=0xff3f => self.apu.write_u8(address, value)?,
            // PPU mappings
            0xff40..=0xff44 => self.ppu.write_u8(address, value)?,
            // lyc write, we need to check if that triggers a stat interrupt
            0xff45 => {
                let result = {
                    self.ppu.write_u8(address, value)?;
                    self.ppu.state.lcd.check_ly_equals_lyc()
                };
                if let Some(interrupt) = result {
                    self.interrupt_regs.interrupt(interrupt);
                }
            }
            0xff47..=0xff4b => self.ppu.write_u8(address, value)?,
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
