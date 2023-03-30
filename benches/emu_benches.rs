mod perf;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use gameboy_emulator::cartridge::Cartridge;

use gameboy_emulator::emulator::events::EmulationControlEvent;
use gameboy_emulator::gameboy::GameBoyState;
use gameboy_emulator::texture::TextureBook;
use gameboy_emulator::ppu::{NoGuiEngine, BasePpu, CanvasEngine};
use sdl2::render::BlendMode;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;

fn repeat_regular_opcode(c: &mut Criterion, name: &str, opcode: u8) {
    let (event_sender, event_receiver) = mpsc::channel();
    let (control_event_sender, _control_event_receiver) =
        mpsc::channel::<EmulationControlEvent>();

    let graphics_engine = Box::new(NoGuiEngine {});
    let ppu = Rc::new(RefCell::new(BasePpu::new(graphics_engine)));

    let mut gameboy_state = GameBoyState::new(ppu.clone(), event_sender);
    
    let mut cpu = gameboy_state.cpu.borrow_mut();
    let mut memory_bus = gameboy_state.memory_bus.borrow_mut();

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
    let (event_sender, event_receiver) = mpsc::channel();
    let (control_event_sender, _control_event_receiver) =
        mpsc::channel::<EmulationControlEvent>();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Gameboy Emulator", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())
        .unwrap();

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    canvas
        .set_logical_size((20 + 1 + 32) * 8, (32 + 1 + 32) * 8)
        .map_err(|e| e.to_string())?;
    canvas.set_blend_mode(BlendMode::Blend);
    let mut texture_book = TextureBook::new(&canvas)?;

    let canvas = Rc::new(RefCell::new(canvas));

    let graphics_engine = Box::new(CanvasEngine::new(&texture_book.texture_creator)?);
    let ppu = Rc::new(RefCell::new(BasePpu::new(graphics_engine)));


    let mut gameboy_state = GameBoyState::new(ppu.clone(), event_sender);

    let cart = Cartridge::mock();
    gameboy_state
                .load_cartridge(cart)
                .map_err(|e| e.to_string())?;

    c.bench_function("gameboy tick", |b| {
        b.iter(|| {
            black_box(gameboy_state.tick());
            // gameboy.cpu.borrow_mut().pc = 0x100;
        });
    });
}

criterion_group! {
    name = gameboy_benches;
    config = Criterion::default().with_profiler(perf::FlamegraphProfiler::new(100)).sample_size(500);
    targets = repeat_nop, repeat_inc_b_reg, bench_gameboy_tick
}

criterion_main!(gameboy_benches);
