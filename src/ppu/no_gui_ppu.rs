use log::trace;

use crate::error::{Error, Result};
use crate::{
    component::{Address, Addressable, ElapsedTime, Steppable},
    gameboy::GameBoyState,
    ppu::Ppu,
};

use super::{canvas_ppu::Tile, lcd};

/// A Ppu without an attached gui
pub struct NoGuiPpu {
    /// Tile data takes up addresses 0x8000-0x97ff.
    tile_data: Vec<u8>,

    /// Cache of decoded tile data -- the gameboy can store 384 different tiles
    tile_cache: Vec<Tile>,
    /// Addresses 0x9800-0x9bff are a 32x32 map of background tiles.
    /// Each byte contains the number of a tile to be displayed.
    background_map: Vec<u8>,

    /// A table containing data for 40 sprites
    sprite_tiles_table: Vec<u8>,

    lcd: lcd::Lcd,
}

impl NoGuiPpu {
    pub fn new() -> NoGuiPpu {
        NoGuiPpu {
            tile_data: vec![0; 0x1800],
            // The gameboy has room for 384 tiles in addresses 0x8000 to 0x97ff
            tile_cache: vec![Tile::new(); 384],
            background_map: vec![0; 32 * 32],
            sprite_tiles_table: vec![0; 160],
            lcd: lcd::Lcd::new(),
        }
    }

    fn _read(&mut self, address: Address) -> Result<u8> {
        let value = match address {
            0x8000..=0x97ff => self.tile_data[address - 0x8000],
            0x9800..=0x9bff => self.background_map[address - 0x9800],
            0xfe00..=0xfe9f => self.sprite_tiles_table[address - 0xfe00],
            0xff40 => self.lcd.lcd_control.read(),
            0xff41 => self.lcd.stat.0,
            0xff44 => self.lcd.ly,
            0xff45 => self.lcd.lyc,
            _ => return Err(Error::new("Invalid address")),
        };

        Ok(value)
    }

    fn _write(&mut self, address: Address, data: u8) -> Result<()> {
        match address {
            0x8000..=0x97ff => {
                trace!("write to tile data: {:#x} into {:#x}", data, address);
                self.tile_data[address - 0x8000] = data;
            }
            0x9800..=0x9bff => {
                self.background_map[address - 0x9800] = data;
            }
            0xfe00..=0xfe9f => {
                self.sprite_tiles_table[address - 0xfe00] = data;
            }
            0xff40 => self.lcd.lcd_control.write(data),
            0xff41 => self.lcd.stat.0 = data,
            0xff45 => self.lcd.lyc = data,
            _ => return Err(Error::new("Invalid address")),
        }

        Ok(())
    }
}

impl Addressable for NoGuiPpu {
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

impl Steppable for NoGuiPpu {
    fn step(&mut self, state: &GameBoyState) -> Result<ElapsedTime> {
        self.lcd.step(state)
    }
}

impl<'a> Ppu<'a> for NoGuiPpu {}
