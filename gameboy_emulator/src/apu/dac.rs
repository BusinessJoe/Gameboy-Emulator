use super::utils::digital_to_analog;

pub struct Dac {
    pub enabled: bool,
    capacitor: f32,
}

impl Dac {
    pub fn new() -> Self {
        Dac {
            enabled: false,
            capacitor: 0.,
        }
    }

    pub fn to_analog(&mut self, input: u8) -> f32 {
        digital_to_analog(input)
    }

    fn high_pass(&mut self, input: f32) -> f32 {
        let mut out = 0.0;
        if self.enabled || true {
            out = input - self.capacitor;

            // capacitor slowly charges to input
            // charge factor is 0.999958^(4194304/rate), so 0.996 at 44100 Hz
            self.capacitor = input - out * 0.996;
        }
        out
    }
}
