use crate::cartridge::Cartridge;
use crate::gameboy::GameBoyState;
use crate::joypad::JoypadInput;
use crate::ppu::{Ppu, CanvasPpu};
use crate::gameboy::Interrupt;
use log::warn;
use std::cell::RefCell;
use std::io::{self, Write};
use std::rc::Rc;
use std::thread;
use std::time::{Duration, Instant};
use strum::IntoEnumIterator;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;

pub const WIDTH: usize = 8 * (16 + 32);
pub const HEIGHT: usize = 8 * 32;

/// Manages GameBoy CPU exectution, adding breakpoint functionality.
/// Runs the GameBoy in a separate thread.
pub struct GameboyEmulator {
    counter: u64,
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

impl GameboyEmulator {
    pub fn new() -> Self {
        let emulator = Self { counter: 0 };

        emulator
    }

    /// Runs the gameboy emulator with a gui.
    fn run_gameboy_loop(
        cartridge: Box<dyn Cartridge>,
    ) -> Result<(), String> {
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;

        let window = video_subsystem
            .window("Gameboy Emulator", 800, 600)
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())?;

        let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
        canvas.set_logical_size((16 + 32) * 8, 32 * 8).map_err(|e| e.to_string())?;
        let creator = canvas.texture_creator();
        let tile_map = creator
            .create_texture_target(PixelFormatEnum::RGBA8888, 128, 192)
            .map_err(|e| e.to_string())?;
        let mut background_map = creator
            .create_texture_target(PixelFormatEnum::RGBA8888, 8 * 32, 8 * 32)
            .map_err(|e| e.to_string())?;
        let sprite_map = creator
            .create_texture_target(PixelFormatEnum::RGBA8888, 8 * 32, 8 * 32)
            .map_err(|e| e.to_string())?;

        let canvas = Rc::new(RefCell::new(canvas));

        let canvas_ppu = Rc::new(RefCell::new(CanvasPpu::new(&creator)));

        // Initialize gameboy and load cartridge
        let mut gameboy_state = GameBoyState::new(canvas_ppu.clone());
        gameboy_state.on_serial_port_data(|chr: char| {
            print!("{}", chr);
            io::stdout().flush().unwrap();
        });
        gameboy_state.load_cartridge(cartridge);

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

            let start = Instant::now();
            let mut cycle_total = 0;
            // The clock runs at 4,194,304 Hz, and every 4 clock cycles is 1 machine cycle.
            // Dividing by 4 and 60 should roughly give the number of machine cycles that
            // need to run per frame at 60fps.
            while cycle_total < 4_194_304 / 60 {
                cycle_total += Self::update(&mut gameboy_state);
            }
            {
                let mut canvas = canvas.borrow_mut();
                let mut canvas_ppu = canvas_ppu.borrow_mut();
                canvas_ppu
                    .render_tile_map(&mut canvas)
                    .expect("error rendering tile map");
                canvas
                    .with_texture_canvas(&mut background_map, |mut texture_canvas| {
                        canvas_ppu
                        .render_background_map(&mut texture_canvas)
                        .expect("error rendering background map");

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
                /*
                canvas.copy(
                    &sprite_map,
                    None,
                    Some(Rect::new(128, 0, 32 * 8, 32 * 8)),
                )?;
                */
            }
            /*
            gameboy_state
                .ppu
                .borrow_mut()
                .render_sprites()
                .expect("error rendering sprites");
            */
            let duration = start.elapsed();
            if duration > Duration::from_millis(1000 / 60) {
                warn!("Time elapsed this frame is: {:?} > 16ms", duration);
            }

            canvas.borrow_mut().present();
        }

        Ok(())

        /*
        event_loop.run(move |event, _, control_flow| {
            control_flow.set_poll();

            if input.update(&event) {
                // Handle joypad inputs
                let mut send_interrupt = false;
                for joypad_input in JoypadInput::iter() {
                    for virtual_key_code in map_joypad_to_keys(joypad_input).iter() {
                        if input.key_pressed(*virtual_key_code) {
                            let prev_state =
                                gameboy_state.joypad.borrow_mut().key_pressed(joypad_input);
                            // If previous state was not pressed, we send interrupt
                            send_interrupt |= !prev_state;
                        }
                        if input.key_released(*virtual_key_code) {
                            gameboy_state.joypad.borrow_mut().key_released(joypad_input);
                        }
                    }
                }

                if send_interrupt {
                    gameboy_state
                        .memory_bus
                        .borrow_mut()
                        .interrupt(Interrupt::Joypad)
                        .expect("error sending joypad interrupt");
                }

                let start = Instant::now();
                let mut cycle_total = 0;
                // The clock runs at 4,194,304 Hz, and every 4 clock cycles is 1 machine cycle.
                // Dividing by 4 and 60 should roughly give the number of machine cycles that
                // need to run per frame at 60fps.
                while cycle_total < 4_194_304 / 60 {
                    cycle_total += emulator.update(&mut gameboy_state);
                }
                gameboy_state
                    .ppu
                    .borrow_mut()
                    .render_tile_map()
                    .expect("error rending tile map");
                gameboy_state
                    .ppu
                    .borrow_mut()
                    .render_background_map()
                    .expect("error rendering background");
                gameboy_state
                    .ppu
                    .borrow_mut()
                    .render_sprites()
                    .expect("error rendering sprites");
                let duration = start.elapsed();
                if duration > Duration::from_millis(1000 / 60) {
                    warn!("Time elapsed this frame is: {:?} > 16ms", duration);
                }

                emulator.redraw_screen(&gameboy_state, &mut screen);
            }
        });
        */
    }

