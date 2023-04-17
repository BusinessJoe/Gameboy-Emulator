pub struct Channel2 {
    // 0xff16 - channel 2 lengh timer & duty cycle
    pub nr21: u8,

    // 0xff17 - channel 2 volume & envelope
    pub nr22: u8,

    // 0xff18 - channel 2 wavelength low [write-only]
    pub nr23: u8,

    // 0xff19 - channel 2 wavelength high & control
    pub nr24: u8,

    // stores number of T-cycles until next waveform step
    freq_timer: u32,

    // current position in waveform
    waveform_step: u8,
}

impl Channel2 {
    pub fn new() -> Self {
        let mut ch2 = Self {
            nr21: 0,
            nr22: 0,
            nr23: 0,
            nr24: 0,
            
            freq_timer: 0,
            waveform_step: 0,
        };
        ch2.reset_frequency();
        ch2
    }

    pub fn sample(&self) -> u8 {
        let current_waveform = (self.duty_cycle_waveform() >> self.waveform_step) & 1;
        
        current_waveform * 0xf
    }

    // called every clock cycle - a rate of 4 MHz
    pub fn tick(&mut self) {
        self.freq_timer -= 1;
        if self.freq_timer == 0 {
            self.reset_frequency();
            self.waveform_step = (self.waveform_step + 1) % 8;
        }
    }

    fn wavelength(&self) -> u32 {
        ((self.nr24 as u32 & 0b111) << 8) | self.nr23 as u32
    }

    // The rate at which the channel steps through the 8 steps in its waveform is
    // 1048576 / (2048 - wavelength) Hz = 1 / (2048 - wavelength) MHz.
    // The channel takes a step once the frequency timer hits 0, then resets the timer.
    // Since the frequency timer is decremented at a rate of 4 MHz, it will reach zero
    // at a rate of 4 / initial_value MHz, so the initial value must be (2048 - wavelength) * 4.
    fn reset_frequency(&mut self) {
        self.freq_timer = (2048 - self.wavelength()) * 4;
    }

    fn duty_cycle_waveform(&self) -> u8 {
        match (self.nr21 & 0b11000000) >> 6 {
            0 => 0b10000000,
            1 => 0b10000001,
            2 => 0b11100001,
            3 => 0b01111110,
            _ => panic!()
        }
    }
}