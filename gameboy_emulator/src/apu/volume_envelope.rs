enum Direction {
    Increase,
    Decrease,
}

pub struct VolumeEnvelope {
    initial_volume: u8,
    direction: Direction,
    period: u8,

    volume: u8,
    timer: u8,
}

impl VolumeEnvelope {
    pub fn new(register: u8) -> Self {
        Self {
            initial_volume: register >> 4,
            direction: if register & 0b1000 == 0 {
                Direction::Decrease
            } else {
                Direction::Increase
            },
            period: register & 0b111,

            volume: register >> 4,
            timer: register & 0b111,
        }
    }

    pub fn tick(&mut self) {
        if self.timer == 0 {
            return;
        }
        self.timer -= 1;
        if self.timer == 0 {
            self.timer = self.period;
            match self.direction {
                Direction::Increase if self.volume < 0xf => {
                    self.volume += 1;
                }
                Direction::Decrease if self.volume > 0 => {
                    self.volume -= 1;
                }
                _ => {}
            }
        }
    }

    pub fn volume(&self) -> u8 {
        self.volume
    }
}
