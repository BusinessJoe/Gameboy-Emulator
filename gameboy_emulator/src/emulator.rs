pub mod events;

use crate::cartridge::Cartridge;
use crate::error::Result;
use crate::gameboy::{GameBoyState, GameboyDebugInfo, Interrupt};
use crate::mainloop::{sdl2::Sdl2MainloopBuilder, Mainloop, MainloopBuilder};
use crate::ppu::{BasePpu, NoGuiEngine};
use log::warn;
use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use self::events::{EmulationControlEvent, EmulationEvent};

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

    pub fn gameboy_thread<M: MainloopBuilder + 'static>(
        cartridge: Cartridge,
        mainloop_builder: M,
    ) -> Result<(
        JoinHandle<Result<()>>,
        mpsc::Sender<EmulationControlEvent>,
        mpsc::Receiver<EmulationEvent>,
    )> {
        let (event_sender, event_receiver) = mpsc::channel();
        let (control_event_sender, _control_event_receiver) =
            mpsc::channel::<EmulationControlEvent>();

        let join_handle = thread::spawn(move || -> Result<()> {
            let mut mainloop = mainloop_builder.init()?;
            let ppu = mainloop.get_ppu();

            let mut emulator = GameboyEmulator::new(false);

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

            let joypad = gameboy_state.joypad.clone();
            let memory_bus = gameboy_state.memory_bus.clone();

            mainloop.mainloop(
                move |joypad_input, pressed| {
                    if pressed {
                        let prev_state = joypad.borrow_mut().key_pressed(joypad_input);
                        // If previous state was not pressed, we send interrupt
                        if !prev_state {
                            memory_bus
                                .borrow_mut()
                                .interrupt(Interrupt::Joypad)
                                .expect("error sending joypad interrupt");
                        }
                    } else {
                        joypad.borrow_mut().key_released(joypad_input);
                    }
                },
                move || {
                    for _ in 0..1000 {
                        let elapsed_cycles = emulator.update(&mut gameboy_state, total_cycles);
                        total_cycles += elapsed_cycles as u128;
                        frame_cycles += elapsed_cycles;
                    }

                    // The clock runs at 4,194,304 Hz, and every 4 clock cycles is 1 machine cycle.
                    // Dividing by 4 and 60 should roughly give the number of machine cycles that
                    // need to run per frame at 60fps.
                    if frame_cycles >= 4_194_304 / 60 {
                        ppu.borrow_mut().render().unwrap();

                        frame_cycles -= 4_194_304 / 60;

                        let duration = start.elapsed();
                        if duration > Duration::from_millis(1000 / 60) {
                            warn!("Time elapsed this frame is: {:?} > 16ms", duration);
                        } else {
                            std::thread::sleep(Duration::from_millis(1000 / 60) - duration);
                        }
                        start = Instant::now();
                    }
                },
            );

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
        let mainloop_builder = Sdl2MainloopBuilder {};

        let (join_handle, _control_event_sender, event_receiver) =
            Self::gameboy_thread(cartridge, mainloop_builder)?;

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
