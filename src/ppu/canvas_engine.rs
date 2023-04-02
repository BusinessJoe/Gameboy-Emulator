use crate::component::Address;
use crate::error::{Error, Result};
use crate::ppu::{OamData, TileDataAddressingMethod, self};
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::{RenderTarget, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};

use super::base_ppu::{GraphicsEngine, PpuState};
use super::palette::TileColor;

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

pub struct CanvasEngine {
    tile_map: Texture,
    oam_tile_map: Texture,

    /// Cache of decoded tile data -- the gameboy can store 384 different tiles
    tile_cache: Vec<Tile>,

    screen_pixels: Vec<u8>,
}

impl CanvasEngine {
    pub fn new(creator: &TextureCreator<WindowContext>) -> Result<Self> {
        let tile_map = creator
            .create_texture_target(PixelFormatEnum::RGBA8888, 128, 192)
            .map_err(|e| Error::from_message(e.to_string()))?;
        let oam_tile_map = creator
            .create_texture_target(PixelFormatEnum::RGBA8888, 128, 192)
            .map_err(|e| Error::from_message(e.to_string()))?;

        Ok(Self {
            tile_map,
            oam_tile_map,
            // The gameboy has room for 384 tiles in addresses 0x8000 to 0x97ff
            tile_cache: vec![Tile::new(); 384],

            screen_pixels: vec![0; 160 * 144],
        })
    }

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

