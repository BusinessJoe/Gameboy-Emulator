use crate::apu::Apu;
use crate::cartridge::{self, Cartridge};
use crate::component::{Addressable, Steppable};
use crate::cpu::Cpu;
use crate::error::Result;
use crate::interrupt::Interrupt;
use crate::joypad::{Joypad, JoypadInput};
use crate::memory::MemoryBus;
use crate::ppu::palette::TileColor;
use crate::ppu::BasePpu;
use crate::timer::Timer;
use core::fmt;
use js_sys::Uint8Array;
use log::trace;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
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
    pub(crate) cpu: Cpu,
    pub(crate) memory_bus: MemoryBus,

    elapsed_since_ppu_step: u32,
    next_ppu_step: u32,
}

impl GameBoyState {
    pub fn get_pc(&self) -> u16 {
        self.cpu.pc
    }

    pub fn load(&mut self, filename: &str) -> Result<()> {
        let bytes = fs::read(filename).unwrap();
        let cartridge = cartridge::Cartridge::cartridge_from_data(&bytes).unwrap();
        self.load_cartridge(cartridge)
    }

    pub fn load_cartridge(&mut self, cartridge: Cartridge) -> Result<()> {
        // just reset the entire gameboy by rebuilding it
        println!("Loaded cartridge: {:?}", cartridge);

        let cpu = Cpu::new();
        let ppu = BasePpu::new();
        let apu = Apu::new();
        let joypad = Joypad::new();
        let timer = Timer::new();
        let memory_bus = MemoryBus::new(
            ppu,
            apu,
            joypad,
            timer,
        );

        self.cpu = cpu;
        // self.ppu = ppu.clone();
        // self.apu = apu.clone();
        // self.joypad = joypad;
        // self.timer = timer;
        self.memory_bus = memory_bus;

        self.memory_bus.insert_cartridge(cartridge);
        trace!("{:#x}", self.memory_bus.read_u8(0x100)?);
        Ok(())
    }

    pub fn tick(&mut self) -> u32 {
        let elapsed_cycles = self
            .cpu
            .step(&mut self.memory_bus, 4)
            .map_err(|e| println!("{}", e))
            .unwrap();

        {
            // let mut ppu = self.ppu.borrow_mut();
            // let mut timer = self.timer.borrow_mut();
            // let mut apu = self.apu.borrow_mut();

            for _ in 0..elapsed_cycles {
                // Timer, and apu step each T-cycle
                self.memory_bus.timer.step(&mut self.memory_bus.interrupt_regs, 1).expect("error while stepping timer");

                self.memory_bus.apu.tick(self.memory_bus.timer.get_div());

                self.elapsed_since_ppu_step += 1;

                if self.elapsed_since_ppu_step == self.next_ppu_step {
                    self.next_ppu_step = self.memory_bus.ppu
                        .step(&mut self.memory_bus.interrupt_regs, self.elapsed_since_ppu_step)
                        .expect("error while stepping ppu");
                    self.elapsed_since_ppu_step = 0;
                }
            }
            trace!("stepped ppu and timer for {} M-cycles", elapsed_cycles);
        }

        // Return T-cycles
        4 * elapsed_cycles
    }

    pub fn debug_info(&mut self) -> GameboyDebugInfo {
        let cpu = &self.cpu;

        let register_f = [
            cpu.registers.f.zero,
            cpu.registers.f.subtract,
            cpu.registers.f.half_carry,
            cpu.registers.f.carry,
        ];

        let opcode = self.memory_bus.read_u8(cpu.pc.into()).unwrap();

        GameboyDebugInfo {
            pc: cpu.pc,
            opcode,
            sp: cpu.sp,
            register_a: cpu.registers.a,
            register_f,
            register_bc: u16::from_le_bytes([cpu.registers.b, cpu.registers.c]),
            register_de: u16::from_le_bytes([cpu.registers.d, cpu.registers.e]),
            register_hl: u16::from_le_bytes([cpu.registers.h, cpu.registers.l]),
            mem_tima_ff05: self.memory_bus.read_u8(0xff05).unwrap(),
            interrupt_enabled: cpu.interrupt_enabled,
        }
    }

    pub fn get_cpu(&self) -> &Cpu {
        &self.cpu
    }

    pub fn get_cpu_mut(&mut self) -> &mut Cpu {
        &mut self.cpu
    }

    pub fn get_joypad(&self) -> &Joypad {
        &self.memory_bus.joypad
    }

    pub fn get_joypad_mut(&mut self) -> &mut Joypad {
        &mut self.memory_bus.joypad
    }

    pub fn get_memory_bus(&self) -> &MemoryBus {
        &self.memory_bus
    }

    pub fn get_memory_bus_mut(&mut self) -> &mut MemoryBus {
        &mut self.memory_bus
    }

    pub fn get_ppu(&self) -> &BasePpu {
        &self.memory_bus.ppu
    }

    pub fn get_screen(&self) -> Vec<TileColor> {
        self.memory_bus.ppu.get_screen()
    }

