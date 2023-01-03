use crate::gameboy::GameBoyState;
use crate::screen::{PixelsScreen, Screen};
use crate::component::Steppable;
use crossterm::{terminal, ExecutableCommand};
use std::io::{self, stdout};
use std::thread::{self};
use log::info;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
};

const WIDTH: usize = 8 * 32;
const HEIGHT: usize = 8 * 32;

/// Manages GameBoy CPU exectution, adding breakpoint functionality.
/// Runs the GameBoy in a separate thread.
pub struct GameboyEmulator {
    counter: u64,
}

impl GameboyEmulator {
    pub fn new() -> Self {
        let emulator = Self {
            counter: 0
        };

        emulator
    }

    /// Spawns gameboy thread. This thread ticks the gameboy's cpu as long
    /// as the manager is not paused.
    fn run_gameboy_loop(mut emulator: Self, mut gameboy_state: GameBoyState, test_mode: bool) {
        let mut current_output: String = String::from("");

        let event_loop = EventLoop::new();
        let mut screen = PixelsScreen::new(WIDTH.try_into().unwrap(), HEIGHT.try_into().unwrap(), 512*2, 512, &event_loop);

        event_loop.run(move |event, _, control_flow| {
            control_flow.set_poll();

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    println!("Closing");
                    control_flow.set_exit();
                }
                Event::MainEventsCleared => {
                    emulator.update(
                        &mut gameboy_state,
                        &mut current_output,
                        test_mode,
                        &mut screen,
                    );
                }
                Event::RedrawRequested(_) => {}
                _ => (),
            }
        });
    }

    fn update(
        &mut self,
        gameboy_state: &mut GameBoyState,
        current_output: &mut String,
        test_mode: bool,
        mut screen: &mut PixelsScreen,
    ) -> Option<Result<String, String>> {
        gameboy_state.tick();

        if self.counter % 1000 == 0 {
            //println!("Redrawing");
            self.redraw_screen(&gameboy_state, &mut screen);
        }
        self.counter += 1;

        None
    }

    fn redraw_screen(&mut self, gameboy_state: &GameBoyState, screen: &mut PixelsScreen) {
        gameboy_state.ppu.lock().unwrap().step(gameboy_state).expect("Error while stepping ppu");

        let ppu_screen = &gameboy_state.ppu.lock().unwrap().screen;
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
                screen.set_pixel(row.try_into().unwrap(), col.try_into().unwrap(), &rgba).unwrap();
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
