/*!
 * This PPU serves as an implementation for all the gameboy's graphics. It maintains an internal
 * representation of the screen.
 */

use crate::{
    component::{Address, Addressable, ElapsedTime, Steppable},
    error::{Error, Result},
    gameboy::{GameBoyState, Interrupt},
};
use sdl2::{rect::Rect, render::TextureCreator, video::WindowContext};
use sdl2::{
    pixels::PixelFormatEnum,
    render::{RenderTarget, Texture},
};
use sdl2::video::Window;
use std::collections::VecDeque;
use log::trace;

#[derive(Debug, Clone, Copy)]
pub enum TileDataAddressingMethod {
    Method8000,
    Method8800,
}

/// Represents the LCD Control register at 0xff40
#[derive(Debug, Clone, Copy)]
pub struct LCDC {
    pub bg_window_enable: bool,
    pub obj_enable: bool,
    pub obj_size: bool,
    pub bg_tile_map_area: bool,
    pub bg_window_tile_data_area: bool,
    pub window_enable: bool,
    pub window_tile_map_area: bool,
    pub lcd_ppu_enable: bool,
}

impl LCDC {
    pub fn new() -> Self {
        Self {
            bg_window_enable: false,
            obj_enable: false,
            obj_size: false,
            bg_tile_map_area: false,
            bg_window_tile_data_area: false,
            window_enable: false,
            window_tile_map_area: false,
            lcd_ppu_enable: false,
        }
    }

    pub fn read(&self) -> u8 {
        (self.bg_window_enable as u8) + (self.obj_enable as u8)
            << 1 + (self.obj_size as u8)
            << 2 + (self.bg_tile_map_area as u8)
            << 3 + (self.bg_window_tile_data_area as u8)
            << 4 + (self.window_enable as u8)
            << 5 + (self.window_tile_map_area as u8)
            << 6 + (self.lcd_ppu_enable as u8)
            << 7
    }

    pub fn write(&mut self, value: u8) {
        let old_4 = self.bg_window_tile_data_area;

        self.bg_window_enable = (value >> 0) & 1 == 1;
        self.obj_enable = (value >> 1) & 1 == 1;
        self.obj_size = (value >> 2) & 1 == 1;
        self.bg_tile_map_area = (value >> 3) & 1 == 1;
        self.bg_window_tile_data_area = (value >> 4) & 1 == 1;
        self.window_enable = (value >> 5) & 1 == 1;
        self.window_tile_map_area = (value >> 6) & 1 == 1;
        self.lcd_ppu_enable = (value >> 7) & 1 == 1;

        if self.bg_window_tile_data_area != old_4 {
            println!("New: {}", self.bg_window_tile_data_area);
        }
    }
}

#[derive(Debug)]
pub struct PixelData {
    color: u8,
    palette: u8,
    background_priority: bool,
}

#[derive(Debug)]
enum PPUState {
    OAMSearch,
    PixelTransfer,
    VBlank,
    HBlank,
}

#[derive(Debug, Clone)]
pub struct OamData {
    data: Vec<u8>,
}

impl OamData {
    pub fn new(data: &[u8]) -> OamData {
        OamData {
            data: data.to_vec(),
        }
    }

    fn y_pos(&self) -> u8 {
        self.data[0]
    }

    fn x_pos(&self) -> u8 {
        self.data[1]
    }

    fn tile_index(&self) -> u8 {
        self.data[2]
    }

    fn palette_number(&self) -> u8 {
        self.data[3] >> 4 & 1
    }

    /// true iff horizontally mirrored
    fn x_flip(&self) -> bool {
        self.data[3] >> 5 & 1 == 1
    }

    /// true iff vertically mirrored
    fn y_flip(&self) -> bool {
        self.data[3] >> 6 & 1 == 1
    }

    /// false=No, true=BG and Window colors 1-3 over the OBJ
    fn bg_window_over_obj(&self) -> bool {
        self.data[3] >> 7 & 1 == 1
    }
}

pub trait Ppu<'a>: Addressable + Steppable {}

