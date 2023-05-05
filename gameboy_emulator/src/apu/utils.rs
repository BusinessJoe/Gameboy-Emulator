/// Linearly convert an integer value in the range 0x0 - 0xf to a float from -1 to +1.
pub fn digital_to_analog(value: u8) -> f32 {
    (f32::from(value) / 7.5) - 1.
}

#[cfg(test)]
mod tests {
    use num::abs;

    use super::digital_to_analog;

    #[test]
    fn d_to_a() {
        assert_eq!(digital_to_analog(0x0), -1.0);
        assert!(abs(digital_to_analog(0x1) + 0.8667) < 0.001);
        assert!(abs(digital_to_analog(0x2) + 0.7333) < 0.001);
        assert!(abs(digital_to_analog(0x3) + 0.6) < 0.001);
        assert_eq!(digital_to_analog(0xf), 1.0);
    }
}
