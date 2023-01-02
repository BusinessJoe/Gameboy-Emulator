use crate::{error::Result, gameboy::GameBoyState};

pub type Address = usize;

pub type ElapsedTime = u64;

pub trait Addressable {
    fn read(&mut self, address: Address, data: &mut [u8]) -> Result<()>;
    fn write(&mut self, address: Address, data: &[u8]) -> Result<()>;

    fn read_u8(&mut self, address: Address) -> Result<u8> {
        let mut data = [0];
        self.read(address, &mut data)?;
        Ok(data[0])
    }

    fn write_u8(&mut self, address: Address, data: u8) -> Result<()> {
        let data = [data];
        self.write(address, &data)
    }
}

pub trait Steppable {
    fn step(&mut self, state: &GameBoyState) -> Result<ElapsedTime>;
}

pub trait Component {
    fn as_addressable(&mut self) -> Option<Box<dyn Addressable>> {
        None
    }

    fn as_steppable(&mut self) -> Option<Box<dyn Steppable>> {
        None
    }
}

pub type ComponentBox = Box<dyn Component>;