/// The PPU is responsible for the emulated gameboy's graphics.
pub struct CanvasPpu<'a> {
    tile_map: Texture<'a>,
    oam_tile_map: Texture<'a>,

    /// Tile data takes up addresses 0x8000-0x97ff.
    tile_data: Vec<u8>,

    /// Cache of decoded tile data -- the gameboy can store 384 different tiles
    tile_cache: Vec<Tile>,
    /// Addresses 0x9800-0x9bff are a 32x32 map of background tiles.
    /// Each byte contains the number of a tile to be displayed.
    background_map: Vec<u8>,

    /// A table containing data for 40 sprites
    sprite_tiles_table: Vec<u8>,

    /// LY: LCD Y coordinate (read only)
    ly: u8,
    /// Current x position in scanline
    lx: u32,
    lcdc: LCDC,

    background_queue: VecDeque<PixelData>,
    sprite_queue: VecDeque<PixelData>,

    state: PPUState,
    dots: u32,
}

/// Decoded tile data which is stored as a vec of 64 integers from 0 to 3
#[derive(Debug, Clone)]
pub struct Tile(Vec<u8>);
impl Tile {
    fn new() -> Tile {
        Tile(vec![0; 64])
    }

    fn as_rgba(&self) -> Vec<u8> {
        let mut color_data = vec![0; 64 * 4];
        for (i, pixel) in self.0.iter().enumerate() {
            let rgba = match pixel {
                0 => [255, 255, 255, 255],
                1 => [255, 200, 200, 200],
                2 => [255, 100, 100, 100],
                3 => [255, 0, 0, 0],
                _ => panic!()
            };
            color_data[i*4..(i+1)*4].copy_from_slice(&rgba);
        }
        color_data
    }

    fn as_oam_rgba(&self) -> Vec<u8> {
        let mut color_data = vec![0; 64 * 4];
        for (i, pixel) in self.0.iter().enumerate() {
            let rgba = match pixel {
                0 => [0, 0, 0, 0],
                1 => [255, 200, 200, 200],
                2 => [255, 100, 100, 100],
                3 => [255, 0, 0, 0],
                _ => panic!()
            };
            color_data[i*4..(i+1)*4].copy_from_slice(&rgba);
        }
        color_data
    }
}

impl<'a> CanvasPpu<'a> {
    pub fn new(creator: &'a TextureCreator<WindowContext>) -> Self {
        let tile_map = creator.create_texture_target(PixelFormatEnum::RGBA8888, 128, 192).unwrap();
        let oam_tile_map = creator.create_texture_target(PixelFormatEnum::RGBA8888, 128, 192).unwrap();

        let ppu = CanvasPpu {
            tile_map,
            oam_tile_map,

            tile_data: vec![0; 0x1800],
            // The gameboy has room for 384 tiles in addresses 0x8000 to 0x97ff
            tile_cache: vec![Tile::new(); 384],
            background_map: vec![0; 32 * 32],
            sprite_tiles_table: vec![0; 160],
            ly: 0,
            lx: 0,
            lcdc: LCDC::new(),
            background_queue: VecDeque::new(),
            sprite_queue: VecDeque::new(),
            state: PPUState::OAMSearch,
            dots: 0,
        };
        ppu
    }