        let x = (tile_index % 16) * 8;
        let y = tile_index / 16 * 8;
        self.tile_map
            .update(
                Some(Rect::new(x as i32, y as i32, 8, 8)),
                &tile.as_rgba(),
                8 * 4,
            )
            .unwrap();
        self.oam_tile_map
            .update(
                Some(Rect::new(x as i32, y as i32, 8, 8)),
                &tile.as_oam_rgba(),
                8 * 4,
            )
            .unwrap();
    }

    fn update_scanline_cache(&mut self, ppu_state: &PpuState, y_coord: u8) {}

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
        let source_rect = Rect::new(
            (adjusted_index as i32 % 16) * 8,
            adjusted_index as i32 / 16 * 8,
            8,
            8,
        );
        let dest_rect = Rect::new(col as i32 * 8, row as i32 * 8, 8, 8);

        texture_canvas
            .copy(&self.tile_map, Some(source_rect), Some(dest_rect))
            .map_err(|e| Error::from_message(e))
    }

    /// Copies an 8x16 sprite described by oam data from the oam tile map onto the canvas.
    pub fn set_sprite(
        &mut self,
        texture_canvas: &mut sdl2::render::Canvas<Window>,
        oam_data: &OamData,
    ) -> Result<()> {
        // Position values in OAM data are x + 8 and y + 16. Account for that here.
        let x: i32 = i32::from(oam_data.x_pos()) - 8;
        let y: i32 = i32::from(oam_data.y_pos()) - 16;
        let tile_index = oam_data.tile_index();

        let source_rect = Rect::new(
            (tile_index as i32 % 16) * 8,
            tile_index as i32 / 16 * 8,
            8,
            8,
        );
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
            .map_err(|e| Error::from_message(e))
    }

    /// Copies an 8x16 sprite described by oam data from the oam tile map onto the canvas.
    pub fn set_sprite_16(
        &mut self,
        texture_canvas: &mut sdl2::render::Canvas<Window>,
        oam_data: &OamData,
    ) -> Result<()> {
        let (top_idx, bottom_idx) = if !oam_data.y_flip() {
            oam_data.tile_index_16()
        } else {
            let (top, bottom) = oam_data.tile_index_16();
            (bottom, top)
        };

        // top half
        let source_rect = Rect::new((top_idx as i32 % 16) * 8, top_idx as i32 / 16 * 8, 8, 8);
        // Position values in OAM data are x + 8 and y + 16. Account for that here.
        let dest_rect = Rect::new(
            i32::from(oam_data.x_pos()) - 8,
            i32::from(oam_data.y_pos()) - 16,
            8,
            8,
        );

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
            .map_err(|e| Error::from_message(e))?;

        // bottom half
        let source_rect = Rect::new(
            (bottom_idx as i32 % 16) * 8,
            bottom_idx as i32 / 16 * 8,
            8,
            8,
        );
        // Position values in OAM data are x + 8 and y + 16. Account for that here.
        let dest_rect = Rect::new(
            i32::from(oam_data.x_pos()) - 8,
            i32::from(oam_data.y_pos()) - 16 + 8, // bottom half is 8 pixels (1 tile) lower
            8,
            8,
        );

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
            .map_err(|e| Error::from_message(e))?;

        Ok(())
    }

    pub fn render_tile_map<T: RenderTarget>(
        &mut self,
        texture_canvas: &mut sdl2::render::Canvas<T>,
        dst: Rect,
    ) -> Result<()> {
        texture_canvas
            .copy(&self.tile_map, None, Some(dst))
            .map_err(|e| Error::from_message(e))
    }

    pub fn render_main_screen(
        &mut self,
        ppu_state: &PpuState,
        texture_canvas: &mut sdl2::render::Canvas<Window>,
        background_map: &sdl2::render::Texture,
        window_map: &sdl2::render::Texture,
        sprite_map: &sdl2::render::Texture,
    ) -> Result<()> {
        //Render screen pixels (currently just the background layer) onto the canvas
        for x in 0u8..160 {
            for y in 0u8..144 {
                let color_index = self.screen_pixels[x as usize + 160 * y as usize];
                let color = match ppu_state.background_palette.map_index(color_index) {
                    TileColor::White => sdl2::pixels::Color::RGBA(255, 255, 255, 255),
                    TileColor::LightGrey => sdl2::pixels::Color::RGBA(200, 200, 200, 255),
                    TileColor::DarkGrey => sdl2::pixels::Color::RGBA(100, 100, 100, 255),
                    TileColor::Black => sdl2::pixels::Color::RGBA(0, 0, 0, 255),
                    TileColor::Debug => sdl2::pixels::Color::RGBA(255, 0, 0, 255),
                };
                texture_canvas.set_draw_color(color);
                texture_canvas.draw_point((x as i32, y as i32))?;
            }
        }

        // {
        //     if ppu_state.lcd.lcd_control.bg_window_enable
        //         && (0..=166).contains(&ppu_state.wx)
        //         && (0..=143).contains(&ppu_state.wy)
        //     {
        //         let dst = Rect::new(
        //             ppu_state.wx as i32 - 7,
        //             ppu_state.wy as i32,
        //             160 - (ppu_state.wx as u32 - 7),
        //             144 - ppu_state.wy as u32,
        //         );
        //         let src = Rect::new(0, 0, dst.width(), dst.height());
        //         texture_canvas.copy(window_map, Some(src), Some(dst))?;
        //     }
        // }

        texture_canvas.copy(sprite_map, None, None)?;

        Ok(())
    }

    pub fn render_background_map(
        &mut self,
        ppu_state: &PpuState,
        texture_canvas: &mut sdl2::render::Canvas<Window>,
    ) -> Result<()> {
        let method = if ppu_state.lcd.lcd_control.bg_window_tile_data_area {
            TileDataAddressingMethod::Method8000
        } else {
            TileDataAddressingMethod::Method8800
        };

        // Render background map
        for row in 0..32 {
            for col in 0..32 {
                let tile_number = ppu_state.background_map[col + row * 32];
                self.set_tile(texture_canvas, row, col, tile_number.into(), method)?;
            }
        }

        // Due to the viewport offset, the screen is split into four rectangles.
        texture_canvas.set_draw_color(sdl2::pixels::Color::RGBA(0, 0, 255, 255));
        let top_left = Rect::new(
            ppu_state.scx.into(),
            ppu_state.scy.into(),
            std::cmp::min(160, 256 - u32::from(ppu_state.scx)),
            std::cmp::min(144, 256 - u32::from(ppu_state.scy)),
        );
        texture_canvas.draw_rect(top_left)?;

        if top_left.width() < 160 {
            let top_right = Rect::new(
                0,
                ppu_state.scy.into(),
                160 - top_left.width(),
                top_left.height(),
            );
            texture_canvas.draw_rect(top_right)?;
        }

        if top_left.height() < 144 {
            let bottom_left = Rect::new(
                ppu_state.scx.into(),
                0,
                top_left.width(),
                144 - top_left.height(),
            );
            texture_canvas.draw_rect(bottom_left)?;
        }

        if top_left.width() < 160 && top_left.height() < 144 {
            let bottom_right = Rect::new(0, 0, 160 - top_left.width(), 144 - top_left.height());
            texture_canvas.draw_rect(bottom_right)?;
        }

        Ok(())
    }

    pub fn render_window_map(
        &mut self,
        ppu_state: &PpuState,
        texture_canvas: &mut sdl2::render::Canvas<Window>,
    ) -> Result<()> {
        let method = if ppu_state.lcd.lcd_control.bg_window_tile_data_area {
            TileDataAddressingMethod::Method8000
        } else {
            TileDataAddressingMethod::Method8800
        };

        // Render background map
        for row in 0..32 {
            for col in 0..32 {
                let tile_number = ppu_state.window_map[col + row * 32];
                self.set_tile(texture_canvas, row, col, tile_number.into(), method)?;
            }
        }

        Ok(())
    }

    pub fn render_sprites(
        &mut self,
        ppu_state: &PpuState,
        texture_canvas: &mut sdl2::render::Canvas<Window>,
    ) -> Result<()> {
        for i in 0..40 {
            let oam_data = OamData::new(&ppu_state.sprite_tiles_table[i * 4..i * 4 + 4]);

            if !ppu_state.lcd.lcd_control.obj_size {
                // 8x8
                self.set_sprite(texture_canvas, &oam_data)?;
            } else {
                self.set_sprite_16(texture_canvas, &oam_data)?;
            }
        }

        Ok(())
    }
}

