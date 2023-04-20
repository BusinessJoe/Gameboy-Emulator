use crate::{component::Address, Result, Error};

use super::{dac::Dac, volume_envelope::VolumeEnvelope};

pub struct NoiseChannel {
    // length timer [write-only]
    pub length_timer: u8,

    // volume & envelope
    pub nr42: u8,
    volume_envelope: VolumeEnvelope,

    // frequency & randomness
    pub nr43: u8,

    // control
    pub nr44: u8,
    
    // linear feedback shift register is a 15-bits
    lfsr: u16,

    // stores number of T-cycles until next waveform step
    freq_timer: u16,

    waveform_step: u8,
    on: bool,
    dac: Dac,
}


impl NoiseChannel {
    pub fn new() -> Self {
        let mut channel = Self {
            length_timer: 0,
            nr42: 0,
            volume_envelope: VolumeEnvelope::new(0),
            nr43: 0,
            nr44: 0,

            lfsr: 0,

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
        
        current_waveform * self.volume_envelope.volume()
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
            self.tick_lfsr();
            self.waveform_step = (self.waveform_step + 1) % 32;
        }
    }

    pub fn tick_length_counter(&mut self) {
        if (self.nr44 >> 6) & 1 == 0 {
            return;
        }

        if self.length_timer > 0 {
            self.length_timer -= 1;
            
            if self.length_timer == 0 {
                self.disable()
            }
        }
    }
    
    pub fn tick_volume_envelope(&mut self) {
        self.volume_envelope.tick();
    }

    fn tick_lfsr(&mut self) {
        let xor = (self.lfsr & 1) ^ ((self.lfsr >> 1) & 1);
        self.lfsr >>= 1;
        self.lfsr |= xor << 14;
        if (self.nr43 >> 3) & 1 == 1 {
            self.lfsr |= xor << 6;
        }
    }

    // The rate at which the channel clocks LFSR is 262144 / (r * 2^s) Hz = (1/4) / (r * 2^s) MHz, 
    // where r is the value in the upper 4 bits of NR43 and s is the value in the lower 3 bits of NR43.
    // The channel takes a step once the frequency timer hits 0, then resets the timer.
    // Since the frequency timer is decremented at a rate of 4 MHz, it will reach zero
    // at a rate of 4 / initial_value MHz, so the initial value must be (r * 2^s) * 16.
    // A value of 0 for r is treated as 0.5.
    fn reset_frequency(&mut self) {
        let r = self.nr43 >> 4;
        let s = self.nr43 & 0b111;
        if r == 0 {
            self.freq_timer = (1 << s) * 8;
        } else {
            self.freq_timer = r as u16 * (1 << s) * 16;
        }
    }

    fn waveform_amplitude(&self) -> u8 {
        1 - (self.lfsr & 1) as u8
    }
    
    fn trigger(&mut self) {
        self.enable();
        if self.length_timer == 0 {
            self.length_timer = 64;
        }
        self.reset_frequency();
        
        self.volume_envelope = VolumeEnvelope::new(self.nr42);
        self.lfsr = 0x7fff;
    }

    pub fn read(&self, address: Address) -> Result<u8> {
        match address {
            0xff20 => Ok(0xff),
            0xff21 => Ok(self.nr42),
            0xff22 => Ok(self.nr43),
            0xff23 => Ok(self.nr44 | 0b00111111),
            _ => Err(Error::from_address_with_source(address, "square".to_string()))
        }
    }

    pub fn write(&mut self, address: Address, value: u8) -> Result<()> {
        match address {
            0xff20 => self.length_timer = 64 - (value & 0b111111),
            0xff21 => self.nr42 = value,
            0xff22 => self.nr43 = value,
            0xff23 => {
                self.nr44 = value;
                if value & (1 << 7) != 0 {
                    self.trigger();
                }
            },
            _ => return Err(Error::from_address_with_source(address, "channel1".to_string()))
        }
        Ok(())
    }
}