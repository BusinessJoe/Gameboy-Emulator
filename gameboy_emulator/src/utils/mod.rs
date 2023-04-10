pub struct BitField(pub u8);

impl BitField {
    pub fn get_bit(&self, index: usize) -> bool {
        debug_assert!(index < 8);
        self.0 & (1 << index) != 0
    }

    pub fn set_bit(&mut self, index: usize, value: bool) {
        debug_assert!(index < 8);

        if value {
            self.0 |= 1 << index;
        } else {
            self.0 &= !(1 << index);
        }
    }
}
