use crate::component::{Address, Addressable, Steppable, BatchSteppable, TickCount};
use crate::error::Error;
use crate::interrupt::{Interrupt, InterruptRegs};
use log::info;

#[derive(Clone, Debug)]
pub struct Timer {
    /// Number of clock cycles per second.
    div_clocksum: u64,
    timer_clocksum: u64,

    // Timer registers
    div: u8,
    tima: u8,
    tma: u8,
    tac: u8,

    pub last_update: TickCount,
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

            last_update: 0,
        }
    }

    pub fn get_div(&self) -> u8 {
        self.div
    }

    pub fn get_tima(&self) -> u8 {
        self.tima
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
}

impl Addressable for Timer {
    fn read_u8(&mut self, address: Address) -> crate::error::Result<u8> {
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

    fn write_u8(&mut self, address: Address, value: u8) -> crate::error::Result<()> {
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

impl Steppable for Timer {
    type Context = InterruptRegs;

    fn step(
        &mut self,
        interrupt_regs: &mut Self::Context,
        _elapsed: u32,
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
                    interrupt_regs.interrupt(Interrupt::Timer);
                    self.tima = self.tma;
                }
                self.timer_clocksum = 0;
            }
        }

        Ok(1)
    }
}

impl BatchSteppable for Timer {
    type Context = InterruptRegs;

    fn fast_forward(&mut self, interrupt_regs: &mut Self::Context, current_time: TickCount) -> crate::Result<()> {
        let elapsed_time = current_time - self.last_update;
        if elapsed_time == 0 {
            return Ok(())
        }

        // handle DIV
        if elapsed_time >= 256 - u128::from(self.div_clocksum) {
            let remainder = elapsed_time - (256 - u128::from(self.div_clocksum));
            self.div = ((u128::from(self.div) + 1 + remainder / 256) % 256).try_into().unwrap();
        }

        self.div_clocksum += (elapsed_time % 256) as u64; // range of 0-255 is guaranteed to fit into u64 
        self.div_clocksum %= 256;

        // handle TIMA
        if self.is_enabled() {
            let timer_clocksum_max: u64 = self.cpu_clock_speed() / self.get_frequency();

            if elapsed_time >= u128::from(timer_clocksum_max) - u128::from(self.timer_clocksum) {
                let remainder: u128 = elapsed_time - (u128::from(timer_clocksum_max) - u128::from(self.timer_clocksum));
                let tima_increase = 1 + remainder / u128::from(timer_clocksum_max);

                if tima_increase >= 256 - u128::from(self.tima) {
                    let remainder = tima_increase - (256 - u128::from(self.tima));
                    self.tima = self.tma + u8::try_from(remainder % (256 - u128::from(self.tma))).unwrap();
                    // TIMA overflows, so send an interrupt
                    interrupt_regs.interrupt(Interrupt::Timer);
                } else {
                    self.tima += u8::try_from(tima_increase).unwrap();
                }
            }

            self.timer_clocksum += (elapsed_time % u128::from(timer_clocksum_max)) as u64;
            self.timer_clocksum %= timer_clocksum_max;
        }

        self.last_update = current_time;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{component::{Steppable, BatchSteppable}, interrupt::InterruptRegs};

    use super::Timer;

    #[test]
    fn test_batch_step_same_outcome_tima() {
        for initial_tima in 250u8 ..= 255 {
            for initial_timer_clocksum in 0 .. 32 {
                let initial_timer_clocksum = 128 * initial_timer_clocksum;
                for power in 0 ..= 9 {
                    let mut timer = Timer::new();
                    let mut timer2 = Timer::new();
                    let mut interrupt_regs = InterruptRegs::new();
                    let mut interrupt_regs2 = InterruptRegs::new();
                    timer.tima = initial_tima;
                    timer2.tima = initial_tima;
                    timer.timer_clocksum = initial_timer_clocksum;
                    timer2.timer_clocksum = initial_timer_clocksum;

                    let elapsed = u128::pow(5, power);

                    for _ in 0 .. elapsed {
                        timer.step(&mut interrupt_regs, 1).unwrap();
                    }

                    timer2.fast_forward(&mut interrupt_regs2, elapsed).unwrap();

                    println!("initial tima: {} - initial timer_clocksum: {} - elapsed: {}", initial_tima, initial_timer_clocksum, elapsed);
                    assert_eq!(timer.div, timer2.div);
                    assert_eq!(timer.tima, timer2.tima);
                    assert_eq!(interrupt_regs.interrupt_flag, interrupt_regs2.interrupt_flag);
                }
            }
        }
    }
}