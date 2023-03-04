mod common;

use std::time::Duration;

#[test]
fn test_blargg_cpu_all() {
    common::test_rom(
        "tests/blargg/gb-test-roms-master/cpu_instrs/cpu_instrs.gb",
        "Passed".as_bytes(),
        Duration::from_secs(120),
    );
}

#[test]
fn test_blargg_cpu_01() {
    common::test_rom(
        "tests/blargg/gb-test-roms-master/cpu_instrs/individual/01-special.gb",
        "Passed".as_bytes(),
        Duration::from_secs(10),
    );
}

#[test]
fn test_blargg_cpu_02() {
    common::test_rom(
        "tests/blargg/gb-test-roms-master/cpu_instrs/individual/02-interrupts.gb",
        "Passed".as_bytes(),
        Duration::from_secs(10),
    );
}

#[test]
fn test_blargg_cpu_03() {
    common::test_rom(
        "tests/blargg/gb-test-roms-master/cpu_instrs/individual/03-op sp,hl.gb",
        "Passed".as_bytes(),
        Duration::from_secs(10),
    );
}

#[test]
fn test_blargg_cpu_04() {
    common::test_rom(
        "tests/blargg/gb-test-roms-master/cpu_instrs/individual/04-op r,imm.gb",
        "Passed".as_bytes(),
        Duration::from_secs(10),
    );
}

#[test]
fn test_blargg_cpu_05() {
    common::test_rom(
        "tests/blargg/gb-test-roms-master/cpu_instrs/individual/05-op rp.gb",
        "Passed".as_bytes(),
        Duration::from_secs(10),
    );
}

#[test]
fn test_blargg_cpu_06() {
    common::test_rom(
        "tests/blargg/gb-test-roms-master/cpu_instrs/individual/06-ld r,r.gb",
        "Passed".as_bytes(),
        Duration::from_secs(10),
    );
}

#[test]
fn test_blargg_cpu_07() {
    common::test_rom(
        "tests/blargg/gb-test-roms-master/cpu_instrs/individual/07-jr,jp,call,ret,rst.gb",
        "Passed".as_bytes(),
        Duration::from_secs(10),
    );
}

#[test]
fn test_blargg_cpu_08() {
    common::test_rom(
        "tests/blargg/gb-test-roms-master/cpu_instrs/individual/08-misc instrs.gb",
        "Passed".as_bytes(),
        Duration::from_secs(10),
    );
}

#[test]
fn test_blargg_cpu_09() {
    common::test_rom(
        "tests/blargg/gb-test-roms-master/cpu_instrs/individual/09-op r,r.gb",
        "Passed".as_bytes(),
        Duration::from_secs(20),
    );
}

#[test]
fn test_blargg_cpu_10() {
    common::test_rom(
        "tests/blargg/gb-test-roms-master/cpu_instrs/individual/10-bit ops.gb",
        "Passed".as_bytes(),
        Duration::from_secs(20),
    );
}
#[test]
fn test_blargg_cpu_11() {
    common::test_rom(
        "tests/blargg/gb-test-roms-master/cpu_instrs/individual/11-op a,(hl).gb",
        "Passed".as_bytes(),
        Duration::from_secs(30),
    );
}