    /// Update the cached forwards and backwards tile data associated with this memory address.
    /// Called after a write to tile data to keep caches valid.
    fn update_tile_cache(&mut self, address: Address) {
        // Translate the address into a relative address from 0x8000
        let address = address - 0x8000;

        // Tile data starts at 0x8000 and each tile occupies 16 bytes
        let tile_index: usize = address / 16;
        // Which row of the tile this address corresponds to, keeping in mind that each row is 2
        // bytes.
        let row_index: usize = (address % 16) / 2;

        let tile = &mut self.tile_cache[tile_index];

        let row_to_update = &mut tile.0[(row_index * 8)..(row_index * 8 + 8)];

        // Update row.
        // If the address is even, then it is the first byte for the row, otherwise it is the
        // second byte
        let byte_1;
        let byte_2;
        if address % 2 == 0 {
            byte_1 = self.tile_data[address];
            byte_2 = self.tile_data[address + 1];
        } else {
            byte_1 = self.tile_data[address - 1];
            byte_2 = self.tile_data[address];
        }

        for i in 0..8 {
            let bit_1 = (byte_1 >> i) & 1;
            let bit_2 = (byte_2 >> i) & 1;
            let color_id = (bit_2 << 1) | bit_1;
            row_to_update[7 - i] = color_id;
        }

        let x = (tile_index % 16) * 8;
        let y = tile_index / 16 * 8;
        self.tile_map.update(Some(Rect::new(x as i32, y as i32, 8, 8)), &tile.as_rgba(), 8*4).unwrap();
        self.oam_tile_map.update(Some(Rect::new(x as i32, y as i32, 8, 8)), &tile.as_oam_rgba(), 8*4).unwrap();
    }

    /// Uses the tile addressing method to adjust the provided index so it can be used with the tile cache.
    pub fn adjust_tile_index(&self, tile_index: usize, method: TileDataAddressingMethod) -> usize {
        match method {
            TileDataAddressingMethod::Method8000 => tile_index,
            TileDataAddressingMethod::Method8800 => {
                if tile_index <= 127 {
                    tile_index + 256
                } else {
                    tile_index
                }
            }
        }
    }

    pub fn set_tile(
        &mut self,
        texture_canvas: &mut sdl2::render::Canvas<Window>,
        row: usize,
        col: usize,
        tile_index: usize,
        method: TileDataAddressingMethod,
    ) -> Result<()> {
        let adjusted_index = self.adjust_tile_index(tile_index, method);
        let source_rect = Rect::new((adjusted_index as i32 % 16) * 8, adjusted_index as i32 / 16 * 8, 8, 8);
        let dest_rect = Rect::new(col as i32 * 8, row as i32 * 8, 8, 8);

        texture_canvas
            .copy(&self.tile_map, Some(source_rect), Some(dest_rect)).map_err(|e| Error::new(&e.to_string()))
    }

    /// x is tile's horizontal position, y is tile's vertical position.
    /// Keep in mind that the values in OAM are x + 8 and y + 16.
    /// If bottom_half is true, this method treats the provided object as the top half of a 16 row sprite to
    /// act on data corresponding to the bottom half.
    pub fn set_sprite(
        &mut self,
        texture_canvas: &mut sdl2::render::Canvas<Window>,
        oam_data: &OamData,
        tile_index_offset: i8,
        y_offset: i32,
    ) -> Result<()> {
        let x: i32 = i32::from(oam_data.x_pos()) - 8;
        let y: i32 = i32::from(oam_data.y_pos()) - 16 + y_offset;
        let tile_index = (oam_data.tile_index() as i16 + tile_index_offset as i16) as u8;

        let source_rect = Rect::new((tile_index as i32 % 16) * 8, tile_index as i32 / 16 * 8, 8, 8);
        let dest_rect = Rect::new(x, y, 8, 8);

        texture_canvas
            .copy_ex(
                &self.oam_tile_map, 
                Some(source_rect), 
                Some(dest_rect), 
                0., 
                None, 
                oam_data.x_flip(), 
                oam_data.y_flip(),
            )
            .map_err(|e| Error::new(&e.to_string()))
    }

    fn _read(&mut self, address: Address) -> Result<u8> {
        let value = match address {
            0x8000..=0x97ff => self.tile_data[address - 0x8000],
            0x9800..=0x9bff => self.background_map[address - 0x9800],
            0xfe00..=0xfe9f => self.sprite_tiles_table[address - 0xfe00],
            0xff40 => self.lcdc.read(),
            0xff44 => self.ly,
            _ => return Err(Error::new("Invalid address")),
        };

        Ok(value)
    }

