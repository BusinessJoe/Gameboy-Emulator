use crate::{component::Address, Result, Error};

use super::utils::digital_to_analog;

pub struct Channel1 {
    // 0xff10 - channel 1 sweep
    pub nr10: u8,

    // 0xff11 - channel 1 lengh timer & duty cycle
    pub duty_cycle: u8,
    pub length_timer: u8,

    // 0xff12 - channel 1 volume & envelope
    pub nr12: u8,

    // 0xff13 - channel 1 wavelength low [write-only]
    pub nr13: u8,

    // 0xff14 - channel 1 wavelength high & control
    pub nr14: u8,
    
    // stores number of T-cycles until next waveform step
    freq_timer: u32,

    // current position in waveform
    waveform_step: u8,

    on: bool,

    baseline_address: Address,
    tick_counter: u64,
}


impl Channel1 {
    pub fn new() -> Self {
        let mut ch1 = Self {
            nr10: 0,
            duty_cycle: 0,
            length_timer: 0,
            nr12: 0,
            nr13: 0,
            nr14: 0,
            
            freq_timer: 0,
            waveform_step: 0,

            on: false,

            baseline_address: 0xff10,
            tick_counter: 0,
        };
        ch1.reset_frequency();
        ch1
    }

    fn sample(&self) -> u8 {

        let current_waveform = self.waveform_amplitude();
        
        current_waveform * 0xf
        // self.test_value += 0.5;
        // if self.test_value > 15. {
        //     self.test_value -= 15.;
        // }
        // self.test_value as u8
    }

    fn enable(&mut self) {
        if !self.on {
            self.on = true;
        }
    }

    fn disable(&mut self) {
        if self.on {
            self.on = false;
        }
    }

    pub fn sample_dac(&self) -> f32 {
        if !self.on {
            return 0.
        }
        
        digital_to_analog(self.sample())
    }

    // called every clock cycle - a rate of 4 MHz
    pub fn tick(&mut self) {
        self.tick_counter += 1;
        self.freq_timer -= 1;
        if self.freq_timer == 0 {
            self.reset_frequency();
            self.waveform_step = (self.waveform_step + 1) % 8;
        }
    }

    pub fn tick_length_counter(&mut self) {
        if (self.nr14 >> 6) & 1 == 0 {
            return;
        }

        if self.length_timer > 0 {
            self.length_timer -= 1;
            
            if self.length_timer == 0 {
                self.disable()
            }
        }
    }

    fn wavelength(&self) -> u32 {
        ((self.nr14 as u32 & 0b111) << 8) | self.nr13 as u32
    }

    // The rate at which the channel steps through the 8 steps in its waveform is
    // 1048576 / (2048 - wavelength) Hz = 1 / (2048 - wavelength) MHz.
    // The channel takes a step once the frequency timer hits 0, then resets the timer.
    // Since the frequency timer is decremented at a rate of 4 MHz, it will reach zero
    // at a rate of 4 / initial_value MHz, so the initial value must be (2048 - wavelength) * 4.
    fn reset_frequency(&mut self) {
        self.freq_timer = (2048 - self.wavelength()) * 4;
    }

    fn waveform_amplitude(&self) -> u8 {
        [
            [0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 1, 1, 1],
            [0, 1, 1, 1, 1, 1, 1, 0],
        ][self.duty_cycle as usize][self.waveform_step as usize]
    }
    
    fn trigger(&mut self) {
        self.enable();
        if self.length_timer == 0 {
            self.length_timer = 64;
        }
        self.reset_frequency();
        self.waveform_step = 0;
    }

    pub fn read(&self, address: Address) -> Result<u8> {
        match address - self.baseline_address {
            0 => Ok(self.nr10),
            1 => Ok(self.duty_cycle << 6 | 0b00111111),
            2 => Ok(self.nr12),
            3 => Ok(self.nr13),
            4 => Ok(self.nr14),
            _ => Err(Error::from_address_with_source(address, "channel1".to_string()))
        }
    }

    pub fn write(&mut self, address: Address, value: u8) -> Result<()> {
        match address - self.baseline_address {
            0 => self.nr10 = value,
            1 => {
                self.duty_cycle = (value & 0b11000000) >> 6;
                self.length_timer = 64 - (value & 0b00111111);
            }
            2 => self.nr12 = value,
            3 => self.nr13 = value,
            4 => {
                self.nr14 = value;
                if value & (1 << 7) != 0 {
                    self.trigger();
                }
            }
            _ => return Err(Error::from_address_with_source(address, "channel1".to_string()))
        }
        Ok(())
    }
}