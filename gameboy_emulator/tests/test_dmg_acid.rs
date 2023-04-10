mod common;

#[test]
fn dmg_acid_2() {
    common::test_rom_screen_hash(
        "tests/dmg-acid2.gb",
        11164760529020568850,
        2 * 60
    );
}