use log::debug;

pub enum Interrupt {
    VBlank,
    Stat,
    Timer,
    Joypad,
}

#[derive(Clone)]
pub struct InterruptRegs {
    pub interrupt_flag: u8
}

impl InterruptRegs {
    pub fn new() -> Self {
        Self {
            interrupt_flag: 0
        }
    }

    pub fn interrupt(&mut self, interrupt: Interrupt) {
        debug!("Interrupting");
        let bit = match interrupt {
            Interrupt::VBlank => 0,
            Interrupt::Stat => 1,
            Interrupt::Timer => 2,
            Interrupt::Joypad => 4,
        };
        self.interrupt_flag |= 1 << bit;
    }
}