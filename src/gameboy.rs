use crate::cartridge;
use crate::component::{Addressable, Steppable};
use crate::cpu::CPU;
use crate::error::Result;
use crate::memory::MemoryBus;
use crate::ppu::PPU;
use crate::timer::Timer;
use log::trace;
use std::fs;
use std::sync::{Arc, Mutex};

const CLOCK_SPEED: u64 = 4_194_304;

pub type Observer = fn(chr: char);

pub struct GameBoyState {
    pub cpu: Arc<Mutex<CPU>>,
    pub ppu: Arc<Mutex<PPU>>,
    pub memory_bus: Arc<Mutex<MemoryBus>>,
    serial_port_observer: Option<Observer>,
    timer: Timer,
}

impl GameBoyState {
    pub fn new() -> Self {
        let ppu = Arc::new(Mutex::new(PPU::new()));
        let memory_bus = Arc::new(Mutex::new(MemoryBus::new(ppu.clone())));
        Self {
            cpu: Arc::new(Mutex::new(CPU::new())),
            ppu: ppu.clone(),
            memory_bus: memory_bus.clone(),
            serial_port_observer: None,
            timer: Timer::new(CLOCK_SPEED, memory_bus),
        }
    }

    pub fn load(&mut self, filename: &str) -> Result<()> {
        let bytes = fs::read(filename).unwrap();
        let cartridge = cartridge::build_cartridge(&bytes).unwrap();
        let mut memory_bus = self.memory_bus.lock().unwrap();
        memory_bus.insert_cartridge(cartridge);
        trace!("{:#x}", memory_bus.read_u8(0x100)?);
        Ok(())
    }

    pub fn tick(&mut self) {
        let elapsed_cycles = self.cpu.lock().unwrap().step(&self).expect("Error while stepping cpu");
        self.timer.tick(elapsed_cycles);
        
        // If data exists on the serial port, forward it to the observer
        let serial_port_data = &mut self.memory_bus.lock().unwrap().serial_port_data;
        if let Some(observer) = self.serial_port_observer {
            for chr in serial_port_data.drain(..) {
                observer(chr);
            }
        }
    }

    pub fn on_serial_port_data(&mut self, observer: Observer) {
        self.serial_port_observer = Some(observer);
    }
}

pub enum Interrupt {
    Timer,
}
