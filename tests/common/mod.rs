use std::{thread, time::Duration};

use gameboy_emulator::{
    cartridge::Cartridge,
    emulator::{events::EmulationEvent, GameboyEmulator},
};

pub fn test_rom(path: &str, target_serial_data: &[u8], mut timeout_duration: Duration) {
    let bytes = std::fs::read(path).unwrap();
    let cartridge = Cartridge::cartridge_from_data(&bytes).expect("failed to build cartridge");

    let (control_event_sender, event_receiver) =
        GameboyEmulator::gameboy_thread_no_gui(cartridge).unwrap();

    let sleep_duration = Duration::from_secs(1);

    let mut serial_port_output = Vec::new();
    let mut pass = false;

    'outer: while !timeout_duration.is_zero() {
        thread::sleep(sleep_duration.clone());
        timeout_duration = timeout_duration.saturating_sub(sleep_duration);

        while let Ok(event) = event_receiver.try_recv() {
            if let EmulationEvent::SerialData(byte) = event {
                serial_port_output.push(byte);
                if serial_port_output.ends_with(target_serial_data) {
                    pass = true;
                    break 'outer;
                }
                print!("{}", byte as char);
            }
        }
    }
    assert!(pass);
}
