use crate::error::Result;

pub type Address = usize;

pub type ElapsedTime = u32;
pub type TickCount = u128;

pub trait Addressable {
    fn read_u8(&mut self, address: Address) -> Result<u8>;

    fn write_u8(&mut self, address: Address, data: u8) -> Result<()>;
}

pub trait Steppable {
    type Context;

    fn step(&mut self, context: &mut Self::Context, elapsed: u32) -> Result<ElapsedTime>;
}

pub trait BatchSteppable {
    type Context;

    fn fast_forward(&mut self, context: &mut Self::Context, current_time: TickCount) -> Result<()>;
}