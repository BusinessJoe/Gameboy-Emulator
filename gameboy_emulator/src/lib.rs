mod component;
mod error;
mod mainloop;

mod bit_field;
pub mod cartridge;
pub mod cpu;
pub mod emulator;
pub mod gameboy;
mod joypad;
mod memory;
pub mod ppu;
mod timer;
mod utils;

#[cfg(not(target_arch = "wasm32"))]
mod sdl2;
#[cfg(target_arch = "wasm32")]
mod web;

pub use error::{Error, Result};
pub use joypad::Joypad;
pub use memory::MemoryBus;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn greet() {
    alert("Hello, {{project-name}}!");
}