pub mod events;

use crate::cartridge::Cartridge;
use crate::error::Result;
use crate::gameboy::{GameBoyState, GameboyDebugInfo, Interrupt};
use crate::joypad::JoypadInput;
use crate::ppu::{BasePpu, CanvasEngine, NoGuiEngine};
use log::warn;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::BlendMode;
use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use strum::IntoEnumIterator;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use self::events::{EmulationControlEvent, EmulationEvent};
use crate::texture::TextureBook;

pub const WIDTH: usize = 8 * (16 + 32);
pub const HEIGHT: usize = 8 * 32;

/// Manages GameBoy CPU exectution, adding breakpoint functionality.
pub struct GameboyEmulator {
    // During debug mode, gameboy runs until the program counter
    // reaches this value if it exists. If it doesn't exist,
    // read in a value from stdin.
    target_pc: Option<u16>,
    debug: bool,
}

struct EmulatorDebugInfo {
    gameboy_info: GameboyDebugInfo,
    total_cycles: u128,
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

fn update_frame(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    ppu: &mut BasePpu,
    texture_book: &mut TextureBook,
) -> Result<()> {
    canvas.set_draw_color(Color::RGBA(50, 131, 168, 255));
    let (width, height) = canvas.logical_size();
    canvas.fill_rect(Rect::new(0, 0, width, height))?;

    ppu.render(canvas, texture_book)?;

    Ok(())
}

impl GameboyEmulator {
    pub fn new(debug: bool) -> Self {
        Self {
            target_pc: None,
            debug,
        }
    }

    pub fn gameboy_thread_no_gui(
        cartridge: Cartridge,
    ) -> Result<(
        JoinHandle<Result<()>>,
        mpsc::Sender<EmulationControlEvent>,
        mpsc::Receiver<EmulationEvent>,
    )> {
        let (event_sender, event_receiver) = mpsc::channel();
        let (control_event_sender, _control_event_receiver) =
            mpsc::channel::<EmulationControlEvent>();

        let join_handle = thread::spawn(move || -> Result<()> {
            let mut emulator = GameboyEmulator::new(false);

            let graphics_engine = Box::new(NoGuiEngine {});
            let ppu = Rc::new(RefCell::new(BasePpu::new(graphics_engine)));

            let mut gameboy_state = GameBoyState::new(ppu.clone(), event_sender);
            gameboy_state
                .load_cartridge(cartridge)
                .map_err(|e| e.to_string())?;
            let mut total_cycles: u128 = 0;
            loop {
                let elapsed_cycles = emulator.update(&mut gameboy_state, total_cycles);
                total_cycles += elapsed_cycles as u128;
            }
        });

        Ok((join_handle, control_event_sender, event_receiver))
    }

