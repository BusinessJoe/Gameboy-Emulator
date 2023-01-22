use crate::component::Addressable;
use crate::gameboy::Interrupt;
use crate::memory::MemoryBus;
use log::info;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Timer<'a> {
    /// Number of clock cycles per second.
    clock_speed: u64,
    /// The Game Boy's memory bus.
    memory_bus: Rc<RefCell<MemoryBus<'a>>>,
    div_clocksum: u64,
    timer_clocksum: u64,
}

// Divider register
const DIV: usize = 0xFF04;
// Timer counter
const TIMA: usize = 0xFF05;
// Timer modulo
const TMA: usize = 0xFF06;
// Timer control
const TAC: usize = 0xFF07;

impl<'a> Timer<'a> {
    pub fn new(clock_speed: u64, memory_bus: Rc<RefCell<MemoryBus<'a>>>) -> Self {
        Self {
            clock_speed,
            memory_bus,
            div_clocksum: 0,
            timer_clocksum: 0,
        }
    }

    fn is_enabled(&self) -> bool {
        let mut memory = self.memory_bus.borrow_mut();
        (memory.read_u8(TAC).unwrap() >> 2) & 1 == 1
    }

    fn get_frequency(&self) -> u64 {
        let mut memory = self.memory_bus.borrow_mut();
        let bits = memory.read_u8(TAC).unwrap() & 0b11;
        match bits {
            0b00 => 4_096,
            0b01 => 262_144,
            0b10 => 65_536,
            0b11 => 16_384,
            _ => panic!(),
        }
    }

    pub fn tick(&mut self, elapsed_cycles: u64) {
        assert!(elapsed_cycles < 256);
        // Manage divider timer
        self.div_clocksum += elapsed_cycles;
        if self.div_clocksum >= 256 {
            self.div_clocksum -= 256;
            let mut memory = self.memory_bus.borrow_mut();
            let mut divider_register = memory.read_u8(DIV).unwrap();
            divider_register = divider_register.wrapping_add(1);
            memory.write_u8(DIV, divider_register).unwrap();
        }

        if self.is_enabled() {
            self.timer_clocksum += elapsed_cycles * 4;

            let freq = self.get_frequency();

            let mut memory = self.memory_bus.borrow_mut();
            while self.timer_clocksum >= self.clock_speed / freq {
                // Increment TIMA
                let counter = memory.read_u8(TIMA).unwrap();
                memory.write_u8(TIMA, counter.wrapping_add(1)).unwrap();

                // When TIMA overflows, send an interrupt and reset TIMA to TMA
                if memory.read_u8(TIMA).unwrap() == 0x00 {
                    info!("Sending timer interrupt");
                    memory.interrupt(Interrupt::Timer).unwrap();
                    let timer_modulo = memory.read_u8(TMA).unwrap();
                    memory.write_u8(TIMA, timer_modulo).unwrap();
                }
                self.timer_clocksum -= self.clock_speed / freq;
            }
        }
    }
}
