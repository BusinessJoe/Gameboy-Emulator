use crate::error::{Error, Result};
use crate::texture::TileTexture;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{BlendMode, Canvas, Texture, TextureCreator};
use sdl2::video::Window;

/// Stores all the textures displayed in the gui in one place.
/// Requires the "unsafe_textures" feature of sdl2 because lifetimes were too confusing.
pub struct TextureBook {
    pub texture_creator: TextureCreator<sdl2::video::WindowContext>,
    pub background_map: TileTexture,
    pub window_map: TileTexture,
    pub sprite_map: Texture,
    pub main_screen: Texture,
}

impl TextureBook {
    pub fn new(canvas: &Canvas<Window>) -> Result<TextureBook> {
        let creator = canvas.texture_creator();
        let background_map = TileTexture::new(&creator, 32, 32)?;
        let window_map = TileTexture::new(&creator, 32, 32)?;
        let mut sprite_map = creator
            .create_texture_target(PixelFormatEnum::RGBA8888, 160, 144)
            .map_err(|e| Error::from_message(e.to_string()))?;
        sprite_map.set_blend_mode(BlendMode::Blend);
        let main_screen = creator
            .create_texture_target(PixelFormatEnum::RGBA8888, 160, 144)
            .map_err(|e| Error::from_message(e.to_string()))?;

        Ok(TextureBook {
            texture_creator: creator,
            background_map,
            window_map,
            sprite_map,
            main_screen,
        })
    }
}