    pub fn get_screen_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        let screen_data: Vec<u8> = self.get_screen().iter().map(|c| c.to_u8()).collect();
        Hash::hash_slice(&screen_data, &mut hasher);
        let hash = hasher.finish();

        hash
    }

    pub fn press_joypad_input(&mut self, joypad_input: JoypadInput) {
        let prev_state = self.memory_bus.joypad.key_pressed(joypad_input);
        // If previous state was not pressed, we send interrupt
        if !prev_state {
            self.memory_bus
                .interrupt_regs
                .interrupt(Interrupt::Joypad);
        }
    }

    pub fn release_joypad_input(&mut self, joypad_input: JoypadInput) {
        self.memory_bus.joypad.key_released(joypad_input);
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

#[wasm_bindgen]
impl GameBoyState {
    pub fn new() -> Self {
        console_error_panic_hook::set_once();

        let ppu = BasePpu::new();
        let apu = Apu::new();
        let joypad = Joypad::new();
        let timer = Timer::new();
        let memory_bus = MemoryBus::new(
            ppu,
            apu,
            joypad,
            timer,
        );
        Self {
            cpu: Cpu::new(),
            // ppu,
            // apu,
            // joypad,
            // timer,
            memory_bus,

            elapsed_since_ppu_step: 0,
            // LCD starts in OAM search, which lasts for 80 dots
            next_ppu_step: 80,
        }
    }

    pub fn load_rom_web(&mut self, array: Uint8Array) -> std::result::Result<(), JsValue> {
        let bytes: Vec<u8> = array.to_vec();
        let cartridge = Cartridge::cartridge_from_data(&bytes)
            .ok_or_else(|| JsValue::from_str("failed to build cartridge"))?;
        self.load_cartridge(cartridge)
            .map_err(|err| format!("failed to load cartridge: {}", err))?;
        Ok(())
    }

    pub fn get_web_screen(&self) -> Uint8Array {
        let colors = self.get_screen();
        let colors: Vec<u8> = colors.iter().map(|c| c.to_u8()).collect();
        let array = Uint8Array::new_with_length(colors.len() as u32);
        array.copy_from(&colors);
        array
    }

    pub fn tick_for_frame(&mut self) -> u32 {
        let mut elapsed_cycles = 0;
        // let old_frame_count = self.ppu.borrow().get_frame_count();
        let old_frame_count = self.memory_bus.ppu.get_frame_count();
        loop {
            //println!("{:?}", self.debug_info());
            elapsed_cycles += self.tick();

            // if self.ppu.borrow().get_frame_count() > old_frame_count {
            if self.memory_bus.ppu.get_frame_count() > old_frame_count {
                break;
            }
        }
        elapsed_cycles
    }

    /**
     * Maps the provided u8 to joypad inputs as shown:
     *   0 => A
     *   1 => B
     *   2 => Start
     *   3 => Select
     *   4 => Left
     *   5 => Right
     *   6 => Up
     *   7 => Down
     * Panics on other values.
     */
    fn map_u8_to_joypad_input(key: u8) -> JoypadInput {
        match key {
            0 => JoypadInput::A,
            1 => JoypadInput::B,
            2 => JoypadInput::Start,
            3 => JoypadInput::Select,
            4 => JoypadInput::Left,
            5 => JoypadInput::Right,
            6 => JoypadInput::Up,
            7 => JoypadInput::Down,
            _ => panic!("unexpected key"),
        }
    }

    pub fn press_key(&mut self, key: u8) {
        let joypad_input = Self::map_u8_to_joypad_input(key);
        self.press_joypad_input(joypad_input);
    }

    pub fn release_key(&mut self, key: u8) {
        let joypad_input = Self::map_u8_to_joypad_input(key);
        self.release_joypad_input(joypad_input);
    }

    pub fn game_name(&self) -> Option<String> {
        if let Some(cartridge) = &self.memory_bus.cartridge {
            if let Some(cartridge_type) = &cartridge.cartridge_type {
                return Some(cartridge_type.title.clone());
            }
        }

        None
    }

    pub fn saves_available(&self) -> bool {
        if let Some(cartridge) = &self.memory_bus.cartridge {
            if let Some(cartridge_type) = &cartridge.cartridge_type {
                return cartridge_type.has_battery;
            }
        }

        false
    }

    pub fn get_save(&self) -> Option<Uint8Array> {
        if let Some(cartridge) = &self.memory_bus.cartridge {
            let array = Uint8Array::new_with_length(cartridge.ram.len() as u32);
            array.copy_from(&cartridge.ram);
            Some(array)
        } else {
            None
        }
    }

    pub fn load_save(&mut self, ram: Uint8Array) -> bool {
        let ram: Vec<u8> = ram.to_vec();

        if let Some(cartridge) = &mut self.memory_bus.cartridge {
            if cartridge.ram.len() == ram.len() {
                cartridge.ram = ram;
                return true;
            } else {
                return false;
            }
        }

        false
    }

    pub fn get_queued_audio(&mut self) -> Vec<f32> {
        self.memory_bus.apu.get_queued_audio()
    }
}
