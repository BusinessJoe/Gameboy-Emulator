use crate::component::Address;
use crate::error::Result;

use crate::ppu::base_ppu::{GraphicsEngine, PpuState};

pub struct NoGuiEngine {}

impl GraphicsEngine for NoGuiEngine {
    fn after_write(&mut self, _ppu_state: &PpuState, _address: Address) {
        // do nothing
    }

    fn render(&mut self, _ppu_state: &PpuState) -> Result<()> {
        // do nothing
        Ok(())
    }

    fn place_pixel(
        &mut self,
        _ppu_state: &PpuState,
        _x: u8,
        _y: u8,
    ) -> Result<()> {
        // do nothing
        Ok(())
    }
}
