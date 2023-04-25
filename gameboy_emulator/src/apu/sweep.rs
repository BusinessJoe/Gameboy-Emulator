pub struct FrequencySweep {
    register: u8,
    enabled: bool,
    shadow_frequency: u16,
    timer: u8,
}

impl FrequencySweep {
    pub fn new(register: u8) -> Self {
        Self {
            register,
            enabled: false,
            shadow_frequency: 0,
            timer: 0,
        }
    }

    pub fn get(&self) -> u8 {
        self.register
    }

    pub fn set(&mut self, value: u8) {
        self.register = value;
    }

    pub fn tick(&mut self) -> Option<u16> {
        let mut return_val = None;

        if self.timer == 0 {
            return None;
        }
        self.timer -= 1;
        if self.timer == 0 {
            if self.pace() > 0 {
                self.timer = self.pace()
            } else {
                self.timer = 8;
            }

            if self.enabled && self.pace() != 0 {
                let new_frequency = self.calculate_frequency();

                if new_frequency > 2047 {
                    self.enabled = false;
                }

                if new_frequency <= 2047 && self.slope() > 0 {
                    return_val = Some(new_frequency);
                    self.shadow_frequency = new_frequency;

                    // another overflow check
                    if self.calculate_frequency() > 2047 {
                        self.enabled = false;
                    }
                }
            }
        }

        return_val
    }

    pub fn trigger(&mut self, frequency: u16) {
        self.shadow_frequency = frequency;
        if self.pace() > 0 {
            self.timer = self.pace();
        } else {
            self.timer = 8;
        }
        if self.pace() != 0 || self.slope() != 0 {
            self.enabled = true;
        }
        if self.slope() != 0 {
            if self.calculate_frequency() > 2047 {
                self.enabled = false;
            }
        }
    }

    fn calculate_frequency(&self) -> u16 {
        let new_frequency;
        if self.direction() == 0 {
            new_frequency = self.shadow_frequency + self.shadow_frequency >> self.slope();
        } else {
            new_frequency = self.shadow_frequency - self.shadow_frequency >> self.slope();
        }

        new_frequency
    }

    fn pace(&self) -> u8 {
        self.register >> 4
    }

    // 0 is addition, 1 is subtraction
    fn direction(&self) -> u8 {
        (self.register >> 3) & 1
    }

    fn slope(&self) -> u8 {
        self.register & 0b111
    }
}
