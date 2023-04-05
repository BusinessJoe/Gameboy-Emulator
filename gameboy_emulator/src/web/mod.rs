pub mod mainloop;

pub struct Screen {
    // Might be able to make this smaller by packing 4 color values into each integer
    pixels: [u8; 160 * 144]
}