use crate::cpu::CPU;
use crate::timer::Timer;
use log::trace;
use std::fs;
use std::rc::Rc;
use std::sync::Mutex;

const CLOCK_SPEED: u64 = 4_194_304;

pub struct GameBoyState {
    pub cpu: CPU,
    timer: Timer,
    memory_bus: Rc<Mutex<MemoryBus>>,
}

impl GameBoyState {
    pub fn new() -> Self {
        let memory_bus = Rc::new(Mutex::new(MemoryBus::default()));
        Self {
            cpu: CPU::new(memory_bus.clone()),
            timer: Timer::new(CLOCK_SPEED, memory_bus.clone()),
            memory_bus,
        }
    }

    pub fn load(&mut self, filename: &str) {
        let bytes = fs::read(filename).unwrap();
        for (idx, b) in bytes.into_iter().enumerate() {
            self.set_memory_value(idx, b);
        }
        trace!("{:#x}", self.get_memory_value(0x100));
    }

    pub fn tick(&mut self) {
        let elapsed_cycles = self.cpu.tick();
        self.timer.tick(elapsed_cycles);
    }

    pub fn get_memory_value(&self, address: usize) -> u8 {
        trace!("getting memory at {:#x}", address);
        self.memory_bus.lock().unwrap().get(address)
    }

    pub fn set_memory_value(&mut self, address: usize, value: u8) {
        trace!("setting memory at {:#x}", address);
        self.memory_bus.lock().unwrap().set(address, value);
    }
}

pub enum Interrupt {
    Timer,
}

/// Mock memory bus
#[derive(Debug)]
pub struct MemoryBus {
    pub data: [u8; 0x10000],
}

impl Default for MemoryBus {
    fn default() -> Self {
        Self { data: [0; 0x10000] }
    }
}

impl MemoryBus {
    pub fn get(&self, address: usize) -> u8 {
        self.data[address]
    }

    pub fn set(&mut self, address: usize, value: u8) {
        if address == 0xFF02 && value == 0x81 {
            let chr = char::from_u32(self.data[0xFF01] as u32).unwrap();
            print!("{}", chr);
        }
        self.data[address] = value;
    }

    pub fn interrupt(&mut self, interrupt: Interrupt) {
        let bit = match interrupt {
            Interrupt::Timer => 2,
        };
        let mut interrupt_flag = self.get(0xFF0F);
        interrupt_flag |= 1 << bit; 
        self.set(0xFF0F, interrupt_flag);
    }
}