    pub fn gameboy_thread(
        cartridge: Cartridge,
    ) -> Result<(
        JoinHandle<Result<()>>,
        mpsc::Sender<EmulationControlEvent>,
        mpsc::Receiver<EmulationEvent>,
    )> {
        let (event_sender, event_receiver) = mpsc::channel();
        let (control_event_sender, _control_event_receiver) =
            mpsc::channel::<EmulationControlEvent>();

        let join_handle = thread::spawn(move || -> Result<()> {
            let mut emulator = GameboyEmulator::new(false);

            let sdl_context = sdl2::init()?;
            let video_subsystem = sdl_context.video()?;

            let window = video_subsystem
                .window("Gameboy Emulator", 1200, 900)
                .position_centered()
                .opengl()
                .build()
                .map_err(|e| e.to_string())?;

            let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
            canvas
                .set_logical_size((20 + 1 + 32) * 8, (32 + 1 + 32) * 8)
                .map_err(|e| e.to_string())?;
            canvas.set_blend_mode(BlendMode::Blend);
            let mut texture_book = TextureBook::new(&canvas)?;

            let canvas = Rc::new(RefCell::new(canvas));

            let graphics_engine = Box::new(CanvasEngine::new(&texture_book.texture_creator)?);
            let ppu = Rc::new(RefCell::new(BasePpu::new(graphics_engine)));

            // Initialize gameboy and load cartridge
            let mut gameboy_state = GameBoyState::new(ppu.clone(), event_sender);
            gameboy_state
                .load_cartridge(cartridge)
                .map_err(|e| e.to_string())?;

            // Keep track of total cycles and current cycles in current frame
            let mut total_cycles: u128 = 0;
            let mut frame_cycles = 0;

            // Start timing frames
            let mut start = Instant::now();

            'mainloop: loop {
                for event in sdl_context.event_pump()?.poll_iter() {
                    match event {
                        Event::KeyDown {
                            keycode: Some(Keycode::Escape),
                            ..
                        }
                        | Event::Quit { .. } => {
                            /*
                            let mut f = std::fs::File::create("events.log").expect("Unable to create file");
                            for event in gameboy_state.event_queue.iter() {
                                writeln!(f, "{:?}", event).expect("unable to write to event log file");
                            }
                             */
                            break 'mainloop;
                        }
                        Event::KeyDown {
                            keycode: Some(keycode),
                            ..
                        } => {
                            let mut send_interrupt = false;
                            for joypad_input in JoypadInput::iter() {
                                if map_joypad_to_keys(joypad_input).contains(&keycode) {
                                    let prev_state =
                                        gameboy_state.joypad.borrow_mut().key_pressed(joypad_input);
                                    // If previous state was not pressed, we send interrupt
                                    send_interrupt |= !prev_state;
                                }
                            }
                            if send_interrupt {
                                gameboy_state
                                    .memory_bus
                                    .borrow_mut()
                                    .interrupt(Interrupt::Joypad)
                                    .expect("error sending joypad interrupt");
                            }
                        }
                        Event::KeyUp {
                            keycode: Some(keycode),
                            ..
                        } => {
                            for joypad_input in JoypadInput::iter() {
                                if map_joypad_to_keys(joypad_input).contains(&keycode) {
                                    gameboy_state.joypad.borrow_mut().key_released(joypad_input);
                                }
                            }
                        }
                        _ => {}
                    }
                }

                for _ in 0..1000 {
                    let elapsed_cycles = emulator.update(&mut gameboy_state, total_cycles);
                    total_cycles += elapsed_cycles as u128;
                    frame_cycles += elapsed_cycles;
                }

                // The clock runs at 4,194,304 Hz, and every 4 clock cycles is 1 machine cycle.
                // Dividing by 4 and 60 should roughly give the number of machine cycles that
                // need to run per frame at 60fps.
                if frame_cycles >= 4_194_304 / 60 {
                    update_frame(
                        &mut canvas.borrow_mut(),
                        &mut ppu.borrow_mut(),
                        &mut texture_book,
                    )?;

                    frame_cycles -= 4_194_304 / 60;

                    let duration = start.elapsed();
                    if duration > Duration::from_millis(1000 / 60) {
                        warn!("Time elapsed this frame is: {:?} > 16ms", duration);
                    } else {
                        std::thread::sleep(Duration::from_millis(1000 / 60) - duration);
                    }
                    start = Instant::now();

                    canvas.borrow_mut().present();
                }
            }

            Ok(())
        });

        Ok((join_handle, control_event_sender, event_receiver))
    }

    fn update(&mut self, gameboy_state: &mut GameBoyState, total_cycles: u128) -> u64 {
        if self.debug {
            self.update_debug(gameboy_state, total_cycles)
        } else {
            self.update_regular(gameboy_state)
        }
    }

    fn get_target_pc_from_stdin() -> u16 {
        loop {
            // Read target_pc from stdin
            let mut buffer = String::new();
            print!("Enter target pc (in hex): ");
            std::io::stdout().flush().unwrap();
            std::io::stdin()
                .read_line(&mut buffer)
                .expect("reading from stdin failed");
            if let Ok(target_pc) = u16::from_str_radix(buffer.trim(), 16) {
                return target_pc;
            } else {
                eprintln!("Unable to convert to hex");
            }
        }
    }

    fn update_debug(&mut self, gameboy_state: &mut GameBoyState, total_cycles: u128) -> u64 {
        if let None = self.target_pc {
            self.target_pc = Some(Self::get_target_pc_from_stdin())
        }

        if gameboy_state.get_pc() != self.target_pc.unwrap() {
            let elapsed_cycles = gameboy_state.tick();
            let debug_info = EmulatorDebugInfo {
                gameboy_info: gameboy_state.debug_info(),
                // total_cycles is the total before the update is run, so we need to add
                // elapsed_cycles to get an accurate value
                total_cycles: total_cycles + elapsed_cycles as u128,
            };
            println!("{}", debug_info);
            elapsed_cycles
        } else {
            self.target_pc = Some(Self::get_target_pc_from_stdin());
            0
        }
    }

    fn update_regular(&self, gameboy_state: &mut GameBoyState) -> u64 {
        gameboy_state.tick()
    }

    /// Runs the gameboy emulator with a gui.
    pub fn run(cartridge: Cartridge) -> Result<()> {
        let (join_handle, _control_event_sender, event_receiver) = Self::gameboy_thread(cartridge)?;

        thread::spawn(move || {
            while let Ok(event) = event_receiver.recv() {
                match event {
                    EmulationEvent::SerialData(byte) => {
                        println!("serial data: {}/{}/0x{:x}", byte as char, byte, byte)
                    }
                    EmulationEvent::Trace(debug_info) => {
                        log::trace!("{:?}", debug_info);
                    }
                    event => println!("{:?}", event),
                }
            }
        });

        join_handle.join().expect("panic during execution")
    }
}

impl std::fmt::Display for EmulatorDebugInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} | cycles: {}", self.gameboy_info, self.total_cycles)
    }
}
