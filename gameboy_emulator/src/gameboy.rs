use crate::cartridge::{self, Cartridge};
use crate::component::{Addressable, Steppable};
use crate::cpu::CPU;
use crate::emulator::events::EmulationEvent;
use crate::error::Result;
use crate::joypad::Joypad;
use crate::memory::MemoryBus;
use crate::ppu::palette::TileColor;
use crate::ppu::BasePpu;
use crate::timer::Timer;
use core::fmt;
use js_sys::{Array, Uint8Array};
use log::trace;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::fs;
use std::rc::Rc;
use std::sync::mpsc::Sender;
use wasm_bindgen::prelude::*;

pub type Observer = Box<dyn FnMut(u8)>;

#[derive(Debug, Clone)]
pub struct GameboyDebugInfo {
    pub pc: u16,
    pub opcode: u8, // opcode at pc
    pub sp: u16,
    pub register_a: u8,
    pub register_f: [bool; 4],
    pub register_bc: u16,
    pub register_de: u16,
    pub register_hl: u16,
    pub mem_tima_ff05: u8,
    pub interrupt_enabled: bool,
}

#[wasm_bindgen]
pub struct GameBoyState {
    pub(crate) cpu: Rc<RefCell<CPU>>,
    pub(crate) ppu: Rc<RefCell<BasePpu>>,
    pub(crate) joypad: Rc<RefCell<Joypad>>,
    pub(crate) timer: Rc<RefCell<Timer>>,
    pub(crate) memory_bus: Rc<RefCell<MemoryBus>>,
    emulation_event_sender: Option<Sender<EmulationEvent>>,
    pub(crate) event_queue: VecDeque<EmulationEvent>,
}

impl GameBoyState {
    pub fn new(emulation_event_sender: Sender<EmulationEvent>) -> Self {
        let ppu = Rc::new(RefCell::new(BasePpu::new()));
        let joypad = Rc::new(RefCell::new(Joypad::new()));
        let timer = Rc::new(RefCell::new(Timer::new()));
        let memory_bus = Rc::new(RefCell::new(MemoryBus::new(
            ppu.clone(),
            joypad.clone(),
            timer.clone(),
            Some(emulation_event_sender.clone()),
        )));
        Self {
            cpu: Rc::new(RefCell::new(CPU::new())),
            ppu: ppu.clone(),
            joypad,
            timer,
            memory_bus: memory_bus.clone(),
            emulation_event_sender: Some(emulation_event_sender),
            event_queue: VecDeque::new(),
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
        self.emulation_event(EmulationEvent::Trace(self.debug_info()));

        let elapsed_cycles = self
            .cpu
            .borrow_mut()
            .step(&self)
            .map_err(|e| println!("{}", e))
            .unwrap();

        {
            let mut ppu = self.ppu.borrow_mut();
            let mut timer = self.timer.borrow_mut();
            for _ in 0..elapsed_cycles {
                // Timer and ppu step each T-cycle
                for _ in 0..4 {
                    ppu.step(&self).expect("error while stepping ppu");
                    timer.step(&self).expect("error while stepping timer");
                }
            }
            trace!("stepped ppu and timer for {} M-cycles", elapsed_cycles);
        }

        // If data exists on the serial port, output it as an emulation event
        let serial_port_data = &mut self.memory_bus.borrow_mut().serial_port_data.split_off(0);
        for byte in serial_port_data.drain(..) {
            self.emulation_event(EmulationEvent::SerialData(byte));
        }

        // Return T-cycles
        4 * elapsed_cycles
    }

    pub fn emulation_event(&mut self, event: EmulationEvent) {
        // store most recent n events
        /*
        if self.event_queue.len() > 1_000_000 {
            self.event_queue.pop_front();
        }
        self.event_queue.push_back(event.clone());
        */
        if let Some(sender) = &self.emulation_event_sender {
            sender.send(event).unwrap();
        }
    }

    pub fn debug_info(&self) -> GameboyDebugInfo {
        let cpu = self.cpu.borrow();

        let register_f = [
            cpu.registers.f.zero,
            cpu.registers.f.subtract,
            cpu.registers.f.half_carry,
            cpu.registers.f.carry,
        ];

        let opcode = self.memory_bus.borrow_mut().read_u8(cpu.pc.into()).unwrap();

        GameboyDebugInfo {
            pc: cpu.pc,
            opcode,
            sp: cpu.sp,
            register_a: cpu.registers.a,
            register_f,
            register_bc: u16::from_le_bytes([cpu.registers.b, cpu.registers.c]),
            register_de: u16::from_le_bytes([cpu.registers.d, cpu.registers.e]),
            register_hl: u16::from_le_bytes([cpu.registers.h, cpu.registers.l]),
            mem_tima_ff05: self.memory_bus.borrow_mut().read_u8(0xff05).unwrap(),
            interrupt_enabled: cpu.interrupt_enabled,
        }
    }

    pub fn get_cpu(&self) -> Rc<RefCell<CPU>> {
        self.cpu.clone()
    }

    pub fn get_joypad(&self) -> Rc<RefCell<Joypad>> {
        self.joypad.clone()
    }

    pub fn get_memory_bus(&self) -> Rc<RefCell<MemoryBus>> {
        self.memory_bus.clone()
    }

    pub fn get_ppu(&self) -> Rc<RefCell<BasePpu>> {
        self.ppu.clone()
    }

    pub fn get_screen(&self) -> Vec<TileColor> {
        self.ppu.borrow().get_screen().to_vec()
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

#[wasm_bindgen]
impl GameBoyState {
    pub fn new_web() -> Self {
        console_error_panic_hook::set_once();

        let ppu = Rc::new(RefCell::new(BasePpu::new()));
        let joypad = Rc::new(RefCell::new(Joypad::new()));
        let timer = Rc::new(RefCell::new(Timer::new()));
        let memory_bus = Rc::new(RefCell::new(MemoryBus::new(
            ppu.clone(),
            joypad.clone(),
            timer.clone(),
            None,
        )));
        Self {
            cpu: Rc::new(RefCell::new(CPU::new())),
            ppu: ppu.clone(),
            joypad,
            timer,
            memory_bus: memory_bus.clone(),
            emulation_event_sender: None,
            event_queue: VecDeque::new(),
        }
    }

    pub fn load_zelda(&mut self) {
        let bytes = include_bytes!("../../roms/Legend of Zelda, The - Link's Awakening (USA, Europe).gb");
        let cartridge = Cartridge::cartridge_from_data(bytes).expect("failed to build cartridge");
        self.load_cartridge(cartridge).unwrap();
    }

    pub fn get_web_screen(&self) -> Uint8Array {
        let colors = self.get_screen();
        let colors: Vec<u8> = colors.iter().map(|c| c.to_u8()).collect();
        let mut array = Uint8Array::new_with_length(colors.len() as u32);
        array.copy_from(&colors);
        array
    }

    pub fn tick_for_frame(&mut self) -> u64 {
        let mut elapsed_cycles = 0;
        let old_frame_count = self.ppu.borrow().get_frame_count();
        loop {
            elapsed_cycles += self.tick();

            // Done frame at start of VBLANK
            if self.ppu.borrow().get_frame_count() > old_frame_count {
                break;
            }
        }
        elapsed_cycles
    }
}