    fn run_gameboy_loop_no_gui(mut emulator: Self, mut gameboy_state: GameBoyState) {
        loop {
            let start = Instant::now();
            let mut cycle_total = 0;
            // The clock runs at 4,194,304 Hz, and every 4 clock cycles is 1 machine cycle.
            // Dividing by 4 and 60 should roughly give the number of machine cycles that
            // need to run per frame at 60fps.
            while cycle_total < 4_194_304 / 4 / 60 {
                cycle_total += Self::update(&mut gameboy_state);
            }
            let duration = start.elapsed();
            if duration > Duration::from_millis(1000 / 60) {
                warn!("Time elapsed this frame is: {:?} > 16ms", duration);
            }
        }
    }

    fn update(gameboy_state: &mut GameBoyState) -> u64 {
        gameboy_state.tick()
    }

    /*
    fn redraw_screen(&mut self, gameboy_state: &GameBoyState, screen: &mut PixelsScreen) {
        gameboy_state
            .ppu
            .borrow_mut()
            .step(gameboy_state)
            .expect("Error while stepping ppu");

        let ppu_screen = &gameboy_state.ppu.borrow().screen;
        for row in 0..HEIGHT {
            for col in 0..WIDTH {
                let index = row * WIDTH + col;
                let pixel_id = ppu_screen[index];
                let rgba: [u8; 4] = match pixel_id {
                    0 => [255, 255, 255, 255],
                    1 => [128, 128, 128, 255],
                    2 => [64, 64, 64, 255],
                    3 => [0, 0, 0, 255],
                    _ => unreachable!(),
                };
                screen
                    .set_pixel(row.try_into().unwrap(), col.try_into().unwrap(), &rgba)
                    .unwrap();
            }
        }
        screen.redraw();
    }
    */

    fn spawn_input_handler_thread(&mut self) {
        //let pause_state = self.pause_state.clone();

        thread::spawn(move || loop {
            let mut input = String::new();

            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");

            //pause_state.lock().unwrap().toggle_paused();
        });
    }

    pub fn run(cartridge: Box<dyn Cartridge>) {
        Self::run_gameboy_loop(cartridge);
    }

    pub fn test(mut gameboy_state: GameBoyState) -> Result<String, String> {
        let mut emulator = GameboyEmulator::new();

        gameboy_state.on_serial_port_data(|chr: char| {
            print!("{}", chr);
            io::stdout().flush();
        });

        Self::run_gameboy_loop_no_gui(emulator, gameboy_state);

        unimplemented!()
    }

    /*
    pub fn pause(&mut self) {
    self.pause_state.lock().unwrap().pause();
    }

    pub fn resume(&mut self) {
    self.pause_state.lock().unwrap().resume();
    }

    pub fn add_breakpoint(&mut self, breakpoint: u16) {
    self.breakpoints.lock().unwrap().insert(breakpoint);
    }

    pub fn remove_breakpoint(&mut self, breakpoint: u16) {
    self.breakpoints.lock().unwrap().remove(&breakpoint);
    }

    pub fn toggle_breakpoint(&mut self, breakpoint: u16) {
    let mut breakpoints = self.breakpoints.lock().unwrap();
    if breakpoints.contains(&breakpoint) {
    breakpoints.remove(&breakpoint);
    } else {
    breakpoints.insert(breakpoint);
    }
    }
    */
}
