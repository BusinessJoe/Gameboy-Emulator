/// Events created by the emulator and broadcasted across a channel
#[derive(Debug)]
pub enum EmulationEvent {
    SerialData(u8),
}

/// Events sent to the emulator to control its status
#[derive(Debug)]
pub enum EmulationControlEvent {
    Quit,
}
