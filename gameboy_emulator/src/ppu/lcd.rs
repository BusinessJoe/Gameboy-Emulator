use crate::MemoryBus;
use crate::error::Result;
use crate::gameboy::GameBoyState;
use crate::gameboy::Interrupt;
use crate::utils::BitField;

use super::base_ppu::PpuState;

/// Represents the LCD Control register at 0xff40
#[derive(Debug, Clone, Copy)]
pub struct LcdControl {
    pub bg_window_enable: bool,
    pub obj_enable: bool,
    pub obj_size: bool,
    pub bg_tile_map_area: bool,
    pub bg_window_tile_data_area: bool,
    pub window_enable: bool,
    pub window_tile_map_area: bool,
    pub lcd_ppu_enable: bool,
}

impl LcdControl {
    pub fn new() -> Self {
        Self {
            bg_window_enable: false,
            obj_enable: false,
            obj_size: false,
            bg_tile_map_area: false,
            bg_window_tile_data_area: false,
            window_enable: false,
            window_tile_map_area: false,
            lcd_ppu_enable: false,
        }
    }

    pub fn read(&self) -> u8 {
        (self.bg_window_enable as u8)
            | (self.obj_enable as u8) << 1
            | (self.obj_size as u8) << 2
            | (self.bg_tile_map_area as u8) << 3
            | (self.bg_window_tile_data_area as u8) << 4
            | (self.window_enable as u8) << 5
            | (self.window_tile_map_area as u8) << 6
            | (self.lcd_ppu_enable as u8) << 7
    }

    pub fn write(&mut self, value: u8) {
        self.bg_window_enable = (value >> 0) & 1 == 1;
        self.obj_enable = (value >> 1) & 1 == 1;
        self.obj_size = (value >> 2) & 1 == 1;
        self.bg_tile_map_area = (value >> 3) & 1 == 1;
        self.bg_window_tile_data_area = (value >> 4) & 1 == 1;
        self.window_enable = (value >> 5) & 1 == 1;
        self.window_tile_map_area = (value >> 6) & 1 == 1;
        self.lcd_ppu_enable = (value >> 7) & 1 == 1;
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum PpuScanlineState {
    OamSearch,
    PixelTransfer,
    VBlank,
    HBlank,
}

/// Struct returned to indicate the pixel with these coords should be updated
pub struct UpdatePixel {
    pub x: u8,
    pub y: u8,
}
pub struct Lcd {
    /// LY: LCD Y coordinate (read only)
    pub ly: u8,
    /// LYC: LY compare
    pub lyc: u8,
    /// Current x position in scanline
    pub scan_x: u8,
    pub lcd_control: LcdControl,
    pub stat: BitField,
    stat_interrupt_line: [bool; 4],

    state: PpuScanlineState,
    dots: u32,
    pub window_line_counter: u8,

    /// Count of number of elapsed frames since initialization
    pub frame_count: u128,
}

impl Lcd {
    pub fn new() -> Lcd {
        Lcd {
            ly: 0,
            lyc: 0,
            scan_x: 0,
            lcd_control: LcdControl::new(),
            stat: BitField(0),
            stat_interrupt_line: [false; 4],
            state: PpuScanlineState::OamSearch,
            dots: 0,
            window_line_counter: 0,
            frame_count: 0,
        }
    }
}

impl Lcd {
    fn increment_ly(&mut self, wx: u8, wy: u8) -> Option<Interrupt> {
        let window_enable = self.lcd_control.window_enable;
        if window_enable && self.ly >= wy && wx <= 166 {
            self.window_line_counter += 1;
        }
        
        self.ly += 1;


        if self.ly == 153 {
            self.ly = 0;
            self.window_line_counter = 0;
        }

        self.check_ly_equals_lyc()
    }

    pub fn check_ly_equals_lyc(&mut self) -> Option<Interrupt> {
        if self.ly == self.lyc && self.stat.get_bit(6) {
            self.update_stat_interrupt_line(3, true)
        } else {
            self.update_stat_interrupt_line(3, false)
        }
    }

    fn change_state(&mut self, new_state: PpuScanlineState) -> Option<Interrupt> {
        self.state = new_state;

        self.update_stat_interrupt_line(
            0,
            self.stat.get_bit(3) && new_state == PpuScanlineState::HBlank,
        )
        .or(self.update_stat_interrupt_line(
            1,
            self.stat.get_bit(4) && new_state == PpuScanlineState::VBlank,
        ))
        .or(self.update_stat_interrupt_line(
            2,
            self.stat.get_bit(5) && new_state == PpuScanlineState::OamSearch,
        ))
    }

    fn update_stat_interrupt_line(&mut self, index: usize, value: bool) -> Option<Interrupt> {
        if self.stat_interrupt_line[index] == value {
            return None;
        }

        if !value {
            self.stat_interrupt_line[index] = value;
            return None;
        } else {
            // Or all the stat interrupt line values together
            let old_line_value = self.stat_interrupt_line[0] 
                | self.stat_interrupt_line[1] 
                | self.stat_interrupt_line[2] 
                | self.stat_interrupt_line[3];
            self.stat_interrupt_line[index] = value;

            // Since the `value` parameter went from low to high here,
            // if the old line value is false, then this change causes the stat line to go high
            // and we should send an interrupt
            if !old_line_value {
                Some(Interrupt::Stat)
            } else {
                None
            }
        }
    }

    pub fn step(&mut self, memory_bus: &mut MemoryBus, wx: u8, wy: u8) -> Result<Option<UpdatePixel>> {
        let mut pixel_data = None;
        self.dots += 1;

        match self.state {
            PpuScanlineState::OamSearch => {
                if self.dots == 80 {
                    self.change_state(PpuScanlineState::PixelTransfer);
                }
            }
            PpuScanlineState::PixelTransfer => {
                // TODO: Fetch pixel data into our pixel FIFO.
                // TODO: Put a pixel (if any) from the FIFO on screen.

                // For now, just use the current xy coordinates as an index into the background map
                // to get a pixel
                pixel_data = Some(UpdatePixel {
                    x: self.scan_x,
                    y: self.ly.into(),
                });

                self.scan_x += 1;
                if self.scan_x == 160 {
                    self.scan_x = 0;
                    if let Some(interrupt) = self.change_state(PpuScanlineState::HBlank) {
                        memory_bus.interrupt(interrupt)?;
                    }
                }
            }
            PpuScanlineState::HBlank => {
                if self.dots == 456 {
                    self.dots = 0;
                    if let Some(interrupt) = self.increment_ly(wx, wy) {
                        memory_bus.interrupt(interrupt)?;
                    }
                    if self.ly == 144 {
                        if let Some(interrupt) = self.change_state(PpuScanlineState::VBlank) {
                            memory_bus.interrupt(interrupt)?;
                        }
                        memory_bus.interrupt(Interrupt::VBlank)?;
                        self.frame_count += 1;
                    } else {
                        if let Some(interrupt) = self.change_state(PpuScanlineState::OamSearch) {
                            memory_bus.interrupt(interrupt)?;
                        }
                    }
                }
            }
            PpuScanlineState::VBlank => {
                if self.dots == 456 {
                    self.dots = 0;
                    if let Some(interrupt) = self.increment_ly(wx, wy) {
                        memory_bus.interrupt(interrupt)?;
                    }
                    if self.ly == 0 {
                        //println!("End VBLANK");
                        if let Some(interrupt) = self.change_state(PpuScanlineState::OamSearch) {
                            memory_bus.interrupt(interrupt)?;
                        }
                    }
                }
            }
        }

        Ok(pixel_data)
    }
}
