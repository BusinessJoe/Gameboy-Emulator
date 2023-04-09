mod component;
mod error;

mod bit_field;
pub mod cartridge;
pub mod cpu;
pub mod emulator;
pub mod gameboy;
pub mod joypad;
mod memory;
pub mod ppu;
mod timer;
mod utils;

pub use error::{Error, Result};
pub use memory::MemoryBus;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn greet() {
    alert("Hello, {{project-name}}!");
}
