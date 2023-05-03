use crate::{
    component::{Address, Addressable, ElapsedTime, Steppable},
    interrupt::InterruptRegs,
};
use crate::{Error, Result};

use super::{
    lcd::{self},
    palette::TileColor,
};
use super::{
    lcd::{PlacePixel, PpuScanlineState},
    palette::PaletteRegister,
    renderer::Renderer,
};

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

    pub background_palette: PaletteRegister,
    pub object_palette_0: PaletteRegister,
    pub object_palette_1: PaletteRegister,

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
            background_palette: PaletteRegister { register_value: 0 },
            object_palette_0: PaletteRegister { register_value: 0 },
            object_palette_1: PaletteRegister { register_value: 0 },

            scy: 0,
            scx: 0,
            wy: 0,
            wx: 0,
        }
    }

    fn read(&mut self, address: Address) -> Result<u8> {
        let value = match address {
            // VRAM is disabled while LCD enabled during pixel transfer
            0x8000..=0x9fff
                if self.lcd.lcd_control.lcd_ppu_enable
                    && self.lcd.state == PpuScanlineState::PixelTransfer =>
            {
                dbg!(address);
                0xff
            }
            0x8000..=0x97ff => self.tile_data[address - 0x8000],
            0x9800..=0x9bff => self.background_map[address - 0x9800],
            0x9c00..=0x9fff => self.window_map[address - 0x9c00],

            // OAM is disabled while LCD enabled during OAM search and pixel transfer
            0xfe00..=0xfe9f
                if self.lcd.lcd_control.lcd_ppu_enable
                    && (self.lcd.state == PpuScanlineState::OamSearch)
                        | (self.lcd.state == PpuScanlineState::PixelTransfer) =>
            {
                0xff
            }
            0xfe00..=0xfe9f => self.sprite_tiles_table[address - 0xfe00],

            0xff40 => self.lcd.lcd_control.read(),
            0xff41 => self.lcd.stat.0,
            0xff42 => self.scy,
            0xff43 => self.scx,
            0xff44 => self.lcd.ly,
            0xff45 => self.lcd.lyc,
            0xff47 => self.background_palette.register_value,
            0xff48 => self.object_palette_0.register_value,
            0xff49 => self.object_palette_1.register_value,
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
            // VRAM is disabled while LCD enabled during pixel transfer
            0x8000..=0x9fff
                if self.lcd.lcd_control.lcd_ppu_enable
                    && self.lcd.state == PpuScanlineState::PixelTransfer =>
            {
                //println!("write to address {:#x} ignored", address);
            }
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

            // OAM is disabled while LCD enabled during OAM search and pixel transfer
            0xfe00..=0xfe9f
                if self.lcd.lcd_control.lcd_ppu_enable
                    && (self.lcd.state == PpuScanlineState::OamSearch)
                        | (self.lcd.state == PpuScanlineState::PixelTransfer) => {}
            0xfe00..=0xfe9f => {
                self.sprite_tiles_table[address - 0xfe00] = data;
            }

            0xff40 => self.lcd.lcd_control.write(data),
            0xff41 => self.lcd.stat.0 = data,
            0xff42 => self.scy = data,
            0xff43 => self.scx = data,
            0xff44 => (), // ly: lcd y coordinate is read only
            0xff45 => self.lcd.lyc = data,
            0xff47 => self.background_palette.register_value = data,
            0xff48 => self.object_palette_0.register_value = data,
            0xff49 => self.object_palette_1.register_value = data,
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

/// Decoded tile data which is stored as a vec of 64 integers from 0 to 3
#[derive(Debug, Clone)]
pub struct Tile(pub Vec<u8>);
impl Tile {
    pub fn new() -> Tile {
        Tile(vec![0; 64])
    }

    pub fn get_pixel(&self, x: u8, y: u8) -> u8 {
        self.0[(x + 8 * y) as usize]
    }
}

pub struct BasePpu {
    pub(crate) state: PpuState,
    renderer: Renderer,
}

impl BasePpu {
    pub fn new() -> Self {
        Self {
            state: PpuState::new(),
            renderer: Renderer::new(),
        }
    }

    pub fn get_screen(&self) -> Vec<TileColor> {
        if self.state.lcd.lcd_control.lcd_ppu_enable {
            self.renderer.screen_pixels.clone()
        } else {
            vec![TileColor::White; 160 * 144]
        }
    }

    pub fn get_frame_count(&self) -> u128 {
        self.state.lcd.frame_count
    }

    pub fn oam_transfer(&mut self, data: &[u8]) {
        self.state.sprite_tiles_table.copy_from_slice(data);
    }

    fn place_pixel(&mut self, x: u8, y: u8) -> Result<()> {
        if x == 0 {
            self.renderer.update_scanline_cache(&self.state, y);
        }

        self.renderer
            .cache_lcdc(self.state.lcd.lcd_control, x.into());
        if x == 159 {
            self.renderer.render_scanline(&self.state, y);
        }
        Ok(())
    }
}

impl Steppable for BasePpu {
    type Context = InterruptRegs;

    fn step(&mut self, interrupt_regs: &mut Self::Context, elapsed: u32) -> Result<ElapsedTime> {
        let step_result =
            self.state
                .lcd
                .step(elapsed, interrupt_regs, self.state.wx, self.state.wy)?;
        if let Some(PlacePixel { x, y }) = step_result.pixel {
            self.place_pixel(x, y)?;
        }
        Ok(step_result.sleep.into())
    }
}

impl Addressable for BasePpu {
    fn read_u8(&mut self, address: Address) -> Result<u8> {
        self.state.read(address)
    }

    fn write_u8(&mut self, address: Address, data: u8) -> Result<()> {
        self.state.write(address, data)?;
        match address {
            0x8000..=0x97ff => {
                self.renderer
                    .update_tile_cache(&self.state.tile_data, address);
            }
            _ => {}
        }
        Ok(())
    }
}
