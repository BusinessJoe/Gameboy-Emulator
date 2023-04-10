use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use gameboy_emulator::{cartridge::Cartridge, gameboy::GameBoyState};

pub fn test_rom_serial_data(path: &str, target_serial_data: &[u8], num_frames: u64) {
    let bytes = std::fs::read(path).unwrap();
    let cartridge = Cartridge::cartridge_from_data(&bytes).expect("failed to build cartridge");

    let mut gameboy = GameBoyState::new();
    gameboy.load_cartridge(cartridge).unwrap();

    for _ in 0..num_frames {
        gameboy.tick_for_frame();
    }

    let binding = gameboy.get_memory_bus();
    let binding = binding.borrow();
    let serial_port_data = binding.get_serial_port_data();
    dbg!(serial_port_data);
    if find_subsequence(serial_port_data, target_serial_data).is_none() {
        eprintln!("{}", std::str::from_utf8(serial_port_data).unwrap());
        panic!("target serial data not found");
    }
}

// https://stackoverflow.com/a/35907071
fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

pub fn test_rom_screen_hash(path: &str, target_hash: u64, num_frames: u64) {
    let bytes = std::fs::read(path).unwrap();
    let cartridge = Cartridge::cartridge_from_data(&bytes).expect("failed to build cartridge");

    let mut gameboy = GameBoyState::new();
    gameboy.load_cartridge(cartridge).unwrap();

    for _ in 0..num_frames {
        gameboy.tick_for_frame();
    }

    let hash = gameboy.get_screen_hash();

    if hash != target_hash {
        panic!(
            "Incorrect screen hash. Expected {}, actual {}",
            target_hash, hash
        );
    }
}
