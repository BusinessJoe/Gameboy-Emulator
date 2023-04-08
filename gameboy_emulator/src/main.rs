use gameboy_emulator::{cartridge::Cartridge, gameboy::Interrupt};
use gameboy_emulator::gameboy::GameBoyState;
use gameboy_emulator::joypad::JoypadInput;
use gameboy_emulator::ppu::TileColor;
use js_sys::Intl::Collator;
use sdl2::pixels::Color;
use strum::IntoEnumIterator;
use log::*;
use sdl2;
use sdl2::keyboard::Keycode;
use sdl2::render::{BlendMode, Canvas};
use sdl2::event::Event;
use sdl2::video::Window;

use std::time::{Duration, Instant};
use std::fs;
use std::sync::mpsc;

use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to .gb rom file
    #[arg(short = 'r', long = "rom", required = true)]
    rom_path: String,

    /// Debug mode
    #[arg(short, long, default_value_t = false)]
    debug: bool,
}

fn main() -> Result<(), String> {
    env_logger::init();

    let args = Args::parse();

    let bytes = fs::read(args.rom_path).expect("could not read file");
    let cartridge = Cartridge::cartridge_from_data(&bytes).expect("failed to build cartridge");

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("Gameboy Emulator", 1800, 800)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    canvas
        .set_logical_size((20) * 8, (18) * 8)
        .map_err(|e| e.to_string())?;
    canvas.set_blend_mode(BlendMode::Blend);
    
    let (event_sender, event_receiver) = mpsc::channel();

    let mut gameboy_state = GameBoyState::new(event_sender);
    gameboy_state
                .load_cartridge(cartridge)
                .map_err(|e| e.to_string())?;

    let joypad = gameboy_state.get_joypad();
    let memory_bus = gameboy_state.get_memory_bus();
    let ppu = gameboy_state.get_ppu();

    let mut total_cycles = 0;
    let mut frame_cycles = 0;
    let mut start = Instant::now();
    'mainloop: loop {
        for event in sdl_context.event_pump().unwrap().poll_iter() {
            match event {
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                }
                | Event::Quit { .. } => {
                    break 'mainloop;
                }
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    for joypad_input in JoypadInput::iter() {
                        if map_joypad_to_keys(joypad_input).contains(&keycode) {
                            let prev_state = joypad.borrow_mut().key_pressed(joypad_input);
                            // If previous state was not pressed, we send interrupt
                            if !prev_state {
                                memory_bus
                                    .borrow_mut()
                                    .interrupt(Interrupt::Joypad)
                                    .expect("error sending joypad interrupt");
                            }
                        }
                    }
                }
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => {
                    for joypad_input in JoypadInput::iter() {
                        if map_joypad_to_keys(joypad_input).contains(&keycode) {
                            joypad.borrow_mut().key_released(joypad_input);
                        }
                    }
                }
                _ => {}
            }
        }

        // Tick gameboy for a frame
        let elapsed_cycles = gameboy_state.tick_for_frame();
        total_cycles += elapsed_cycles;
        frame_cycles += elapsed_cycles;

        render_screen(gameboy_state.get_screen(), &mut canvas);

        let duration = start.elapsed();
        if duration > Duration::from_millis(1000 / 60) {
            warn!("Time elapsed this frame is: {:?} > 16ms", duration);
        } else {
            std::thread::sleep(Duration::from_millis(1000 / 60) - duration);
        }
        start = Instant::now();
    }
    Ok(())
}


// Maps keyboard keys to corresponding joypad inputs.
fn map_joypad_to_keys(input: JoypadInput) -> Vec<Keycode> {
    match input {
        JoypadInput::A => vec![Keycode::A],
        JoypadInput::B => vec![Keycode::B],
        JoypadInput::Start => vec![Keycode::Space],
        JoypadInput::Select => vec![Keycode::Return],
        JoypadInput::Up => vec![Keycode::Up],
        JoypadInput::Down => vec![Keycode::Down],
        JoypadInput::Left => vec![Keycode::Left],
        JoypadInput::Right => vec![Keycode::Right],
    }
}


fn render_screen(screen: Vec<TileColor>, canvas: &mut Canvas<Window>) {
    for x in 0 .. 160 {
        for y in 0 .. 144 {
            let color = match screen[x + y * 160] {
                TileColor::White => Color::RGB(255, 255, 255),
                TileColor::LightGrey => Color::RGB(200, 200, 200),
                TileColor::DarkGrey => Color::RGB(100, 100, 100),
                TileColor::Black => Color::RGB(0, 0, 0),
                TileColor::Debug => Color::RGB(255, 0, 0),
            };
            canvas.set_draw_color(color);
            canvas.draw_point((x as i32, y as i32));
        }
    }
    canvas.present();
}