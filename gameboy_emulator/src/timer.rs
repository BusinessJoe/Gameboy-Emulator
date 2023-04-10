use crate::component::{Address, Addressable, Steppable};
use crate::error::Error;
use crate::gameboy::Interrupt;
use log::info;

pub struct Timer {
    /// Number of clock cycles per second.
    div_clocksum: u64,
    timer_clocksum: u64,

    // Timer registers
    div: u8,
    tima: u8,
    tma: u8,
    tac: u8,
}

// Divider register
const DIV: usize = 0xff04;
// Timer counter
const TIMA: usize = 0xff05;
// Timer modulo
const TMA: usize = 0xff06;
// Timer control
const TAC: usize = 0xff07;

impl Timer {
    pub fn new() -> Self {
        Self {
            div_clocksum: 212,
            timer_clocksum: 0,

            div: 0xab,
            tima: 0,
            tma: 0,
            tac: 0xf8,
        }
    }

    fn is_enabled(&self) -> bool {
        self.tac & 0b100 != 0
    }

    fn cpu_clock_speed(&self) -> u64 {
        1024 * 4096
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
            _ => {
                return Err(Error::from_address_with_source(
                    address,
                    "timer read".to_string(),
                ))
            }
        };
        Ok(value)
    }

    fn _write(&mut self, address: Address, value: u8) -> crate::error::Result<()> {
        match address {
            // writing any value to DIV resets it to 0
            DIV => {
                self.div = 0;
                self.div_clocksum = 0;
            }
            TIMA => self.tima = value,
            TMA => self.tma = value,
            TAC => self.tac = 0b11111000 | 0b111 & value,
            _ => {
                return Err(Error::from_address_with_source(
                    address,
                    "timer write".to_string(),
                ))
            }
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
    fn step(
        &mut self,
        state: &crate::gameboy::GameBoyState,
    ) -> crate::error::Result<crate::component::ElapsedTime> {
        // DIV register increments every 256 T-cycles
        self.div_clocksum += 1;
        if self.div_clocksum == 256 {
            self.div_clocksum = 0;
            self.div = self.div.wrapping_add(1);
        }

        if self.is_enabled() {
            self.timer_clocksum += 1;

            if self.timer_clocksum == self.cpu_clock_speed() / self.get_frequency() {
                // Increment TIMA
                self.tima = self.tima.wrapping_add(1);

                // When TIMA overflows, send an interrupt and reset TIMA to TMA
                if self.tima == 0x00 {
                    info!("Sending timer interrupt");
                    state.memory_bus.borrow_mut().interrupt(Interrupt::Timer)?;
                    self.tima = self.tma;
                }
                self.timer_clocksum = 0;
            }
        }

        Ok(1)
    }
}
