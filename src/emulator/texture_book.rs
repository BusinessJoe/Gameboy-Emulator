use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{BlendMode, Canvas, Texture, TextureCreator};
use sdl2::video::Window;

/// Stores all the textures displayed in the gui in one place.
/// Requires the "unsafe_textures" feature of sdl2 because lifetimes were too confusing.
pub struct TextureBook {
    pub texture_creator: TextureCreator<sdl2::video::WindowContext>,
    pub background_map: Texture,
    pub lcd_display: Texture,
    pub sprite_map: Texture,
}

impl TextureBook {
    pub fn new(canvas: &Canvas<Window>) -> Result<TextureBook, String> {
        let creator = canvas.texture_creator();
        let background_map = creator
            .create_texture_target(PixelFormatEnum::RGBA8888, 8 * 32, 8 * 32)
            .map_err(|e| e.to_string())?;
        let mut sprite_map = creator
            .create_texture_target(PixelFormatEnum::RGBA8888, 160, 144)
            .map_err(|e| e.to_string())?;
        let lcd_display = creator
            .create_texture_target(PixelFormatEnum::RGBA8888, 160, 144)
            .map_err(|e| e.to_string())?;
        sprite_map.set_blend_mode(BlendMode::Blend);

        Ok(TextureBook {
            texture_creator: creator,
            background_map,
            lcd_display,
            sprite_map,
        })
    }
}
