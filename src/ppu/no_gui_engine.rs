use crate::error::Result;
use crate::component::Address;

use super::base_ppu::GraphicsEngine;

pub struct NoGuiEngine {}

impl GraphicsEngine for NoGuiEngine {
    fn after_write(&mut self, _ppu_state: &super::base_ppu::PpuState, _address: Address) {
        // do nothing
    }

    fn render(
        &mut self,
        _ppu_state: &super::base_ppu::PpuState,
        _canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
        _texture_book: &mut crate::texture::TextureBook,
    ) -> Result<()> {
        // do nothing
        Ok(())
    }
}
