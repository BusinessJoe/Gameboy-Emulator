use ringbuf::{HeapRb, Rb};

use crate::component::Addressable;

use super::{square::SquareChannel, wave::WaveChannel, noise::NoiseChannel};

/// Audio processing unit
pub struct Apu {
    div_apu: u8,
    old_div: u8,

    channel1: SquareChannel,
    channel2: SquareChannel,
    channel3: WaveChannel,
    channel4: NoiseChannel,

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

            channel1: SquareChannel::new(true, 0xff10),
            channel2: SquareChannel::new(true, 0xff15),
            channel3: WaveChannel::new(),
            channel4: NoiseChannel::new(),
            
            // TODO: lower this later
            /// allocate enough capacity for 10 frames of audio
            left_audio_buffer: HeapRb::new(8192),
            right_audio_buffer: HeapRb::new(8192),

            sample_counter: 95, // = 4194304 / 44100 (rounded)
            //sample_counter: 19, // = 4194304 / (5* 44100) (rounded)
        }
    }

    pub fn tick(&mut self, new_div: u8) {
        if self.old_div & (1 << 5) != 0 && new_div & (1 << 5) == 0 {
            self.increment_frame_sequencer();
        }
        self.old_div = new_div;

        // tick channels
        self.channel1.tick();
        self.channel2.tick();
        self.channel3.tick();
        self.channel4.tick();

        self.sample_counter -= 1;
        if self.sample_counter == 0 {
            self.sample_counter = 95;
            self.gather_sample();
            // self.sub_sample_counter -= 1;
            // if self.sub_sample_counter == 0 {
            //     self.sub_sample_counter = 10;
            //     self.sample_counter += 1;
            // }
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
            self.channel2.tick_length_counter();
            self.channel3.tick_length_counter();
            self.channel4.tick_length_counter();
        }
        if self.div_apu == 7 {
            // volume envelope
            self.channel1.tick_volume_envelope();
            self.channel2.tick_volume_envelope();
            self.channel4.tick_volume_envelope();
        }
        if self.div_apu == 2 || self.div_apu == 6 {
            // sweep
            self.channel1.tick_frequency_sweep();
        }
    }

    fn gather_sample(&mut self) {
        let ch1 = self.channel1.sample_dac();
        let ch2 = self.channel2.sample_dac();
        let ch3 = self.channel3.sample_dac();
        let ch4 = self.channel4.sample_dac();
        let sample = (ch1 + ch2 + ch3 + ch4) / 4.;
        self.left_audio_buffer.push_overwrite(sample);
    }
}

impl Addressable for Apu {
    fn read_u8(&mut self, address: crate::component::Address) -> crate::Result<u8> {
        match address {
            0xff10 ..= 0xff14 => self.channel1.read(address),
            0xff15 => Ok(0xff),
            0xff16 ..= 0xff19 => self.channel2.read(address),
            0xff1a ..= 0xff1e => self.channel3.read(address),
            0xff1f => Ok(0xff),
            0xff20 ..= 0xff23 => self.channel4.read(address),
            0xff24 ..= 0xff2f => Ok(0xff),
            0xff30 ..= 0xff3f => self.channel3.read(address),
            _ => Err(crate::Error::from_address_with_source(address, "APU".to_string()))
        }
    }

    fn write_u8(&mut self, address: crate::component::Address, data: u8) -> crate::Result<()> {
        match address {
            0xff10 ..= 0xff14 => self.channel1.write(address, data)?,
            0xff15 => {},
            0xff16 ..= 0xff19 => self.channel2.write(address, data)?,
            0xff1a ..= 0xff1e => self.channel3.write(address, data)?,
            0xff1f => {},
            0xff20 ..= 0xff23 => self.channel4.write(address, data)?,
            0xff24 ..= 0xff2f => {},
            0xff30 ..= 0xff3f => self.channel3.write(address, data)?,
            _ => return Err(crate::Error::from_address_with_source(address, "APU".to_string()))
        }
        Ok(())
    }
}