use ringbuf::{HeapRb, Rb};

use crate::{component::{Steppable, Addressable}, cartridge::AddressingError};

use super::{channel2::Channel2, utils::digital_to_analog, channel1::Channel1};

/// Audio processing unit
pub struct Apu {
    div_apu: u8,
    old_div: u8,

    channel1: Channel1,

    left_audio_buffer: HeapRb<f32>,
    right_audio_buffer: HeapRb<f32>,

    // Nearest-neighbour sampling.
    // Counts down with each clock tick, gathers an audio sample when it hits 0 then resets.
    sample_counter: u32,
}

impl Apu {
    pub fn new() -> Self {
        Self {
            div_apu: 0,
            old_div: 0,

            channel1: Channel1::new(),

            // TODO: lower this later
            /// allocate enough capacity for 10 frames of audio
            left_audio_buffer: HeapRb::new(735 * 10),
            right_audio_buffer: HeapRb::new(735 * 10),

            sample_counter: 95 // = 4194304 / 44100 (rounded)

        }
    }

    pub fn tick(&mut self, new_div: u8) {
        if self.old_div & (1 << 5) != 0 && new_div & (1 << 5) == 0 {
            //self.increment_frame_sequencer();
        }
        self.old_div = new_div;

        // tick channels
        self.channel1.tick();
        //self.channel2.tick();

        self.sample_counter -= 1;
        if self.sample_counter == 0 {
            self.sample_counter = 95;
            self.gather_sample();
        }
    }

    pub fn get_queued_audio(&mut self) -> Vec<f32> {
        self.left_audio_buffer.pop_iter().collect()
    }

    fn increment_frame_sequencer(&mut self) {
        self.div_apu += 1;
        if self.div_apu > 7 {
            self.div_apu = 0;
        }

        if self.div_apu % 2 == 0 {
            // length counter
            self.channel1.tick_length_counter();
        }
        if self.div_apu == 7 {
            // volume envelope
        }
        if self.div_apu == 2 || self.div_apu == 6 {
            // sweep
        }
    }

    fn gather_sample(&mut self) {
        let ch1 = self.channel1.sample_dac();
        self.left_audio_buffer.push_overwrite(ch1);
    }

    fn _read(&self, address: crate::component::Address) -> crate::Result<u8> {
        match address {
            0xff10 ..= 0xff14 => self.channel1.read(address),
            0xff15 => Ok(0xff),
            // 0xff16 => Ok(self.channel2.nr21),
            // 0xff17 => Ok(self.channel2.nr22),
            // 0xff18 => Ok(self.channel2.nr23),
            // 0xff19 => Ok(self.channel2.nr24),
            0xff16 ..= 0xff19 => Ok(0xff),
            0xff1a ..= 0xff26 => Ok(0xff),
            _ => Err(crate::Error::from_address_with_source(address, "APU".to_string()))
        }
    }

    fn _write(&mut self, address: crate::component::Address, value: u8) -> crate::Result<()> {
        match address {
            0xff10 ..= 0xff14 => self.channel1.write(address, value)?,
            0xff15 => {}
            // 0xff16 => self.channel2.nr21 = value,
            // 0xff17 => self.channel2.nr22 = value,
            // 0xff18 => self.channel2.nr23 = value,
            // 0xff19 => self.channel2.nr24 = value,
            0xff16 ..= 0xff19 => {},
            0xff1a ..= 0xff26 => {},
            _ => return Err(crate::Error::from_address_with_source(address, "APU".to_string()))
        }
        Ok(())
    }
}

impl Addressable for Apu {
    fn read(&mut self, address: crate::component::Address, data: &mut [u8]) -> crate::Result<()> {
        for (offset, byte) in data.iter_mut().enumerate() {
            *byte = self._read(address + offset)?;
        }

        Ok(())
    }

    fn write(&mut self, address: crate::component::Address, data: &[u8]) -> crate::Result<()> {
        for (offset, byte) in data.iter().enumerate() {
            self._write(address + offset, *byte)?;
        }

        Ok(())
    }
}