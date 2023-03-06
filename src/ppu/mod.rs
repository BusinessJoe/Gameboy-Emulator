/*!
 * This PPU serves as an implementation for all the gameboy's graphics. It maintains an internal
 * representation of the screen.
 */

mod canvas_ppu;
mod lcd;
mod no_gui_ppu;

pub use canvas_ppu::CanvasPpu;
pub use no_gui_ppu::NoGuiPpu;

use crate::component::{Addressable, Steppable};

#[derive(Debug, Clone, Copy)]
pub enum TileDataAddressingMethod {
    Method8000,
    Method8800,
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

pub trait Ppu: Addressable + Steppable {}
