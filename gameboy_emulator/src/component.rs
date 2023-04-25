use crate::{error::Result, gameboy::GameBoyState};

pub type Address = usize;

pub type ElapsedTime = u32;

pub trait Addressable {
    fn read_u8(&mut self, address: Address) -> Result<u8>;

    fn write_u8(&mut self, address: Address, data: u8) -> Result<()>;
}

pub trait Steppable {
    fn step(&mut self, state: &GameBoyState, elapsed: u32) -> Result<ElapsedTime>;
}

pub trait Component {
    fn as_addressable(&mut self) -> Option<Box<dyn Addressable>> {
        None
    }

    fn as_steppable(&mut self) -> Option<Box<dyn Steppable>> {
        None
    }
}
