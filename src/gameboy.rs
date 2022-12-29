use crate::cartridge;
use crate::cpu::CPU;
use crate::memory::MemoryBus;
use crate::ppu::PPU;
use crate::timer::Timer;
use log::trace;
use std::fs;
use std::sync::{Arc, Mutex};

const CLOCK_SPEED: u64 = 4_194_304;

pub struct GameBoyState {
    pub cpu: CPU,
    //pub ppu: PPU,
    timer: Timer,
    memory_bus: Arc<Mutex<MemoryBus>>,
}

impl GameBoyState {
    pub fn new() -> Self {
        let memory_bus = Arc::new(Mutex::new(MemoryBus::new()));
        Self {
            cpu: CPU::new(memory_bus.clone()),
            //ppu: PPU::new(memory_bus.clone()),
            timer: Timer::new(CLOCK_SPEED, memory_bus.clone()),
            memory_bus,
        }
    }

    pub fn load(&mut self, filename: &str) {
        let bytes = fs::read(filename).unwrap();
        let cartridge = cartridge::build_cartridge(&bytes).unwrap();
        self.memory_bus.lock().unwrap().insert_cartridge(cartridge);
        trace!("{:#x}", self.get_memory_value(0x100));
    }

    pub fn tick(&mut self) {
        let elapsed_cycles = self.cpu.tick();
        //self.ppu.tick();
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

    pub fn get_output(&self) -> String {
        self.memory_bus.lock().unwrap().output_string.clone()
    }
}

pub enum Interrupt {
    Timer,
}
