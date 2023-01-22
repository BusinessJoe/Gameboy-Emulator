use crate::cartridge::{self, Cartridge};
use crate::component::{Addressable, Steppable};
use crate::cpu::CPU;
use crate::error::Result;
use crate::joypad::Joypad;
use crate::memory::MemoryBus;
use crate::ppu::Ppu;
use crate::timer::Timer;
use log::trace;
use sdl2::render::Canvas;
use std::cell::RefCell;
use std::fs;
use std::rc::Rc;

const CLOCK_SPEED: u64 = 4_194_304;

pub type Observer = fn(chr: char);

pub struct GameBoyState<'a> {
    pub cpu: Rc<RefCell<CPU>>,
    pub ppu: Rc<RefCell<dyn Ppu<'a> + 'a>>,
    pub joypad: Rc<RefCell<Joypad>>,
    pub memory_bus: Rc<RefCell<MemoryBus<'a>>>,
    serial_port_observer: Option<Observer>,
    timer: Timer<'a>,
}

impl<'a> GameBoyState<'a> {
    pub fn new(ppu: Rc<RefCell<dyn Ppu<'a> + 'a>>) -> Self {
        let joypad = Rc::new(RefCell::new(Joypad::new()));
        let memory_bus = Rc::new(RefCell::new(MemoryBus::new(ppu.clone(), joypad.clone())));
        Self {
            cpu: Rc::new(RefCell::new(CPU::new())),
            ppu: ppu.clone(),
            joypad,
            memory_bus: memory_bus.clone(),
            serial_port_observer: None,
            timer: Timer::new(CLOCK_SPEED, memory_bus),
        }
    }

    pub fn load(&mut self, filename: &str) -> Result<()> {
        let bytes = fs::read(filename).unwrap();
        let cartridge = cartridge::build_cartridge(&bytes).unwrap();
        self.load_cartridge(cartridge)
    }

    pub fn load_cartridge(&mut self, cartridge: Box<dyn Cartridge>) -> Result<()> {
        println!("Loaded cartridge: {:?}", cartridge);
        let mut memory_bus = self.memory_bus.borrow_mut();
        memory_bus.insert_cartridge(cartridge);
        trace!("{:#x}", memory_bus.read_u8(0x100)?);
        Ok(())
    }

    pub fn tick(&mut self) -> u64 {
        let elapsed_cycles = self
            .cpu
            .borrow_mut()
            .step(&self)
            .expect("error while stepping cpu");
        for _ in 0..elapsed_cycles {
            self.ppu
                .borrow_mut()
                .step(&self)
                .expect("error while stepping ppu");
        }
        self.timer.tick(elapsed_cycles);

        // If data exists on the serial port, forward it to the observer
        let serial_port_data = &mut self.memory_bus.borrow_mut().serial_port_data;
        if let Some(observer) = self.serial_port_observer {
            for chr in serial_port_data.drain(..) {
                observer(chr);
            }
        }

        elapsed_cycles
    }

    pub fn on_serial_port_data(&mut self, observer: Observer) {
        self.serial_port_observer = Some(observer);
    }
}

pub enum Interrupt {
    VBlank,
    Timer,
    Joypad,
}
