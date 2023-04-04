pub struct BitField(pub u8);

impl BitField {
    pub fn get_bit(&self, index: usize) -> Result<bool, ()> {
        if index >= 8 {
            Err(())
        } else {
            Ok(self.0 & (1 << index) != 0)
        }
    }

    pub fn set_bit(&mut self, index: usize, value: bool) -> Result<(), ()> {
        if index >= 8 {
            return Err(());
        }

        if value {
            self.0 |= 1 << index;
        } else {
            self.0 &= !(1 << index);
        }

        Ok(())
    }
}
