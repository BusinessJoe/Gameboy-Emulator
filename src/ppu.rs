/*!
 * This PPU serves as an implementation for all the gameboy's graphics. It maintains an internal
 * representation of the screen.
 */

use crate::{
    component::{ElapsedTime, Addressable, Steppable},
    error::Result,
    gameboy::GameBoyState,
    memory::MemoryBus,
    screen::{PixelsScreen, Screen},
};

#[derive(Debug)]
pub struct PPU {
    pub screen: Vec<u8>,
    index: usize,
}

impl PPU {
    pub fn new() -> Self {
        let ppu = Self {
            screen: vec![0; 8 * 8],
            index: 0,
        };
        ppu
    }

    pub fn get_tile(&self, memory_bus: &mut MemoryBus, tile_index: usize) -> Result<Vec<u8>> {
        let mut output = Vec::with_capacity(64);

        let start_address = 0x0000 + 16 * tile_index;
        for line_address_1 in (start_address..(start_address + 16)).step_by(2) {
            let line_address_2 = line_address_1 + 1;

            let byte_1 = memory_bus.read_u8(line_address_1)?;
            let byte_2 = memory_bus.read_u8(line_address_2)?;

            for i in 0..8 {
                let bit_1 = (byte_1 >> i) & 1;
                let bit_2 = (byte_2 >> i) & 1;
                let color_id = bit_2 << 1 | bit_1;
                output.push(color_id);
            }
        }

        assert_eq!(64, output.len());
        Ok(output)
    }
}

impl Steppable for PPU {
    fn step(&mut self, state: &GameBoyState) -> Result<ElapsedTime> {
        return Ok(1);
        dbg!(self.index);
        let memory_bus = state.memory_bus.lock().unwrap();
        let tile_pixels = self.get_tile(&mut memory_bus, self.index)?;
        self.index = (self.index + 1) % 256;
        self.screen[..].copy_from_slice(&tile_pixels[..]);
    }
}
