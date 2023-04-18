use crate::{component::Address, Result, Error};

use super::dac::Dac;

pub struct WaveChannel {
    // DAC enable
    pub nr30: u8,

    // length timer [write-only]
    pub length_timer: u16,

    // output level
    pub output_level: u8,

    // wavelength low [write-only]
    pub nr33: u8,

    // wavelength high & control
    pub nr34: u8,

    // wave pattern ram
    pub wave_pattern: [u8; 16],
    
    // stores number of T-cycles until next waveform step
    freq_timer: u16,

    waveform_step: u8,
    on: bool,
    dac: Dac,
}


impl WaveChannel {
    pub fn new() -> Self {
        let mut channel = Self {
            nr30: 0,
            length_timer: 0,
            output_level: 0,
            nr33: 0,
            nr34: 0,
            wave_pattern: [0; 16],

            freq_timer: 0,

            waveform_step: 1,
            on: false,
            dac: Dac::new()
        };
        channel.reset_frequency();
        channel
    }

    fn sample(&self) -> u8 {

        let current_waveform = self.waveform_amplitude();
        
        self.apply_volume(current_waveform)
    }

    fn apply_volume(&self, input: u8) -> u8 {
        match (self.output_level >> 5) & 0b11 {
            0 => 0,
            1 => input,
            2 => input >> 1,
            3 => input >> 2,
            _ => unreachable!()
        }
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

    pub fn sample_dac(&mut self) -> f32 {
        if !self.on {
            return 0.
        }
        
        self.dac.to_analog(self.sample())
    }

    // called every clock cycle - a rate of 4 MHz
    pub fn tick(&mut self) {
        self.freq_timer -= 1;
        if self.freq_timer == 0 {
            self.reset_frequency();
            self.waveform_step = (self.waveform_step + 1) % 32;
        }
    }

    pub fn tick_length_counter(&mut self) {
        if (self.nr34 >> 6) & 1 == 0 {
            return;
        }

        if self.length_timer > 0 {
            self.length_timer -= 1;
            
            if self.length_timer == 0 {
                self.disable()
            }
        }
    }

    fn wavelength(&self) -> u16 {
        ((self.nr34 as u16 & 0b111) << 8) | self.nr33 as u16
    }

    // The rate at which the channel steps through the 8 steps in its waveform is
    // 2097152 / (2048 - wavelength) Hz = 2 / (2048 - wavelength) MHz.
    // The channel takes a step once the frequency timer hits 0, then resets the timer.
    // Since the frequency timer is decremented at a rate of 4 MHz, it will reach zero
    // at a rate of 4 / initial_value MHz, so the initial value must be (2048 - wavelength) * 2.
    fn reset_frequency(&mut self) {
        self.freq_timer = (2048 - self.wavelength()) * 2;
    }

    fn waveform_amplitude(&self) -> u8 {
        let byte = self.wave_pattern[self.waveform_step as usize / 2];
        if self.waveform_step % 2 == 0 {
            byte >> 4
        } else {
            byte & 0b1111
        }
    }
    
    fn trigger(&mut self) {
        self.enable();
        if self.length_timer == 0 {
            self.length_timer = 64;
        }
        self.reset_frequency();
    }

    pub fn read(&self, address: Address) -> Result<u8> {
        match address {
            0xff1a => Ok(self.nr30 | 0b01111111),
            0xff1b => Ok(0xff),
            0xff1c => Ok(self.output_level | 0b10011111),
            0xff1d => Ok(0xff),
            0xff1e => Ok(self.nr34 | 0b10111111),
            0xff30 ..= 0xff3f => Ok(self.wave_pattern[address - 0xff30]),
            _ => Err(Error::from_address_with_source(address, "square".to_string()))
        }
    }

    pub fn write(&mut self, address: Address, value: u8) -> Result<()> {
        match address {
            0xff1a => self.nr30 = value,
            0xff1b => self.length_timer = 256 - value as u16,
            0xff1c => self.output_level = value,
            0xff1d => self.nr33 = value,
            0xff1e => {
                self.nr34 = value;
                if value & (1 << 7) != 0 {
                    self.trigger();
                }
            },
            0xff30 ..= 0xff3f => self.wave_pattern[address - 0xff30] = value,
            _ => return Err(Error::from_address_with_source(address, "channel1".to_string()))
        }
        Ok(())
    }
}