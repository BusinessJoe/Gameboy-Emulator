use crate::cartridge::Cartridge;
use crate::gameboy::{GameBoyState, GameboyDebugInfo};
use crate::joypad::JoypadInput;
use crate::ppu::CanvasPpu;
use crate::gameboy::Interrupt;
use log::warn;
use sdl2::render::BlendMode;
use std::cell::RefCell;
use std::io::{self, Write};
use std::rc::Rc;
use std::time::{Duration, Instant};
use strum::IntoEnumIterator;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;

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
    canvas_ppu: &mut CanvasPpu,
    background_map: &mut sdl2::render::Texture,
    lcd_display: &mut sdl2::render::Texture,
    sprite_map: &mut sdl2::render::Texture,
) -> Result<(), String> {
    canvas_ppu
        .render_tile_map(canvas)
        .expect("error rendering tile map");

    canvas
        .with_texture_canvas(background_map, |mut texture_canvas| {
            canvas_ppu
                .render_background_map(&mut texture_canvas)
                .expect("error rendering background map");

        })
    .map_err(|e| e.to_string())?;

    canvas
        .with_texture_canvas(lcd_display, |texture_canvas| {
            texture_canvas.set_draw_color(sdl2::pixels::Color::RGBA(255, 0, 0, 255));
            texture_canvas.clear();
        })
    .map_err(|e| e.to_string())?;

    canvas
        .with_texture_canvas(sprite_map, |mut texture_canvas| {
            texture_canvas.set_draw_color(sdl2::pixels::Color::RGBA(0, 0, 0, 0));
            texture_canvas.clear();
            // Render sprites over background map for now
            canvas_ppu
                .render_sprites(&mut texture_canvas)
                .expect("error rendering sprite");
        })
    .map_err(|e| e.to_string())?;

    canvas.copy(
        &background_map,
        None,
        Some(Rect::new(128, 0, 32 * 8, 32 * 8)),
        )?;
    canvas.copy(
        &lcd_display,
        None,
        Some(Rect::new(128 + 32 * 8, 0, 160, 144)),
        )?;
    canvas.copy(
        &sprite_map,
        None,
        Some(Rect::new(128 + 32 * 8, 0, 160, 144)),
        )?;

    Ok(())
}

impl GameboyEmulator {
    pub fn new(debug: bool) -> Self {
        Self { 
            target_pc: None,
            debug,
        }
    }

    fn run_gameboy_loop(
        cartridge: Box<dyn Cartridge>,
        debug: bool,
        ) -> Result<(), String> {
        let mut emulator = GameboyEmulator::new(debug);

        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;

        let window = video_subsystem
            .window("Gameboy Emulator", 1200, 900)
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())?;

        let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
        canvas.set_logical_size(128 + 32 * 8 + 160, 32 * 8).map_err(|e| e.to_string())?;
        canvas.set_blend_mode(BlendMode::Blend);
        let creator = canvas.texture_creator();
        let mut background_map = creator
            .create_texture_target(PixelFormatEnum::RGBA8888, 8 * 32, 8 * 32)
            .map_err(|e| e.to_string())?;
        let mut sprite_map = creator
            .create_texture_target(PixelFormatEnum::RGBA8888, 160, 144)
            .map_err(|e| e.to_string())?;
        let mut lcd_display = creator
            .create_texture_target(PixelFormatEnum::RGBA8888, 160, 144)
            .map_err(|e| e.to_string())?;
        sprite_map.set_blend_mode(BlendMode::Blend);

        let canvas = Rc::new(RefCell::new(canvas));

        let canvas_ppu = Rc::new(RefCell::new(CanvasPpu::new(&creator)));

        // Initialize gameboy and load cartridge
        let mut gameboy_state = GameBoyState::new(canvas_ppu.clone());
        gameboy_state.on_serial_port_data(|chr: char| {
            println!("serial data: {}/{}/0x{:x}", chr, chr as u8, chr as u8);
            io::stdout().flush().unwrap();
        });
        gameboy_state.load_cartridge(cartridge).map_err(|e| e.to_string())?;

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
                    | Event::Quit { .. } => break 'mainloop,
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

                    },
                    Event::KeyUp {
                        keycode: Some(keycode),
                        ..
                    } => {
                        for joypad_input in JoypadInput::iter() {
                            if map_joypad_to_keys(joypad_input).contains(&keycode) {
                                gameboy_state.joypad.borrow_mut().key_released(joypad_input);
                            }
                        }
                    },
                        _ => {}
                }
            }

            for _ in 0 .. 1000 {
                let elapsed_cycles =  emulator.update(&mut gameboy_state, total_cycles);
                total_cycles += elapsed_cycles as u128;
                frame_cycles += elapsed_cycles;
            }

            // The clock runs at 4,194,304 Hz, and every 4 clock cycles is 1 machine cycle.
            // Dividing by 4 and 60 should roughly give the number of machine cycles that
            // need to run per frame at 60fps.
            if frame_cycles >= 4_194_304 / 4 / 60 {
                update_frame(
                    &mut canvas.borrow_mut(), 
                    &mut canvas_ppu.borrow_mut(),
                    &mut background_map,
                    &mut lcd_display,
                    &mut sprite_map,
                )?;

                frame_cycles -= 4_194_304 / 4 / 60;

                let duration = start.elapsed();
                if duration > Duration::from_millis(1000 / 60) {
                    warn!("Time elapsed this frame is: {:?} > 16ms", duration);
                } else {
                    //std::thread::sleep(Duration::from_millis(1000 / 60) - duration);
                }
                start = Instant::now();

                canvas.borrow_mut().present();
            }
        }

        Ok(())
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
            std::io::stdin().read_line(&mut buffer).expect("reading from stdin failed");
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
    pub fn run(cartridge: Box<dyn Cartridge>, debug: bool) -> Result<(), ()> {
        Self::run_gameboy_loop(cartridge, debug).map_err(|_| ())
    }

    /*
       pub fn test(mut gameboy_state: GameBoyState) -> Result<String, String> {
       let mut emulator = GameboyEmulator::new();

       gameboy_state.on_serial_port_data(|chr: char| {
       print!("{}", chr);
       io::stdout().flush();
       });

       Self::run_gameboy_loop_no_gui(emulator, gameboy_state);

       unimplemented!()
       }
       */
}

impl std::fmt::Display for EmulatorDebugInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} | cycles: {}", self.gameboy_info, self.total_cycles)
    }
}
