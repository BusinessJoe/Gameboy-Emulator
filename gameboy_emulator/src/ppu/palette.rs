pub struct PaletteRegister {
    pub register_value: u8, // ff47, ff48, or ff49
}

#[derive(Clone, PartialEq, Eq)]
pub enum TileColor {
    White,
    LightGrey,
    DarkGrey,
    Black,
    Debug,
}

impl TileColor {
    pub fn from_u8(color: u8) -> TileColor {
        match color {
            0 => TileColor::White,
            1 => TileColor::LightGrey,
            2 => TileColor::DarkGrey,
            3 => TileColor::Black,
            _ => TileColor::Debug,
        }
    }
    pub fn to_u8(&self) -> u8 {
        match &self {
            TileColor::White => 0,
            TileColor::LightGrey => 1,
            TileColor::DarkGrey => 2,
            TileColor::Black => 3,
            TileColor::Debug => 4,
        }
    }
}

pub enum SpriteTileColor {
    Transparent,
    TileColor(TileColor),
}

impl PaletteRegister {
    pub fn map_index(&self, index: u8) -> TileColor {
        let color = match index {
            0 => self.register_value & 0b11,
            1 => (self.register_value >> 2) & 0b11,
            2 => (self.register_value >> 4) & 0b11,
            3 => (self.register_value >> 6) & 0b11,
            _ => 4,
        };
        TileColor::from_u8(color)
    }

    pub fn map_sprite_index(&self, index: u8) -> SpriteTileColor {
        let color = match index {
            0 => return SpriteTileColor::Transparent,
            1 => (self.register_value >> 2) & 0b11,
            2 => (self.register_value >> 4) & 0b11,
            3 => (self.register_value >> 6) & 0b11,
            _ => 4,
        };
        SpriteTileColor::TileColor(TileColor::from_u8(color))
    }
}
