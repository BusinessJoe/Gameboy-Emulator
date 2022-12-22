use pixels::{Pixels, SurfaceTexture};
use winit::{
        dpi::LogicalSize, event_loop::EventLoop, window::{Window, WindowBuilder},
};
use std::fmt;

#[derive(Debug, Clone)]
pub struct IndexError;

impl fmt::Display for IndexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "index out of screen bounds")
    }
}

pub trait Screen<T> {
    fn set_pixel(&mut self, row: u32, col: u32, value: &T) -> Result<(), IndexError>;
    fn get_pixel(&self, row: u32, col: u32) -> Result<&T, IndexError>;
    fn redraw(&mut self);
}

pub struct PixelsScreen {
    pixels: Pixels,
    pixel_data: Vec<u8>,
    width: u32,
    height: u32,
    _window: Window,
}

impl PixelsScreen {
    pub fn new(logical_width: u32, logical_height: u32, real_width: u32, real_height: u32, event_loop: &EventLoop<()>) -> Self {
        let window = {
            let size = LogicalSize::new(logical_width as f64, logical_height as f64);
            let real_size = LogicalSize::new(real_width as f64, real_height as f64);
            WindowBuilder::new()
                .with_title("GameBoy Screen")
                .with_inner_size(real_size)
                .with_min_inner_size(size)
                .build(event_loop)
                .unwrap()
        };

        let pixels = {
            let window_size = window.inner_size();
            let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
            Pixels::new(logical_width, logical_height, surface_texture).unwrap()
        };

        let size: usize = (logical_width * logical_height * 4).try_into().unwrap();
        Self {
            width: logical_width,
            height: logical_height,
            pixel_data: vec![0; size],
            pixels,
            _window: window,
        }
    }

    fn get_array_index(&self, row: u32, col: u32) -> Result<usize, IndexError> {
        let index: usize = (self.width * col + row).try_into().unwrap();
        if index >= (self.width * self.height).try_into().unwrap() {
            return Err(IndexError);
        }
        // Multiply index by 4 since 4 bytes are used per pixel (RGBA)
        Ok(index * 4)
    }
}

impl Screen<[u8; 4]> for PixelsScreen {
    fn set_pixel(&mut self, row: u32, col: u32, value: &[u8; 4]) -> Result<(), IndexError> {
        let index = self.get_array_index(row, col)?;
        self.pixel_data[index..index+4].copy_from_slice(value);
        Ok(())
    }

    fn get_pixel(&self, row: u32, col: u32) -> Result<&[u8; 4], IndexError> {
        let index = self.get_array_index(row, col)?;
        let rgba: &[u8; 4] = self.pixel_data[index..index+4].try_into().unwrap();
        Ok(rgba)
    }

    fn redraw(&mut self) {
        self.pixels.get_frame_mut().copy_from_slice(&self.pixel_data);

        if let Err(err) = self.pixels.render() {
            dbg!(err);
        }
    }
}
