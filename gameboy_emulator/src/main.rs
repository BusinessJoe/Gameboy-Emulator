use gameboy_emulator::gameboy::GameBoyState;
use gameboy_emulator::joypad::JoypadInput;
use gameboy_emulator::ppu::TileColor;
use gameboy_emulator::{cartridge::Cartridge, gameboy::Interrupt};
use log::*;
use ringbuf::Rb;
use sdl2::audio::AudioQueue;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::{BlendMode, Canvas};
use sdl2::video::Window;
use sdl2::{self, audio};
use strum::IntoEnumIterator;

use std::fs::{self, File};
use std::path::Path;
use std::time::{Duration, Instant};

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
    print!("{}", &cartridge);

    let sdl_context = sdl2::init()?;

    let mut canvas = init_sdl2_video(&sdl_context)?;
    let audio_queue = init_sdl2_audio(&sdl_context)?;

    let mut gameboy_state = GameBoyState::new();
    gameboy_state
        .load_cartridge(cartridge)
        .map_err(|e| e.to_string())?;

    let joypad = gameboy_state.get_joypad();
    let memory_bus = gameboy_state.get_memory_bus();

    let mut audio_data_history = ringbuf::HeapRb::<f32>::new(44100 * 30);

    audio_queue.resume();

    'mainloop: loop {
        for event in sdl_context.event_pump().unwrap().poll_iter() {
            match event {
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                }
                | Event::Quit { .. } => {
                    dbg!(gameboy_state.get_screen_hash());
                    println!("Saving audio");
                    File::create(Path::new("output.wav"))
                        .and_then(|mut wav| {
                            let data: Vec<f32> = audio_data_history.iter().copied().collect();
                            let wav_header =
                                wav::Header::new(wav::WAV_FORMAT_IEEE_FLOAT, 1, 44100, 32);
                            wav::write(wav_header, &wav::BitDepth::ThirtyTwoFloat(data), &mut wav)
                        })
                        .unwrap();
                    println!("Audio saved");

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
        // if buffer has less than four frames of audio, perform multiple ticks
        while audio_queue.size() < 1024 * 4 {
            gameboy_state.tick_for_frame();
            let data = gameboy_state.get_queued_audio();
            audio_data_history.push_iter_overwrite(data.iter().copied());
            audio_queue.queue_audio(&data)?;
        }

        // if buffer has too much data, skip a frame
        if audio_queue.size() < 1024 * 16 {
            gameboy_state.tick_for_frame();
            let data = gameboy_state.get_queued_audio();
            audio_data_history.push_iter_overwrite(data.iter().copied());
            audio_queue.queue_audio(&data)?;
        }

        //while audio_queue.size() > 4096 * 4 {}

        render_screen(gameboy_state.get_screen(), &mut canvas);

        // let duration = start.elapsed();
        // const FRAME_LENGTH: Duration = Duration::from_micros(1_000_000 / 60_000);
        // //const frame_length: Duration = Duration::from_millis(1000);
        // if duration > FRAME_LENGTH {
        //     warn!("Time elapsed this frame is: {:?} > 16ms", duration);
        // } else {
        //     //std::thread::sleep(FRAME_LENGTH - duration);
        // }
        // start = Instant::now();
    }
    Ok(())
}

fn init_sdl2_video(sdl_context: &sdl2::Sdl) -> Result<Canvas<Window>, String> {
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

    Ok(canvas)
}

fn init_sdl2_audio(sdl_context: &sdl2::Sdl) -> Result<AudioQueue<f32>, String> {
    let audio_subsystem = sdl_context.audio()?;

    let audio_spec = sdl2::audio::AudioSpecDesired {
        freq: Some(44100),
        channels: Some(2),
        samples: None,
    };

    let queue: AudioQueue<f32> = audio_subsystem.open_queue(None, &audio_spec)?;

    println!("{:?}", queue.spec());
    Ok(queue)
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
    for x in 0..160 {
        for y in 0..144 {
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
