use crate::{component::Address, Result, Error};

use super::{utils::digital_to_analog, volume_envelope::VolumeEnvelope, sweep::FrequencySweep, dac::Dac};

pub struct SquareChannel {
    // sweep
    frequency_sweep: Option<FrequencySweep>,

    // lengh timer & duty cycle
    pub duty_cycle: u8,
    pub length_timer: u8,

    // volume & envelope
    pub nrx2: u8,

    // wavelength low [write-only]
    pub nrx3: u8,

    // wavelength high & control
    pub nrx4: u8,
    
    // stores number of T-cycles until next waveform step
    freq_timer: u16,

    // current position in waveform
    waveform_step: u8,

    pub on: bool,

    baseline_address: Address,
    volume_envelope: VolumeEnvelope,
    dac: Dac,
}


impl SquareChannel {
    pub fn new(sweep: bool, baseline_address: Address) -> Self {
        let mut channel = Self {
            frequency_sweep: if sweep { Some(FrequencySweep::new(0)) } else { None },
            duty_cycle: 0,
            length_timer: 0,
            nrx2: 0,
            nrx3: 0,
            nrx4: 0,
            
            freq_timer: 0,
            waveform_step: 0,

            on: false,

            baseline_address,
            volume_envelope: VolumeEnvelope::new(0),
            dac: Dac::new(),
        };
        channel.reset_frequency();
        channel
    }

    fn sample(&self) -> u8 {

        let current_waveform = self.waveform_amplitude();
        
        current_waveform * self.volume_envelope.volume()
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
            self.waveform_step = (self.waveform_step + 1) % 8;
        }
    }

    pub fn tick_length_counter(&mut self) {
        if (self.nrx4 >> 6) & 1 == 0 {
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

    pub fn tick_frequency_sweep(&mut self) {
        match &mut self.frequency_sweep {
            Some(ref mut frequency_sweep) => {
                if let Some(new_frequency) = frequency_sweep.tick() {
                    self.nrx3 = (new_frequency & 0b11111111) as u8;
                    self.nrx4 = (self.nrx4 & 0b11111000) | (new_frequency >> 8) as u8;
                }
            }
            None => {
                panic!()
            }
        }
        
    }

    fn wavelength(&self) -> u16 {
        ((self.nrx4 as u16 & 0b111) << 8) | self.nrx3 as u16
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
            [0, 0, 0, 0, 0, 0, 1, 1],
            [0, 0, 0, 0, 1, 1, 1, 1],
            [1, 1, 1, 1, 1, 1, 0, 0],
        ][self.duty_cycle as usize][self.waveform_step as usize]
    }
    
    fn trigger(&mut self) {
        self.enable();
        if self.length_timer == 0 {
            self.length_timer = 64;
        }
        self.reset_frequency();

        self.volume_envelope = VolumeEnvelope::new(self.nrx2);
        let wavelength = self.wavelength();
        if let Some(frequency_sweep) = self.frequency_sweep.as_mut() {
            frequency_sweep.trigger(wavelength)
        }
       
    }

    pub fn read(&self, address: Address) -> Result<u8> {
        match address - self.baseline_address {
            0 => match &self.frequency_sweep {
                Some(fs) => {
                    Ok(fs.get() | 0b10000000)
                }
                None => {
                    Ok(0xff)
                }
            }
            1 => Ok(self.duty_cycle << 6 | 0b00111111),
            2 => Ok(self.nrx2),
            3 => Ok(0xff),
            4 => Ok(self.nrx4 | 0b10111111),
            _ => unreachable!(),
        }
    }

    pub fn write(&mut self, address: Address, value: u8) -> Result<()> {
        match address - self.baseline_address {
            0 => if let Some(fs) = &mut self.frequency_sweep { fs.set(value) },
            1 => {
                self.duty_cycle = (value & 0b11000000) >> 6;
                self.length_timer = 64 - (value & 0b00111111);
            }
            2 => {
                self.nrx2 = value;
                if value & 0xf8 != 0 {
                    self.dac.enabled = true;
                } else {
                    self.dac.enabled = false;
                    self.disable();
                }
            }
            3 => self.nrx3 = value,
            4 => {
                self.nrx4 = value;
                if value & (1 << 7) != 0 {
                    self.trigger();
                }
            }
            _ => unreachable!(),
        }
        Ok(())
    }
}