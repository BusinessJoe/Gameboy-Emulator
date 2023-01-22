mod perf;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use gameboy_emulator::cartridge::{Address, AddressingError, Cartridge, MBCControllerType};
use gameboy_emulator::cpu::CPU;
use gameboy_emulator::gameboy::GameBoyState;
use gameboy_emulator::{Joypad, MemoryBus, CanvasPpu, Ppu};
use std::cell::RefCell;
use std::rc::Rc;

fn repeat_regular_opcode(c: &mut Criterion, name: &str, opcode: u8) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Gameboy Emulator", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string()).unwrap();

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string()).unwrap();
    let creator = canvas.texture_creator();
    let canvas_ppu = Rc::new(RefCell::new(CanvasPpu::new(&creator)));

    let mut cpu = CPU::new();
    let joypad = Rc::new(RefCell::new(Joypad::new()));
    let mut memory_bus = MemoryBus::new(canvas_ppu as Rc<RefCell<dyn Ppu>>, joypad);

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
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Gameboy Emulator", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string()).unwrap();

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string()).unwrap();
    let creator = canvas.texture_creator();
    let canvas_ppu = Rc::new(RefCell::new(CanvasPpu::new(&creator)));

    let mut gameboy = GameBoyState::new(canvas_ppu);
    let cart = MockCartridge::new(vec![0; 32 * 1024]);
    gameboy.load_cartridge(Box::new(cart)).unwrap();

    c.bench_function("gameboy tick", |b| {
        b.iter(|| {
            black_box(gameboy.tick());
            gameboy.cpu.borrow_mut().pc = 0x100;
        });
    });
}

/// A ROM-only cartridge that wraps around a vector of bytes
struct MockCartridge {
    data: Vec<u8>,
}

impl MockCartridge {
    pub fn new(data: Vec<u8>) -> MockCartridge {
        MockCartridge { data }
    }
}

impl Cartridge for MockCartridge {
    fn mbc_controller_type(&self) -> MBCControllerType {
        MBCControllerType::RomOnly
    }

    fn read(&self, address: Address) -> Result<u8, AddressingError> {
        self.data
            .get(address)
            .ok_or(AddressingError(address))
            .copied()
    }

    fn write(&mut self, address: Address, value: u8) -> Result<(), AddressingError> {
        if let Some(elem) = self.data.get_mut(address) {
            *elem = value;
            Ok(())
        } else {
            Err(AddressingError(address))
        }
    }
}

criterion_group! {
    name = gameboy_benches;
    config = Criterion::default().with_profiler(perf::FlamegraphProfiler::new(100)).sample_size(500);
    targets = repeat_nop, repeat_inc_b_reg, bench_gameboy_tick
}

criterion_main!(gameboy_benches);
