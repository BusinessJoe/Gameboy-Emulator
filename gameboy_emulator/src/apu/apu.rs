use ringbuf::{HeapRb, Rb};

use crate::component::Addressable;

use super::{square::SquareChannel, wave::WaveChannel, noise::NoiseChannel, global_control_regs::GlobalControlRegisters};

/// Audio processing unit
pub struct Apu {
    div_apu: u8,
    old_div: u8,

    apu_enable: bool,

    // 0xff25 - sound panning
    /*  Bit 7 - Mix channel 4 into left output
        Bit 6 - Mix channel 3 into left output
        Bit 5 - Mix channel 2 into left output
        Bit 4 - Mix channel 1 into left output
        Bit 3 - Mix channel 4 into right output
        Bit 2 - Mix channel 3 into right output
        Bit 1 - Mix channel 2 into right output
        Bit 0 - Mix channel 1 into right output  */
    nr51: u8,

    // 0xff24 - master volume & VIN panning
    // For volume bits, a value of 0 is a volume of 1 and a value of 7 is a volume of 8.
    /*  Bit 7   - Mix VIN into left output  (1=Enable)
        Bit 6-4 - Left output volume        (0-7)
        Bit 3   - Mix VIN into right output (1=Enable)
        Bit 2-0 - Right output volume       (0-7)  */
    nr50: u8,

    channel1: SquareChannel,
    channel2: SquareChannel,
    channel3: WaveChannel,
    channel4: NoiseChannel,

    audio_buffer: HeapRb<f32>,

    // Nearest-neighbour sampling.
    // Counts down with each clock tick, gathers an audio sample when it hits 0 then resets.
    sample_counter: u32,
}

impl Apu {
    pub fn new() -> Self {
        Self {
            div_apu: 0,
            old_div: 0,

            nr50: 0,
            nr51: 0,
            apu_enable: false,

            channel1: SquareChannel::new(true, 0xff10),
            channel2: SquareChannel::new(true, 0xff15),
            channel3: WaveChannel::new(),
            channel4: NoiseChannel::new(),
            
            // TODO: lower this later
            /// allocate enough capacity for 10 frames of audio
            audio_buffer: HeapRb::new(2048),

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
        self.audio_buffer.pop_iter().collect()
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
        if !self.apu_enable {
            self.audio_buffer.push_overwrite(0.);
            self.audio_buffer.push_overwrite(0.);
            return;
        }

        let channel_samples = [
            self.channel1.sample_dac(), 
            self.channel2.sample_dac(), 
            self.channel3.sample_dac(), 
            self.channel4.sample_dac()];

        let mut right_sample = 0.;
        let mut left_sample = 0.;

        // Mixing
        for i in 0 .. 4 {
            if (self.nr51 >> i) & 1 == 1 {
                right_sample += channel_samples[i] / 4.;
            }
            if (self.nr51 >> (i + 4)) & 1 == 1 {
                left_sample += channel_samples[i] / 4.;
            }
        }

        // Volume
        right_sample *= ((self.nr50 & 0b111) + 1) as f32 / 8.;
        left_sample *= (((self.nr50 >> 4) & 0b111) + 1) as f32 / 8.;

        // High pass filter goes here

        // Output
        self.audio_buffer.push_overwrite(left_sample);
        self.audio_buffer.push_overwrite(right_sample);
    }

    // Write that ignores whether the apu is enabled
    fn write_u8_direct(&mut self, address: crate::component::Address, data: u8) -> crate::Result<()> {
        match address {
            0xff10 ..= 0xff14 => self.channel1.write(address, data)?,
            0xff15 => {},
            0xff16 ..= 0xff19 => self.channel2.write(address, data)?,
            0xff1a ..= 0xff1e => self.channel3.write(address, data)?,
            0xff1f => {},
            0xff20 ..= 0xff23 => self.channel4.write(address, data)?,
            0xff24 => self.nr50 = data,
            0xff25 => self.nr51 = data,
            0xff26 => {
                self.apu_enable = (data & 0b10000000) != 0;
                if !self.apu_enable {
                    for i in 0xff10 ..= 0xff25 {
                        self.write_u8_direct(i, 0)?;
                    }
                }
            }
            0xff27 ..= 0xff2f => {},
            0xff30 ..= 0xff3f => self.channel3.write(address, data)?,
            _ => return Err(crate::Error::from_address_with_source(address, "APU".to_string()))
        }
        Ok(())
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
            0xff24 => Ok(self.nr50),
            0xff25 => Ok(self.nr51),
            0xff26 => {
                let val = (u8::from(self.apu_enable) << 7) | 0b01110000 
                    | (u8::from(self.channel4.on) << 3)
                    | (u8::from(self.channel3.on) << 2)
                    | (u8::from(self.channel2.on) << 1)
                    | (u8::from(self.channel1.on) << 0);
                Ok(val)
            }
            0xff27 ..= 0xff2f => Ok(0xff),
            0xff30 ..= 0xff3f => self.channel3.read(address),
            _ => Err(crate::Error::from_address_with_source(address, "APU".to_string()))
        }
    }

    fn write_u8(&mut self, address: crate::component::Address, data: u8) -> crate::Result<()> {
        // writes to registers are disabled (except for NR52) when apu is off
        if !self.apu_enable && 0xff10 <= address && address <= 0xff25 {
            return Ok(());
        }

        self.write_u8_direct(address, data)
    }
}