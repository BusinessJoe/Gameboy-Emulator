use crate::gameboy::Interrupt;
use crate::memory::MemoryBus;
use std::sync::{Arc, Mutex};

pub struct Timer {
    /// Number of clock cycles per second.
    clock_speed: u64,
    /// The Game Boy's memory bus.
    memory_bus: Arc<Mutex<MemoryBus>>,
    div_clocksum: u64,
    timer_clocksum: u64,
}

const TIMA: usize = 0xFF05;
const TMA: usize = 0xFF06;
const TAC: usize = 0xFF07;

impl Timer {
    pub fn new(clock_speed: u64, memory_bus: Arc<Mutex<MemoryBus>>) -> Self {
        Self {
            clock_speed,
            memory_bus,
            div_clocksum: 0,
            timer_clocksum: 0,
        }
    }

    fn is_enabled(&self) -> bool {
        let memory = self.memory_bus.lock().unwrap();
        (memory.get(TAC) >> 2) & 1 == 1
    }

    fn get_frequency(&self) -> u64 {
        let memory = self.memory_bus.lock().unwrap();
        let bits = memory.get(TAC) & 0b11;
        match bits {
            0b00 => 4_096,
            0b01 => 262_144,
            0b10 => 65_536,
            0b11 => 16_384,
            _ => panic!(),
        }
    }

    pub fn tick(&mut self, elapsed_cycles: u8) {
        // Manage divider timer
        self.div_clocksum += u64::from(elapsed_cycles);
        if self.div_clocksum >= 256 {
            self.div_clocksum -= 256;
            let mut memory = self.memory_bus.lock().unwrap();
            let mut divider_register = memory.get(0xFF04);
            divider_register = divider_register.wrapping_add(1);
            memory.set(0xFF04, divider_register);
        }

        if self.is_enabled() {
            self.timer_clocksum += u64::from(elapsed_cycles) * 4;

            let freq = self.get_frequency();

            let mut memory = self.memory_bus.lock().unwrap();
            while self.timer_clocksum >= self.clock_speed / freq {
                // Increment TIMA
                let counter = memory.get(TIMA);
                memory.set(TIMA, counter.wrapping_add(1));

                // When TIMA overflows, send an interrupt and reset TIMA to TMA
                if memory.get(TIMA) == 0x00 {
                    memory.interrupt(Interrupt::Timer);
                    let timer_modulo = memory.get(TMA);
                    memory.set(TIMA, timer_modulo);
                }
                self.timer_clocksum -= self.clock_speed / freq;
            }
        }
    }
}
