/*!
 * The PPU serves as an implementation for all the gameboy's graphics. It maintains an internal
 * representation of the screen.
 */

pub(crate) mod base_ppu;
mod lcd;
pub(crate) mod palette;

pub use base_ppu::BasePpu;
pub use palette::TileColor;

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

    pub fn y_pos(&self) -> u8 {
        self.data[0]
    }

    pub fn x_pos(&self) -> u8 {
        self.data[1]
    }

    pub fn tile_index(&self) -> u8 {
        self.data[2]
    }

    /// Returns the tile indices of this sprite in 8x16 mode. (top, bottom)
    pub fn tile_index_16(&self) -> (u8, u8) {
        (self.data[2] & 0xfe, self.data[2] | 0x01)
    }

    pub fn palette_number(&self) -> u8 {
        self.data[3] >> 4 & 1
    }

    /// true iff horizontally mirrored
    pub fn x_flip(&self) -> bool {
        self.data[3] >> 5 & 1 == 1
    }

    /// true iff vertically mirrored
    pub fn y_flip(&self) -> bool {
        self.data[3] >> 6 & 1 == 1
    }

    /// false=No, true=BG and Window colors 1-3 over the OBJ
    pub fn bg_window_over_obj(&self) -> bool {
        self.data[3] >> 7 & 1 == 1
    }
}
