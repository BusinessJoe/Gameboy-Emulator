mod common;

fn test_mooneye_rom(path: &str, num_frames: u64) {
    common::test_rom_serial_data(path, &[3, 5, 8, 13, 21, 34], num_frames)
}

mod test_mooneye {
    use crate::test_mooneye_rom;

    #[test]
    fn test_boot_regs() {
        test_mooneye_rom("tests/mooneye/acceptance/boot_regs-dmgABC.gb", 1 * 60);
    }

    #[test]
    fn test_boot_div() {
        test_mooneye_rom("tests/mooneye/acceptance/boot_div-dmgABCmgb.gb", 1 * 60);
    }
}
