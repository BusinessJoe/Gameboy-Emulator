use super::{AddressingError, MemoryBankController};

struct ClockCounter;

impl ClockCounter {
    fn read(&self) -> u8 {
        0xff
    }

    fn write(&mut self, _value: u8) {
        // do nothing for now
    }
}

pub struct Mbc3 {
    ram_timer_enable: bool,
    rom_bank_number: u8,
    // write only register - controls ram bank number or RTC register select
    ram_rtc_select: u8,
    latch_clock_data: u8,
    clock: ClockCounter,
}

impl MemoryBankController for Mbc3 {
    fn read(
        &self,
        address: super::Address,
        rom: &[u8],
        ram: &[u8],
    ) -> Result<u8, super::AddressingError> {
        let value = match address {
            0x0000..=0x3fff => rom[address],
            0x4000..=0x7fff => {
                let mut address = address & 0x3fff;
                address |= (self.rom_bank_number as usize) << 14;
                rom[address]
            }
            0xa000..=0xbfff => match self.ram_rtc_select & 0xf {
                0x0..=0x3 => {
                    let mut address = address & 0x1fff;
                    address |= (self.ram_rtc_select as usize) << 13;
                    ram[address]
                }
                0x8..=0xc => self.clock.read(),
                _ => 0xff,
            },
            _ => return Err(AddressingError(address)),
        };

        //println!("read address {:#x} value {:#x}", address, value);
        Ok(value)
    }

    fn write(
        &mut self,
        address: super::Address,
        value: u8,
        _rom: &mut [u8],
        ram: &mut [u8],
    ) -> Result<(), super::AddressingError> {
        //println!("write address {:#x} value {:#x}", address, value);
        match address {
            // ram enable
            0x0000..=0x1fff => {
                self.ram_timer_enable = (value & 0xf) == 0xa;
                Ok(())
            }
            // rom bank number
            0x2000..=0x3fff => {
                let mut value = value & 0x7f;
                if value == 0 {
                    value = 1;
                }
                self.rom_bank_number = value;
                Ok(())
            }
            // ram bank number or RTC register select
            0x4000..=0x5fff => {
                self.ram_rtc_select = value;
                Ok(())
            }
            // latch clock data
            0x6000..=0x7fff => {
                self.latch_clock_data = value;
                Ok(())
            }
            // ram banks or RTC register
            0xa000..=0xbfff => {
                match self.ram_rtc_select & 0xf {
                    0x0..=0x3 => {
                        let mut address = address & 0x1fff;
                        address |= (self.ram_rtc_select as usize) << 13;
                        ram[address] = value;
                        Ok(())
                    }
                    0x8..=0xc => {
                        self.clock.write(value);
                        Ok(())
                    }
                    // do nothing
                    _ => Ok(()),
                }
            }
            _ => Err(AddressingError(address)),
        }
    }

    fn get_type(&self) -> super::MbcType {
        super::MbcType::Mbc3
    }
}

impl Mbc3 {
    pub fn new() -> Self {
        Self {
            ram_timer_enable: false,
            rom_bank_number: 0,
            ram_rtc_select: 0,
            latch_clock_data: 0,
            clock: ClockCounter,
        }
    }
}
