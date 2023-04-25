use crate::component::Address;

use super::{
    base_ppu::{PpuState, Tile},
    lcd::LcdControl,
    palette::SpriteTileColor,
    OamData, TileColor, TileDataAddressingMethod,
};

pub struct Renderer {
    /// Cache of decoded tile data -- the gameboy can store 384 different tiles
    tile_cache: Vec<Tile>,

    pub screen_pixels: Vec<TileColor>,

    current_scanline_objects: Vec<OamData>,

    lcdc_cache: [LcdControl; 160],
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            // The gameboy has room for 384 tiles in addresses 0x8000 to 0x97ff
            tile_cache: vec![Tile::new(); 384],
            screen_pixels: vec![TileColor::White; 160 * 144],
            current_scanline_objects: vec![],
            lcdc_cache: [LcdControl::new(); 160],
        }
    }

    pub fn cache_lcdc(&mut self, lcdc: LcdControl, x: usize) {
        self.lcdc_cache[x] = lcdc;
    }

    pub fn render_scanline(&mut self, state: &PpuState, y: u8) {
        for x in 0..160 {
            self.place_pixel(state, x, y)
        }
    }

    fn get_bg_or_window_pixel(&self, state: &PpuState, x: u8, y: u8) -> TileColor {
        let lcdc: LcdControl = self.lcdc_cache[usize::from(x)];
        if self.window_contains(state, x, y) && lcdc.window_enable {
            let win_x = x + 7 - state.wx;
            let win_y = state.lcd.window_line_counter; //y - self.state.wy;

            self.get_win_pixel(state, win_x, win_y)
        } else {
            let bg_x = state.scx.wrapping_add(x);
            let bg_y = state.scy.wrapping_add(y);

            self.get_bg_pixel(state, bg_x, bg_y)
        }
    }

    fn place_pixel(&mut self, state: &PpuState, x: u8, y: u8) {
        let lcdc: LcdControl = self.lcdc_cache[usize::from(x)];

        // Check sprite pixel first
        if lcdc.obj_enable {
            if let (SpriteTileColor::TileColor(tile_color), Some(oam_data)) =
                self.get_obj_pixel(state, x, y)
            {
                // We are working with a on transparent sprite pixel

                // Check if the bg/window pixel should be rendered over the OBJ
                let bg_window_pixel = self.get_bg_or_window_pixel(state, x, y);
                let pixel: TileColor;
                if oam_data.bg_window_over_obj() && bg_window_pixel != TileColor::from_u8(0) {
                    pixel = bg_window_pixel;
                } else {
                    pixel = tile_color;
                }

                self.screen_pixels[160 * y as usize + x as usize] = pixel;
                return;
            }
        }

        if !lcdc.bg_window_enable {
            self.screen_pixels[160 * y as usize + x as usize] = TileColor::White;
            return;
        }

        let bg_window_pixel = self.get_bg_or_window_pixel(state, x, y);
        self.screen_pixels[160 * y as usize + x as usize] = bg_window_pixel;
    }

    /// Update the cached forwards and backwards tile data associated with this memory address.
    /// Called after a write to tile data to keep caches valid.
    pub fn update_tile_cache(&mut self, tile_data: &[u8], address: Address) {
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

    pub fn update_scanline_cache(&mut self, ppu_state: &PpuState, y_coord: u8) {
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

    pub fn get_bg_pixel(&self, ppu_state: &PpuState, bg_x: u8, bg_y: u8) -> TileColor {
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

    pub fn get_win_pixel(&self, ppu_state: &PpuState, win_x: u8, win_y: u8) -> TileColor {
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

    pub fn window_contains(&self, ppu_state: &PpuState, x: u8, y: u8) -> bool {
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

    pub fn get_obj_pixel(
        &self,
        ppu_state: &PpuState,
        x: u8,
        y: u8,
    ) -> (SpriteTileColor, Option<OamData>) {
        let lcdc = self.lcdc_cache[usize::from(x)];
        for object in self.current_scanline_objects.iter() {
            let x_pos = i16::from(object.x_pos()) - 8;
            // skip over objects that don't contain this x value
            if !(x_pos <= x.into() && i16::from(x) < x_pos + 8) {
                continue;
            }

            let y_pos = i16::from(object.y_pos()) - 16;

            let tile_index = if !lcdc.obj_size {
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
