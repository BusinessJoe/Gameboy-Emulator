mod perf;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use gameboy_emulator::cartridge::Cartridge;

use gameboy_emulator::emulator::events::EmulationControlEvent;
use gameboy_emulator::gameboy::GameBoyState;
use gameboy_emulator::ppu::BasePpu;
use std::cell::RefCell;
use std::rc::Rc;

fn repeat_regular_opcode(c: &mut Criterion, name: &str, opcode: u8) {
    let mut gameboy_state = GameBoyState::new();

    let binding = gameboy_state.get_cpu();
    let mut cpu = binding.borrow_mut();
    let binding = gameboy_state.get_memory_bus();
    let mut memory_bus = binding.borrow_mut();

    c.bench_function(name, |b| {
        b.iter(|| cpu.execute_regular_opcode(&mut memory_bus, black_box(opcode)))
    });
}

fn repeat_nop(c: &mut Criterion) {
    repeat_regular_opcode(c, "nop", 0x00);
}

fn repeat_inc_b_reg(c: &mut Criterion) {
    repeat_regular_opcode(c, "inc-b", 0x04);
}

fn bench_gameboy_tick(c: &mut Criterion) {
    let mut gameboy_state = GameBoyState::new();

    let cart = Cartridge::mock();
    gameboy_state.load_cartridge(cart).unwrap();

    c.bench_function("gameboy tick", |b| {
        b.iter(|| {
            black_box(gameboy_state.tick());
            //gameboy.cpu.borrow_mut().pc = 0x100;
        });
    });
}

fn bench_blargg_cpu_instrs(c: &mut Criterion) {
    let mut gameboy_state = GameBoyState::new();

    let bytes = include_bytes!("cpu_instrs.gb");
    let cart = Cartridge::cartridge_from_data(bytes).unwrap();
    gameboy_state.load_cartridge(cart).unwrap();

    c.bench_function("gameboy tick", |b| {
        b.iter(|| {
            black_box(gameboy_state.tick());
            // gameboy.cpu.borrow_mut().pc = 0x100;
        });
    });
}

fn bench_blargg_cpu_instrs_frame(c: &mut Criterion) {
    let mut gameboy_state = GameBoyState::new();

    let bytes = include_bytes!("cpu_instrs.gb");
    let cart = Cartridge::cartridge_from_data(bytes).unwrap();
    gameboy_state.load_cartridge(cart).unwrap();

    c.bench_function("gameboy tick for frame", |b| {
        b.iter(|| {
            black_box(gameboy_state.tick_for_frame());
            black_box(gameboy_state.get_queued_audio());
            // gameboy.cpu.borrow_mut().pc = 0x100;
        });
    });
}

criterion_group! {
    name = gameboy_benches;
    config = Criterion::default().with_profiler(perf::FlamegraphProfiler::new(1000)).sample_size(500);
    targets = bench_blargg_cpu_instrs_frame
}

criterion_main!(gameboy_benches);
