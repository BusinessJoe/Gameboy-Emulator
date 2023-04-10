use crate::{
    component::{Address, Addressable, ElapsedTime, Steppable},
    gameboy::GameBoyState,
};
use crate::{Error, Result};

use super::palette::PaletteRegister;
use super::{
    lcd::{self, UpdatePixel},
    palette::{SpriteTileColor, TileColor},
    OamData, TileDataAddressingMethod,
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
pub struct Tile(Vec<u8>);
impl Tile {
    pub fn new() -> Tile {
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
                _ => panic!(),
            };
            color_data[i * 4..(i + 1) * 4].copy_from_slice(&rgba);
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
                _ => panic!(),
            };
            color_data[i * 4..(i + 1) * 4].copy_from_slice(&rgba);
        }
        color_data
    }

    pub fn get_pixel(&self, x: u8, y: u8) -> u8 {
        self.0[(x + 8 * y) as usize]
    }
}

struct Renderer {
    /// Cache of decoded tile data -- the gameboy can store 384 different tiles
    tile_cache: Vec<Tile>,

    screen_pixels: Vec<TileColor>,

    current_scanline_objects: Vec<OamData>,
}

impl Renderer {
    /// Update the cached forwards and backwards tile data associated with this memory address.
    /// Called after a write to tile data to keep caches valid.
    fn update_tile_cache(&mut self, tile_data: &[u8], address: Address) {
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
            byte_1 = tile_data[address];
            byte_2 = tile_data[address + 1];
        } else {
            byte_1 = tile_data[address - 1];
            byte_2 = tile_data[address];
        }

