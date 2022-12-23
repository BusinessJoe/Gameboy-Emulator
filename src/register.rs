use std::{marker::PhantomData, mem::size_of, slice::SliceIndex};

/// The generic parameter should be an integer type
#[derive(Debug, Clone)]
pub struct Register<T> {
    values: Vec<bool>,
    register_type: PhantomData<T>,
}

/// Treats least significant bit as index 0
impl<T> Register<T> {
    pub fn new() -> Self {
        Self {
            values: vec![false; size_of::<T>() * 8],
            register_type: PhantomData,
        }
    }

    pub fn size(&self) -> usize {
        self.values.len()
    }

    pub fn get(&self, index: usize) -> bool {
        self.values[index]
    }

    pub fn set(&mut self, index: usize, value: bool) {
        self.values[index] = value;
    }

    pub fn get_range<U>(&self, indices: U) -> &[bool]
    where
        U: SliceIndex<[bool], Output = [bool]>,
    {
        &self.values[indices]
    }

    pub fn set_range<U>(&mut self, indices: U, values: &[bool])
    where
        U: SliceIndex<[bool], Output = [bool]>,
    {
        self.values[indices].clone_from_slice(values);
    }
}

impl<T> Register<T>
where
    T: num::Zero
        + num::One
        + std::ops::Shl<usize, Output = T>
        + std::ops::Shr<usize, Output = T>
        + std::ops::BitOrAssign
        + std::ops::BitAnd<Output = T>
        + Eq
        + Copy,
{
    pub fn get_range_value<U>(&self, indices: U) -> T
    where
        U: SliceIndex<[bool], Output = [bool]>,
    {
        let mut value = T::zero();
        for (i, flag) in self.values[indices].iter().enumerate() {
            if *flag {
                value |= T::one() << i;
            }
        }
        value
    }

    pub fn set_range_value<U>(&mut self, indices: U, value: T)
    where
        U: SliceIndex<[bool], Output = [bool]>,
    {
        for (i, flag) in self.values[indices].iter_mut().enumerate() {
            *flag = (value >> i) & T::one() == T::one();
        }
    }

    pub fn as_value(&self) -> T {
        self.get_range_value(0..self.size())
    }
}

impl<T> From<T> for Register<T>
where
    T: num::Zero
        + num::One
        + std::ops::Shl<usize, Output = T>
        + std::ops::Shr<usize, Output = T>
        + std::ops::BitOrAssign
        + std::ops::BitAnd<Output = T>
        + Eq
        + Copy,
{
    fn from(value: T) -> Self {
        let mut reg: Register<T> = Self::new();
        reg.set_range_value(0..reg.size(), value);
        reg
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u8_register() {
        let mut reg: Register<u8> = Register::new();
        assert_eq!(0, reg.as_value());

        reg.set(3, true);
        assert_eq!(8, reg.as_value());

        reg.set_range_value(4..=7, 0b1101);
        assert_eq!(0b11011000, reg.as_value());

        assert_eq!(0b01100, reg.get_range_value(1..=5));
    }
}
