/*!
 * This PPU serves as an implementation for all the gameboy's graphics. It maintains an internal
 * representation of the screen.
 */

use crate::{
    component::{ElapsedTime, Addressable, Address, Steppable},
    error::{Error, Result},
    gameboy::GameBoyState,
};

#[derive(Debug)]
pub struct PPU {
    pub screen: Vec<u8>,
    // Tile data takes up addresses 0x8000-0x97ff.
    tile_data: Vec<u8>,
    // Addresses 0x9800-0x9bff are a 32x32 map of background tiles. 
    // Each byte contains the number of a tile to be displayed.
    background_map: Vec<u8>,
}

const WIDTH: usize = 8 * 16;
const HEIGHT: usize = 8 * 16;

impl PPU {
    pub fn new() -> Self {
        let ppu = Self {
            screen: vec![0; WIDTH * HEIGHT],
            tile_data: vec![0; 0x1800],
            background_map: vec![0; 32 * 32],
        };
        ppu
    }

    pub fn get_tile(&self, tile_index: usize) -> Result<Vec<u8>> {
        let mut output = Vec::with_capacity(64);

        let start_address = 16 * tile_index;
        for line_address_1 in (start_address..(start_address + 16)).step_by(2) {
            let byte_1 = self.tile_data[line_address_1];
            let byte_2 = self.tile_data[line_address_1 + 1];

            for i in (0..8).rev() {
                let bit_1 = (byte_1 >> i) & 1;
                let bit_2 = (byte_2 >> i) & 1;
                let color_id = (bit_2 << 1) | bit_1;
                output.push(color_id);
            }
        }

        assert_eq!(64, output.len());
        Ok(output)
    }

    pub fn set_tile(&mut self, row: usize, col: usize, tile_index: usize) -> Result<()> {
        let tile_data = self.get_tile(tile_index)?;

        for row_offset in 0..8 {
            for col_offset in 0..8 {
                let row_idx = (row*8) + row_offset;
                let col_idx = (col*8) + col_offset;
                // Rows in the screen are 8*2 pixels wide
                self.screen[col_idx + row_idx * WIDTH] = tile_data[col_offset + row_offset * 8];
            }
        }

        Ok(())
    }

    fn _read(&mut self, address: Address) -> Result<u8> {
        let value = match address {
            0x8000..=0x97ff => {
                self.tile_data[address - 0x8000]
            },
            0x9800..=0x9bff => {
                self.background_map[address - 0x9800]
            }
            _ => return Err(Error::new("Invalid address"))
        };

        Ok(value)
    }

    fn _write(&mut self, address: Address, data: u8) -> Result<()> {
        match address {
            0x8000..=0x97ff => {
                self.tile_data[address - 0x8000] = data;
            },
            0x9800..=0x9bff => {
                self.background_map[address - 0x9800] = data;
            }
            _ => return Err(Error::new("Invalid address"))
        }

        Ok(())
    }
}

impl Steppable for PPU {
    fn step(&mut self, _state: &GameBoyState) -> Result<ElapsedTime> {
        for i in 0..16 {
            for j in 0..16 {
                self.set_tile(i, j, 16 * i + j)?;
            }
        }

        Ok(1)
    }
}

impl Addressable for PPU {
    fn read(&mut self, address: Address, data: &mut [u8]) -> Result<()> {
        for (offset, byte) in data.iter_mut().enumerate() {
            *byte = self._read(address + offset)?;
        }

        Ok(())
    }

    fn write(&mut self, address: Address, data: &[u8]) -> Result<()> {
        for (offset, byte) in data.iter().enumerate() {
            self._write(address + offset, *byte)?;
        }

        Ok(())
    }
}