impl GraphicsEngine for CanvasEngine {
    fn after_write(&mut self, ppu_addressables: &PpuState, address: Address) {
        match address {
            0x8000..=0x97ff => {
                self.update_tile_cache(&ppu_addressables.tile_data, address);
            }
            _ => {}
        }
    }

    fn render(
        &mut self,
        ppu_state: &PpuState,
        canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
        texture_book: &mut crate::texture::TextureBook,
    ) -> Result<()> {
        self.render_tile_map(canvas, Rect::new((20 + 1) * 8, 0, 16 * 8, 24 * 8))
            .expect("error rendering tile map");

        canvas
            .with_texture_canvas(
                &mut texture_book.background_map.get_texture_mut(),
                |mut texture_canvas| {
                    self.render_background_map(ppu_state, &mut texture_canvas)
                        .expect("error rendering background map");
                },
            )
            .map_err(|e| Error::from_message(e.to_string()))?;

        canvas
            .with_texture_canvas(
                &mut texture_book.window_map.get_texture_mut(),
                |mut texture_canvas| {
                    self.render_window_map(ppu_state, &mut texture_canvas)
                        .expect("error rendering window map");
                },
            )
            .map_err(|e| Error::from_message(e.to_string()))?;

        canvas
            .with_texture_canvas(&mut texture_book.sprite_map, |mut texture_canvas| {
                texture_canvas.set_draw_color(sdl2::pixels::Color::RGBA(0, 0, 0, 0));
                texture_canvas.clear();
                // Render sprites over background map for now
                self.render_sprites(ppu_state, &mut texture_canvas)
                    .expect("error rendering sprite");
            })
            .map_err(|e| Error::from_message(e.to_string()))?;

        canvas
            .with_texture_canvas(&mut texture_book.main_screen, |texture_canvas| {
                self.render_main_screen(
                    ppu_state,
                    texture_canvas,
                    &texture_book.background_map.get_texture(),
                    &texture_book.window_map.get_texture(),
                    &texture_book.sprite_map,
                )
                .expect("error rendering main screen");
            })
            .map_err(|e| Error::from_message(e.to_string()))?;

        texture_book
            .background_map
            .copy_to(canvas, 20 + 1 + 16 + 1, 0)?;
        texture_book
            .window_map
            .copy_to(canvas, 20 + 1 + 16 + 1 + 32 + 1, 0)?;

        canvas.copy(
            &texture_book.main_screen,
            None,
            Some(Rect::new(0, 0, 160, 144)),
        )?;

        Ok(())
    }

    fn place_pixel(&mut self, ppu_state: &PpuState, x: u8, y: u8) -> Result<()> {
        if x == 0 {
            self.update_scanline_cache(ppu_state, y);
        }

        if !ppu_state.lcd.lcd_control.bg_window_enable {
            self.screen_pixels[160 * y as usize + x as usize] = 4; // values outside 0..3 are white
            return Ok(());
        }

        if self.window_contains(ppu_state, x, y) {
            let win_x = (x - (ppu_state.wx - 7));
            let win_y = (y - ppu_state.wy);
            let pixel = self.get_win_pixel(ppu_state, win_x, win_y);
            self.screen_pixels[160 * y as usize + x as usize] = pixel;
        } else {
            let bg_x = (ppu_state.scx + x) % 255;
            let bg_y = (ppu_state.scy + y) % 255;

            let pixel = self.get_bg_pixel(bg_x, bg_y, ppu_state);
            self.screen_pixels[160 * y as usize + x as usize] = pixel;
        }
        Ok(())
    }

}

impl CanvasEngine {
    fn get_bg_pixel(&self, bg_x: u8, bg_y: u8, ppu_state: &PpuState) -> u8 {
        let tile_x = bg_x / 8;
        let tile_y = bg_y / 8;
        let tile_sub_x = bg_x % 8;
        let tile_sub_y = bg_y % 8;

        let mut tile_index =
            ppu_state.background_map[tile_x as usize + 32 * tile_y as usize] as usize;
        let method = if ppu_state.lcd.lcd_control.bg_window_tile_data_area {
            TileDataAddressingMethod::Method8000
        } else {
            TileDataAddressingMethod::Method8800
        };
        tile_index = self.adjust_tile_index(tile_index, method);

        let tile = &self.tile_cache[tile_index];
        tile.get_pixel(tile_sub_x, tile_sub_y)
    }
}