        for i in 0..8 {
            let bit_1 = (byte_1 >> i) & 1;
            let bit_2 = (byte_2 >> i) & 1;
            let color_id = (bit_2 << 1) | bit_1;
            row_to_update[7 - i] = color_id;
        }
    }

    fn update_scanline_cache(&mut self, ppu_state: &PpuState, y_coord: u8) {
        self.current_scanline_objects = self.get_scanline_objects(ppu_state, y_coord);
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

    fn get_bg_pixel(&self, ppu_state: &PpuState, bg_x: u8, bg_y: u8) -> TileColor {
        let tile_x = bg_x / 8;
        let tile_y = bg_y / 8;
        let tile_sub_x = bg_x % 8;
        let tile_sub_y = bg_y % 8;

        let tile_map_area = if !ppu_state.lcd.lcd_control.bg_tile_map_area {
            &ppu_state.background_map
        } else {
            &ppu_state.window_map
        };
        let mut tile_index = tile_map_area[tile_x as usize + 32 * tile_y as usize] as usize;
        let method = if ppu_state.lcd.lcd_control.bg_window_tile_data_area {
            TileDataAddressingMethod::Method8000
        } else {
            TileDataAddressingMethod::Method8800
        };
        tile_index = self.adjust_tile_index(tile_index, method);

        let tile = &self.tile_cache[tile_index];
        let index = tile.get_pixel(tile_sub_x, tile_sub_y);
        ppu_state.background_palette.map_index(index)
    }

    fn get_win_pixel(&self, ppu_state: &PpuState, win_x: u8, win_y: u8) -> TileColor {
        let tile_x = win_x / 8;
        let tile_y = win_y / 8;
        let tile_sub_x = win_x % 8;
        let tile_sub_y = win_y % 8;

        let tile_map_area = if !ppu_state.lcd.lcd_control.window_tile_map_area {
            &ppu_state.background_map
        } else {
            &ppu_state.window_map
        };
        let mut tile_index = tile_map_area[tile_x as usize + 32 * tile_y as usize] as usize;
        let method = if ppu_state.lcd.lcd_control.bg_window_tile_data_area {
            TileDataAddressingMethod::Method8000
        } else {
            TileDataAddressingMethod::Method8800
        };
        tile_index = self.adjust_tile_index(tile_index, method);

        let tile = &self.tile_cache[tile_index];
        let index = tile.get_pixel(tile_sub_x, tile_sub_y);
        ppu_state.background_palette.map_index(index)
    }

    fn window_contains(&self, ppu_state: &PpuState, x: u8, y: u8) -> bool {
        let contains_x = x + 7 >= ppu_state.wx;
        let contains_y = y >= ppu_state.wy;
        contains_x && contains_y
    }

    /// returns up to 10 objects
    /// each object is represented by 4 bytes
    fn get_scanline_objects(&self, ppu_state: &PpuState, y: u8) -> Vec<OamData> {
        let mut objects = Vec::new();

        for object in ppu_state.sprite_tiles_table.chunks_exact(4) {
            let y_pos = i16::from(object[0]) - 16;

            let y_upper = y_pos
                + if ppu_state.lcd.lcd_control.obj_size {
                    16
                } else {
                    8
                };

            if y_pos <= y.into() && i16::from(y) < y_upper {
                // since object guaranteed to be 4 bytes by chunks_exact(), object is always valid oam data.
                objects.push(OamData::new(object));
            }

            if objects.len() == 10 {
                break;
            }
        }

        // prioritize smaller x-coordinates
        objects.sort_by_key(|oam| oam.x_pos());

        objects
    }

    fn get_obj_pixel(
        &self,
        ppu_state: &PpuState,
        x: u8,
        y: u8,
    ) -> (SpriteTileColor, Option<OamData>) {
        for object in self.current_scanline_objects.iter() {
            let x_pos = i16::from(object.x_pos()) - 8;
            // skip over objects that don't contain this x value
            if !(x_pos <= x.into() && i16::from(x) < x_pos + 8) {
                continue;
            }

            let y_pos = i16::from(object.y_pos()) - 16;

            let tile_index = if !ppu_state.lcd.lcd_control.obj_size {
                // 8x8
                object.tile_index()
            } else {
                // 8x16
                let (top_idx, bot_idx) = object.tile_index_16();
                if (i16::from(y) - y_pos < 8) ^ object.y_flip() {
                    top_idx
                } else {
                    bot_idx
                }
            };
            let tile = &self.tile_cache[tile_index as usize];

            // calculate x-index into tile, accounting for x flip
            let mut tile_sub_x = i16::from(x) - x_pos;
            if object.x_flip() {
                tile_sub_x = 7 - tile_sub_x;
            }

            // calculate y-index into tile, accounting for tall tiles and y flip
            let mut tile_sub_y: i16 = if i16::from(y) - y_pos < 8 {
                i16::from(y) - y_pos
            } else {
                i16::from(y) - y_pos - 8
            };
            if object.y_flip() {
                tile_sub_y = 7 - tile_sub_y;
            }

            let index = tile.get_pixel(
                tile_sub_x.try_into().unwrap(),
                tile_sub_y.try_into().unwrap(),
            );
            // ignore transparent pixels
            if index != 0 {
                let palette = match object.palette_number() {
                    0 => &ppu_state.object_palette_0,
                    1 => &ppu_state.object_palette_1,
                    _ => panic!(),
                };
                return (palette.map_sprite_index(index), Some(object.clone()));
            }
        }

        (SpriteTileColor::Transparent, None)
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
            renderer: Renderer {
                // The gameboy has room for 384 tiles in addresses 0x8000 to 0x97ff
                tile_cache: vec![Tile::new(); 384],
                screen_pixels: vec![TileColor::White; 160 * 144],
                current_scanline_objects: vec![],
            },
        }
    }

    pub fn get_screen(&self) -> &[TileColor] {
        &self.renderer.screen_pixels
    }

    pub fn get_frame_count(&self) -> u128 {
        self.state.lcd.frame_count
    }

    fn get_bg_or_window_pixel(&self, x: u8, y: u8) -> TileColor {
        if self.renderer.window_contains(&self.state, x, y)
            && self.state.lcd.lcd_control.window_enable
        {
            let win_x = x + 7 - self.state.wx;
            let win_y = self.state.lcd.window_line_counter; //y - self.state.wy;

            self.renderer.get_win_pixel(&self.state, win_x, win_y)
        } else {
            let bg_x = self.state.scx.wrapping_add(x);
            let bg_y = self.state.scy.wrapping_add(y);

            self.renderer.get_bg_pixel(&self.state, bg_x, bg_y)
        }
    }

    fn place_pixel(&mut self, x: u8, y: u8) -> Result<()> {
        if x == 0 {
            self.renderer.update_scanline_cache(&self.state, y);
        }

        // Check sprite pixel first
        if self.state.lcd.lcd_control.obj_enable {
            if let (SpriteTileColor::TileColor(tile_color), Some(oam_data)) =
                self.renderer.get_obj_pixel(&self.state, x, y)
            {
                // We are working with a on transparent sprite pixel

                // Check if the bg/window pixel should be rendered over the OBJ
                let bg_window_pixel = self.get_bg_or_window_pixel(x, y);
                let pixel: TileColor;
                if oam_data.bg_window_over_obj() && bg_window_pixel != TileColor::from_u8(0) {
                    pixel = bg_window_pixel;
                } else {
                    pixel = tile_color;
                }

                self.renderer.screen_pixels[160 * y as usize + x as usize] = pixel;
                return Ok(());
            }
        }

        if !self.state.lcd.lcd_control.bg_window_enable {
            self.renderer.screen_pixels[160 * y as usize + x as usize] = TileColor::White;
            return Ok(());
        }

        let bg_window_pixel = self.get_bg_or_window_pixel(x, y);
        self.renderer.screen_pixels[160 * y as usize + x as usize] = bg_window_pixel;
        Ok(())
    }
}

impl Steppable for BasePpu {
    fn step(&mut self, state: &GameBoyState) -> Result<ElapsedTime> {
        let mut memory_bus = state.memory_bus.borrow_mut();
        if let Some(UpdatePixel { x, y }) =
            self.state
                .lcd
                .step(&mut memory_bus, self.state.wx, self.state.wy)?
        {
            self.place_pixel(x, y)?;
        }
        Ok(1)
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
            match address {
                0x8000..=0x97ff => {
                    self.renderer
                        .update_tile_cache(&self.state.tile_data, address);
                }
                _ => {}
            }
        }

        Ok(())
    }
}
