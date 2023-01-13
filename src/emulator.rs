use crate::component::Steppable;
use crate::gameboy::GameBoyState;
use crate::joypad::JoypadInput;
use crate::screen::{PixelsScreen, Screen};
use log::warn;
use winit::event::VirtualKeyCode;
use winit::event_loop;
use winit_input_helper::WinitInputHelper;
use std::io::{self, Write};
use std::thread;
use std::time::{Duration, Instant};
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
};
use strum::IntoEnumIterator;

pub const WIDTH: usize = 8 * (16 + 32);
pub const HEIGHT: usize = 8 * 32;

/// Manages GameBoy CPU exectution, adding breakpoint functionality.
/// Runs the GameBoy in a separate thread.
pub struct GameboyEmulator {
    counter: u64,
}

// Maps keyboard keys to corresponding joypad inputs.
fn map_joypad_to_keys(input: JoypadInput) -> Vec<VirtualKeyCode> {
    match input {
        JoypadInput::A => vec![VirtualKeyCode::A],
        JoypadInput::B => vec![VirtualKeyCode::B],
        JoypadInput::Start => vec![VirtualKeyCode::Space],
        JoypadInput::Select => vec![VirtualKeyCode::Return],
        JoypadInput::Up => vec![VirtualKeyCode::Up],
        JoypadInput::Down => vec![VirtualKeyCode::Down],
        JoypadInput::Left => vec![VirtualKeyCode::Left],
        JoypadInput::Right => vec![VirtualKeyCode::Right],
    }
}

impl GameboyEmulator {
    pub fn new() -> Self {
        let emulator = Self { counter: 0 };

        emulator
    }

    /// Spawns gameboy thread. This thread ticks the gameboy's cpu as long
    /// as the manager is not paused.
    fn run_gameboy_loop(mut emulator: Self, mut gameboy_state: GameBoyState, test_mode: bool) {
        let mut current_output: String = String::from("");

        let mut input = WinitInputHelper::new();

        let event_loop = EventLoop::new();
        let mut screen = PixelsScreen::new(
            WIDTH.try_into().unwrap(),
            HEIGHT.try_into().unwrap(),
            512 * 2,
            512,
            &event_loop,
        );

        event_loop.run(move |event, _, control_flow| {
            control_flow.set_poll();

            if input.update(&event) {
                // Handle joypad inputs
                for joypad_input in JoypadInput::iter() {
                    for virtual_key_code in map_joypad_to_keys(joypad_input).iter() {
                        if input.key_pressed(*virtual_key_code) {
                            gameboy_state.joypad.borrow_mut().key_pressed(joypad_input);
                        }
                        if input.key_released(*virtual_key_code) {
                            gameboy_state.joypad.borrow_mut().key_released(joypad_input);
                        }
                    }
                }

                let start = Instant::now();
                let mut cycle_total = 0;
                // The clock runs at 4,194,304 Hz, and every 4 clock cycles is 1 machine cycle.
                // Dividing by 4 and 60 should roughly give the number of machine cycles that
                // need to run per frame at 60fps.
                while cycle_total < 4_194_304 / 4 / 60 {
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
                let duration = start.elapsed();
                if duration > Duration::from_millis(1000 / 60) {
                    warn!("Time elapsed this frame is: {:?} > 16ms", duration);
                }

                emulator.redraw_screen(&gameboy_state, &mut screen);
            }
        });
    }

    fn update(&mut self, gameboy_state: &mut GameBoyState) -> u64 {
        gameboy_state.tick()
    }

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

    pub fn run(mut gameboy_state: GameBoyState) {
        let mut emulator = GameboyEmulator::new();

        gameboy_state.on_serial_port_data(|chr: char| {
            print!("{}", chr);
            io::stdout().flush();
        });

        // emulator.spawn_input_handler_thread();

        Self::run_gameboy_loop(emulator, gameboy_state, false);
    }

    pub fn test(&mut self, gameboy_state: GameBoyState) -> Result<String, String> {
        unimplemented!();
        /*
           let gameboy_join_handle = Self::spawn_gameboy_thread(
           self.breakpoints.clone(),
           self.pause_state.clone(),
           self.global_time.clone(),
           gameboy_state,
           true,
           );

           self.pause_state.lock().unwrap().resume();

           gameboy_join_handle
           .join()
           .expect("Couldn't join on gameboy thread")
           */
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
