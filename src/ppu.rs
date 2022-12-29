/*!
 * This PPU serves as an implementation for all the gameboy's graphics. It maintains an internal
 * representation of the screen.
 */
use std::sync::{Arc, Mutex};

use winit::{event_loop::EventLoop, platform::run_return::EventLoopExtRunReturn};

use crate::{
    memory::MemoryBus,
    screen::{PixelsScreen, Screen},
};

#[derive(Debug)]
pub struct PPU {
    memory_bus: Arc<Mutex<MemoryBus>>,
    pub screen: Vec<u8>,
    index: usize,
}

impl PPU {
    pub fn new(memory_bus: Arc<Mutex<MemoryBus>>) -> Self {
        let ppu = Self {
            memory_bus,
            screen: vec![0; 8 * 8],
            index: 0
        };
        ppu
    }

    pub fn tick(&mut self) {
        return;
        dbg!(self.index);
        let tile_pixels = self.get_tile(self.index);
        self.index = (self.index + 1) % 256;
        self.screen[..].copy_from_slice(&tile_pixels[..]);
    }

    pub fn get_tile(&self, tile_index: usize) -> Vec<u8> {
        let memory_bus = self.memory_bus.lock().unwrap();
        let mut output = Vec::with_capacity(64);

        let start_address = 0x0000 + 16 * tile_index;
        for line_address_1 in (start_address..(start_address + 16)).step_by(2) {
            let line_address_2 = line_address_1 + 1;

            let byte_1 = memory_bus.get(line_address_1);
            let byte_2 = memory_bus.get(line_address_2);

            for i in 0..8 {
                let bit_1 = (byte_1 >> i) & 1;
                let bit_2 = (byte_2 >> i) & 1;
                let color_id = bit_2 << 1 | bit_1;
                output.push(color_id);
            }
        }

        assert_eq!(64, output.len());
        output
    }
}
