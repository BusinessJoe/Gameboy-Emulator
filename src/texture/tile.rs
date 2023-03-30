use crate::{Error, Result};
use sdl2::{
    pixels::PixelFormatEnum,
    rect::Rect,
    render::{Texture, TextureCreator, WindowCanvas},
    video::WindowContext,
};

pub struct TileTexture {
    texture: Texture,
    width: u32,  // width of texture in 8x8 tiles
    height: u32, // height of texture in 8x8 tiles
}

impl TileTexture {
    pub fn new(
        texture_creator: &TextureCreator<WindowContext>,
        width: u32,
        height: u32,
    ) -> Result<TileTexture> {
        let texture = texture_creator
            .create_texture_target(PixelFormatEnum::RGBA8888, width * 8, height * 8)
            .map_err(|e| Error::from_message(e.to_string()))?;

        Ok(Self {
            texture,
            width,
            height,
        })
    }

    pub fn get_texture(&self) -> &Texture {
        &self.texture
    }

    pub fn get_texture_mut(&mut self) -> &mut Texture {
        &mut self.texture
    }

    pub fn copy_to(&self, canvas: &mut WindowCanvas, x: i32, y: i32) -> Result<()> {
        canvas.copy(
            self.get_texture(),
            None,
            Some(Rect::new(x * 8, y * 8, self.width * 8, self.height * 8)),
        )?;

        Ok(())
    }
}