    fn _write(&mut self, address: Address, data: u8) -> Result<()> {
        match address {
            0x8000..=0x97ff => {
                trace!("write to tile data: {:#x} into {:#x}", data, address);
                self.tile_data[address - 0x8000] = data;
                self.update_tile_cache(address);
            }
            0x9800..=0x9bff => {
                self.background_map[address - 0x9800] = data;
            }
            0xfe00..=0xfe9f => {
                self.sprite_tiles_table[address - 0xfe00] = data;
            }
            0xff40 => self.lcdc.write(data),
            _ => return Err(Error::new("Invalid address")),
        }

        Ok(())
    }

    pub fn render_tile_map<T: RenderTarget>(&mut self,
                           texture_canvas: &mut sdl2::render::Canvas<T>) -> Result<()> {
        texture_canvas
            .copy(&self.tile_map, None, Some(Rect::new(0, 0, 16 * 8, 24 * 8)))
            .map_err(|e| Error::new(&e.to_string()))
    }

    pub fn render_background_map(
        &mut self,
        texture_canvas: &mut sdl2::render::Canvas<Window>,
    ) -> Result<()> {
        let method = if self.lcdc.bg_window_tile_data_area {
            TileDataAddressingMethod::Method8000
        } else {
            TileDataAddressingMethod::Method8800
        };
        //println!("Method: {:?}", &method);

        // Render background map
        for row in 0..32 {
            for col in 0..32 {
                let tile_number = self.background_map[col + row * 32];
                self.set_tile(texture_canvas, row, col, tile_number.into(), method)?;
            }
        }

        Ok(())
    }

    pub fn render_sprites(
        &mut self,
        texture_canvas: &mut sdl2::render::Canvas<Window>,
    ) -> Result<()> {

        for i in 0..40 {
            let oam_data = OamData::new(&self.sprite_tiles_table[i * 4..i * 4 + 4]);

            if !self.lcdc.obj_size {
                // 8x8
                self.set_sprite(texture_canvas, &oam_data, 0, 0)?;
            } else {
                // 8x16
                if !oam_data.y_flip() {
                    self.set_sprite(texture_canvas, &oam_data, 0, 0)?;
                    self.set_sprite(texture_canvas, &oam_data, 1, 8)?;
                } else {
                    self.set_sprite(texture_canvas, &oam_data, 1, 0)?;
                    self.set_sprite(texture_canvas, &oam_data, 0, 8)?;
                }
            }
        }

        Ok(())
    }
}

impl<'a> Steppable for CanvasPpu<'a> {
    fn step(&mut self, state: &GameBoyState) -> Result<ElapsedTime> {
        self.dots += 1;

        match self.state {
            PPUState::OAMSearch => {
                if self.dots == 80 {
                    self.state = PPUState::PixelTransfer;
                }
            }
            PPUState::PixelTransfer => {
                // TODO: Fetch pixel data into our pixel FIFO.
                // TODO: Put a pixel (if any) from the FIFO on screen.

                // For now, just use the current xy coordinates as an index into the background map
                // to get a pixel
                if self.lx == 0 {
                    //self.render_background_map()?;
                }

                self.lx += 1;
                if self.lx == 160 {
                    self.lx = 0;
                    self.state = PPUState::HBlank;
                }
            }
            PPUState::HBlank => {
                if self.dots == 456 {
                    self.dots = 0;
                    self.ly += 1;
                    if self.ly == 144 {
                        self.state = PPUState::VBlank;
                        state.memory_bus.borrow_mut().interrupt(Interrupt::VBlank)?;
                        //println!("Start VBLANK");
                    } else {
                        self.state = PPUState::OAMSearch;
                    }
                }
            }
            PPUState::VBlank => {
                if self.dots == 456 {
                    self.dots = 0;
                    self.ly += 1;
                    if self.ly == 153 {
                        self.ly = 0;
                        //println!("End VBLANK");
                        self.state = PPUState::OAMSearch;
                    }
                }
            }
        }

        Ok(1)
    }
}

impl<'a> Addressable for CanvasPpu<'a> {
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

impl<'a> Ppu<'a> for CanvasPpu<'a> {}
