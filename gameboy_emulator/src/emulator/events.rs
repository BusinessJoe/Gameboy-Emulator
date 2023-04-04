use crate::gameboy::GameboyDebugInfo;

/// Events created by the emulator and broadcasted across a channel
#[derive(Debug, Clone)]
pub enum EmulationEvent {
    SerialData(u8),
    Trace(GameboyDebugInfo),
    MemoryRead { address: usize, value: u8 },
    MemoryWrite { address: usize, value: u8 },
}

/// Events sent to the emulator to control its status
#[derive(Debug)]
pub enum EmulationControlEvent {
    Quit,
}
