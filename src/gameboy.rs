use crate::cartridge::{self, Cartridge};
use crate::component::{Addressable, Steppable};
use crate::cpu::CPU;
use crate::emulator::events::EmulationEvent;
use crate::error::Result;
use crate::joypad::Joypad;
use crate::memory::MemoryBus;
use crate::ppu::Ppu;
use crate::timer::Timer;
use core::fmt;
use log::trace;
use std::cell::RefCell;
use std::fs;
use std::rc::Rc;
use std::sync::mpsc::Sender;

pub type Observer = Box<dyn FnMut(u8)>;

pub struct GameboyDebugInfo {
    pc: u16,
    sp: u16,
    register_a: u8,
    register_f: [bool; 4],
    register_bc: u16,
    register_de: u16,
    register_hl: u16,
}

pub struct GameBoyState<'a> {
    cpu: Rc<RefCell<CPU>>,
    pub ppu: Rc<RefCell<dyn Ppu<'a> + 'a>>,
    pub joypad: Rc<RefCell<Joypad>>,
    pub timer: Rc<RefCell<Timer>>,
    pub memory_bus: Rc<RefCell<MemoryBus<'a>>>,
    emulation_event_sender: Sender<EmulationEvent>
}

impl<'a> GameBoyState<'a> {
    pub fn new(ppu: Rc<RefCell<dyn Ppu<'a> + 'a>>, emulation_event_sender: Sender<EmulationEvent>) -> Self {
        let joypad = Rc::new(RefCell::new(Joypad::new()));
        let timer = Rc::new(RefCell::new(Timer::new()));
        let memory_bus = Rc::new(RefCell::new(MemoryBus::new(
            ppu.clone(),
            joypad.clone(),
            timer.clone(),
        )));
        Self {
            cpu: Rc::new(RefCell::new(CPU::new())),
            ppu: ppu.clone(),
            joypad,
            timer,
            memory_bus: memory_bus.clone(),
            emulation_event_sender,
        }
    }

    pub fn get_pc(&self) -> u16 {
        self.cpu.borrow().pc
    }

    pub fn load(&mut self, filename: &str) -> Result<()> {
        let bytes = fs::read(filename).unwrap();
        let cartridge = cartridge::Cartridge::cartridge_from_data(&bytes).unwrap();
        self.load_cartridge(cartridge)
    }

    pub fn load_cartridge(&mut self, cartridge: Cartridge) -> Result<()> {
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
        let elapsed_cycles = 4 * elapsed_cycles;
        {
            let mut ppu = self.ppu.borrow_mut();
            let mut timer = self.timer.borrow_mut();
            for _ in 0..elapsed_cycles {
                ppu.step(&self).expect("error while stepping ppu");
                timer.step(&self).expect("error while stepping ppu");
            }
            trace!("stepped ppu and timer for {} cycles", elapsed_cycles);
        }

        // If data exists on the serial port, forward it to the observer
        let serial_port_data = &mut self.memory_bus.borrow_mut().serial_port_data;
        for byte in serial_port_data.drain(..) {
            self.emulation_event(EmulationEvent::SerialData(byte));
        }

        elapsed_cycles
    }

    pub fn tick_components(&self) {
        self.ppu
            .borrow_mut()
            .step(&self)
            .expect("error while stepping ppu");
        self.timer
            .borrow_mut()
            .step(&self)
            .expect("error while stepping timer");
    }

    pub fn emulation_event(&self, event: EmulationEvent) {
        self.emulation_event_sender.send(event);
    }

    pub fn debug_info(&self) -> GameboyDebugInfo {
        let cpu = self.cpu.borrow();

        let register_f = [
            cpu.registers.f.zero,
            cpu.registers.f.subtract,
            cpu.registers.f.half_carry,
            cpu.registers.f.carry,
        ];

        GameboyDebugInfo {
            pc: cpu.pc,
            sp: cpu.sp,
            register_a: cpu.registers.a,
            register_f,
            register_bc: u16::from_le_bytes([cpu.registers.b, cpu.registers.c]),
            register_de: u16::from_le_bytes([cpu.registers.d, cpu.registers.e]),
            register_hl: u16::from_le_bytes([cpu.registers.h, cpu.registers.l]),
        }
    }
}

impl std::fmt::Display for GameboyDebugInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "pc: {:04x}, sp: {:04x}, A: {:02x}, F: {}{}{}{}, BC: {:04x}, DE: {:04x}, HL: {:04x}",
            self.pc,
            self.sp,
            self.register_a,
            if self.register_f[0] { 'Z' } else { '-' },
            if self.register_f[1] { 'S' } else { '-' },
            if self.register_f[2] { 'H' } else { '-' },
            if self.register_f[3] { 'C' } else { '-' },
            self.register_bc,
            self.register_de,
            self.register_hl,
        )
    }
}

pub enum Interrupt {
    VBlank,
    Stat,
    Timer,
    Joypad,
}
