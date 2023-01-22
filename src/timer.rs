use crate::cartridge::AddressingError;
use crate::component::{Address, Addressable, Steppable};
use crate::error::Error;
use crate::gameboy::Interrupt;
use crate::memory::MemoryBus;
use log::info;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Timer {
    /// Number of clock cycles per second.
    clock_speed: u64,
    div_clocksum: u64,
    timer_clocksum: u64,

    // Timer registers
    div: u8,
    tima: u8,
    tma: u8,
    tac: u8,
}

// Divider register
const DIV: usize = 0xFF04;
// Timer counter
const TIMA: usize = 0xFF05;
// Timer modulo
const TMA: usize = 0xFF06;
// Timer control
const TAC: usize = 0xFF07;

impl Timer {
    pub fn new(clock_speed: u64) -> Self {
        Self {
            clock_speed,
            div_clocksum: 0,
            timer_clocksum: 0,

            div: 0,
            tima: 0,
            tma: 0,
            tac: 0,
        }
    }

    fn is_enabled(&self) -> bool {
        (self.tac >> 2) & 1 == 1
    }

    fn get_frequency(&self) -> u64 {
        let bits = self.tac & 0b11;
        match bits {
            0b00 => 4_096,
            0b01 => 262_144,
            0b10 => 65_536,
            0b11 => 16_384,
            _ => panic!(),
        }
    }

    fn _read(&mut self, address: Address) -> crate::error::Result<u8> {
        let value = match address {
            DIV => self.div,
            TIMA => self.tima,
            TMA => self.tma,
            TAC => self.tac,
            _ => return Err(Error::new("invalid address"))
        };
        Ok(value)
    }

    fn _write(&mut self, address: Address, value: u8) -> crate::error::Result<()> {
        match address {
            DIV => self.div = value,
            TIMA => self.tima = value,
            TMA => self.tma = value,
            TAC => self.tac = value,
            _ => return Err(Error::new("invalid address"))
        }
        Ok(())
    }
}

impl Addressable for Timer {
    fn read(&mut self, address: Address, data: &mut [u8]) -> crate::error::Result<()> {
        for (offset, byte) in data.iter_mut().enumerate() {
            *byte = self._read(address + offset)?;
        }

        Ok(())
    }

    fn write(&mut self, address: Address, data: &[u8]) -> crate::error::Result<()> {
        for (offset, byte) in data.iter().enumerate() {
            self._write(address + offset, *byte)?;
        }

        Ok(())
    }
}

impl Steppable for Timer {
    fn step(&mut self, state: &crate::gameboy::GameBoyState) -> crate::error::Result<crate::component::ElapsedTime> {
        // Manage divider timer
        self.div_clocksum += 1;
        if self.div_clocksum >= 256 {
            self.div_clocksum -= 256;
            self.div = self.div.wrapping_add(1);
        }

        if self.is_enabled() {
            self.timer_clocksum += 4;

            let freq = self.get_frequency();

            while self.timer_clocksum >= self.clock_speed / freq {
                // Increment TIMA
                self.tima = self.tima.wrapping_add(1);

                // When TIMA overflows, send an interrupt and reset TIMA to TMA
                if self.tima == 0x00 {
                    info!("Sending timer interrupt");
                    state
                        .memory_bus
                        .borrow_mut()
                        .interrupt(Interrupt::Timer)?;
                    self.tima = self.tma;
                }
                self.timer_clocksum -= self.clock_speed / freq;
            }
        }

        Ok(1)
    }
}
