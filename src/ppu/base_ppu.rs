use crate::texture::TextureBook;
use crate::{
    component::{Address, Addressable, ElapsedTime, Steppable},
    gameboy::GameBoyState,
};
use crate::{Error, Result};

use super::lcd;

pub trait GraphicsEngine {
    fn after_write(&mut self, ppu_state: &PpuState, address: Address);

    fn render(
        &mut self,
        ppu_state: &PpuState,
        canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
        texture_book: &mut TextureBook,
    ) -> Result<()>;
}

pub struct PpuState {
    /// Tile data takes up addresses 0x8000-0x97ff.
    pub tile_data: Vec<u8>,
    /// Addresses 0x9800-0x9bff are a 32x32 map of background tiles.
    /// Each byte contains the number of a tile to be displayed.
    pub background_map: Vec<u8>,
    /// Addresses 0x9c00-0x9fff are a 32x32 map of window tiles.
    /// Each byte contains the number of a tile to be displayed.
    pub window_map: Vec<u8>,

    /// A table containing data for 40 sprites
    pub sprite_tiles_table: Vec<u8>,

    pub lcd: lcd::Lcd,

    /// Register values
    pub scy: u8,
    pub scx: u8,
    pub wy: u8,
    pub wx: u8,
}

impl PpuState {
    pub fn new() -> Self {
        Self {
            tile_data: vec![0; 0x1800],
            background_map: vec![0; 32 * 32],
            window_map: vec![0; 32 * 32],
            sprite_tiles_table: vec![0; 160],
            lcd: lcd::Lcd::new(),

            scy: 0,
            scx: 0,
            wy: 0,
            wx: 0,
        }
    }

    fn read(&mut self, address: Address) -> Result<u8> {
        let value = match address {
            0x8000..=0x97ff => self.tile_data[address - 0x8000],
            0x9800..=0x9bff => self.background_map[address - 0x9800],
            0x9c00..=0x9fff => self.window_map[address - 0x9c00],
            0xfe00..=0xfe9f => self.sprite_tiles_table[address - 0xfe00],
            0xff40 => self.lcd.lcd_control.read(),
            0xff41 => self.lcd.stat.0,
            0xff42 => self.scy,
            0xff43 => self.scx,
            0xff44 => self.lcd.ly,
            0xff45 => self.lcd.lyc,
            0xff4a => self.wy,
            0xff4b => self.wx,
            _ => {
                return Err(Error::from_address_with_source(
                    address,
                    "ppu read".to_string(),
                ))
            }
        };

        Ok(value)
    }

    fn write(&mut self, address: Address, data: u8) -> Result<()> {
        match address {
            0x8000..=0x97ff => {
                log::trace!("write to tile data: {:#x} into {:#x}", data, address);
                self.tile_data[address - 0x8000] = data;
            }
            0x9800..=0x9bff => {
                self.background_map[address - 0x9800] = data;
            }
            0x9c00..=0x9fff => {
                self.window_map[address - 0x9c00] = data;
            }
            0xfe00..=0xfe9f => {
                self.sprite_tiles_table[address - 0xfe00] = data;
            }
            0xff40 => self.lcd.lcd_control.write(data),
            0xff41 => self.lcd.stat.0 = data,
            0xff42 => self.scy = data,
            0xff43 => self.scx = data,
            0xff44 => (), // ly: lcd y coordinate is read only
            0xff45 => self.lcd.lyc = data,
            0xff4a => self.wy = data,
            0xff4b => self.wx = data,
            _ => {
                return Err(Error::from_address_with_source(
                    address,
                    "ppu write".to_string(),
                ))
            }
        }

        Ok(())
    }
}

pub struct BasePpu {
    pub(super) state: PpuState,
    graphics_engine: Box<dyn GraphicsEngine>,
}

impl BasePpu {
    pub fn new(graphics_engine: Box<dyn GraphicsEngine>) -> Self {
        Self {
            state: PpuState::new(),
            graphics_engine,
        }
    }

    pub fn render(
        &mut self,
        canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
        texture_book: &mut TextureBook,
    ) -> Result<()> {
        self.graphics_engine
            .render(&self.state, canvas, texture_book)
    }
}

impl Steppable for BasePpu {
    fn step(&mut self, state: &GameBoyState) -> Result<ElapsedTime> {
        self.state.lcd.step(state)
    }
}

impl Addressable for BasePpu {
    fn read(&mut self, address: Address, data: &mut [u8]) -> Result<()> {
        for (offset, byte) in data.iter_mut().enumerate() {
            *byte = self.state.read(address + offset)?;
        }

        Ok(())
    }

    fn write(&mut self, address: Address, data: &[u8]) -> Result<()> {
        for (offset, byte) in data.iter().enumerate() {
            self.state.write(address + offset, *byte)?;
            self.graphics_engine
                .after_write(&self.state, address + offset)
        }

        Ok(())
    }
}
