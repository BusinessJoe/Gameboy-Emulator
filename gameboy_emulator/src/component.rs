use crate::{error::Result, gameboy::GameBoyState};

pub type Address = usize;

pub type ElapsedTime = u32;
pub type NextUpdate = u128;

pub trait Addressable {
    fn read_u8(&mut self, address: Address) -> Result<u8>;

    fn write_u8(&mut self, address: Address, data: u8) -> Result<()>;
}

pub trait Steppable {
    type Context;

    fn step(&mut self, context: &mut Self::Context, elapsed: u32) -> Result<ElapsedTime>;
}

pub trait BatchSteppable {
    fn batch_step(&mut self, state: &GameBoyState, current_time: u128) -> Result<NextUpdate>;
}