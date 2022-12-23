use gameboy_emulator::{execution_manager::ExecutionManager, gameboy::GameBoyState};

fn manager_setup() -> ExecutionManager {
    ExecutionManager::new()
}

fn test_blargg_rom(rom_path: &str) {
    let mut manager = manager_setup();

    let mut gameboy = GameBoyState::new();
    gameboy.load(rom_path);
    gameboy.cpu.boot();

    let result = manager.test(gameboy);
    match result {
        Ok(ref s) => println!("Output:\n{}", s),
        Err(ref s) => println!("Output:\n{}", s),
    }
    assert!(result.is_ok(), "Expected Ok");
}

#[test]
fn test_01_special() {
    test_blargg_rom("tests/gb-test-roms-master/cpu_instrs/individual/01-special.gb");
}

#[test]
fn test_02_interrupts() {
    test_blargg_rom("tests/gb-test-roms-master/cpu_instrs/individual/02-interrupts.gb");
}

#[test]
fn test_03_SP_HL_registers() {
    test_blargg_rom("tests/gb-test-roms-master/cpu_instrs/individual/03-op sp,hl.gb");
}

#[test]
fn test_04_R_IMM() {
    test_blargg_rom("tests/gb-test-roms-master/cpu_instrs/individual/04-op r,imm.gb");
}

#[test]
fn test_05_RP() {
    test_blargg_rom("tests/gb-test-roms-master/cpu_instrs/individual/05-op rp.gb");
}

#[test]
fn test_06_LD_R_R() {
    test_blargg_rom("tests/gb-test-roms-master/cpu_instrs/individual/06-ld r,r.gb");
}

#[test]
fn test_07_JR_JP_CALL_RET_RST() {
    test_blargg_rom("tests/gb-test-roms-master/cpu_instrs/individual/07-jr,jp,call,ret,rst.gb");
}

#[test]
fn test_08_misc_instrs() {
    test_blargg_rom("tests/gb-test-roms-master/cpu_instrs/individual/08-misc instrs.gb");
}

#[test]
fn test_09_OP_R_R() {
    test_blargg_rom("tests/gb-test-roms-master/cpu_instrs/individual/09-op r,r.gb");
}

#[test]
fn test_10_bit_ops() {
    test_blargg_rom("tests/gb-test-roms-master/cpu_instrs/individual/10-bit ops.gb");
}

#[test]
fn test_11_OP_A_HL() {
    test_blargg_rom("tests/gb-test-roms-master/cpu_instrs/individual/11-op a,(hl).gb");
}
