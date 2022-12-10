use crate::gameboy::GameBoyState;
use std::io;
use std::collections::HashSet;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

/// Manages GameBoy CPU exectution, adding breakpoint functionality.
/// Runs the GameBoy in a separate thread.
pub struct ExecutionManager {
    /// A JoinHandle on the current gameboy thread, if one exists.
    gameboy_join_handle: Option<JoinHandle<String>>,
    breakpoints: Arc<Mutex<HashSet<u16>>>,
    pause_state: Arc<Mutex<PauseState>>,
}

pub struct PauseState {
    paused: bool,
    resume_sender: Option<SyncSender<()>>,
}

impl ExecutionManager {
    pub fn new(gameboy_state: GameBoyState) -> Self {
        let mut manager = Self {
            gameboy_join_handle: None,
            breakpoints: Arc::new(Mutex::new(HashSet::new())),
            pause_state: Arc::new(Mutex::new(PauseState::new())),
        };

        manager.spawn_gameboy_thread(gameboy_state);
        manager
    }

    /// Spawns gameboy thread. This thread ticks the gameboy's cpu as long
    /// as the manager is not paused.
    fn spawn_gameboy_thread(&mut self, mut gameboy_state: GameBoyState) {
        let breakpoints = self.breakpoints.clone();
        let pause_state = self.pause_state.clone();
        let (resume_sender, resume_receiver) = sync_channel(0);

        pause_state.lock().unwrap().resume_sender = Some(resume_sender);
        self.gameboy_join_handle = Some(thread::spawn(move || {
            loop {
                println!("PC: {}", gameboy_state.cpu.pc);

                if breakpoints.lock().unwrap().contains(&gameboy_state.cpu.pc) {
                    pause_state.lock().unwrap().pause();
                }

                // If paused, wait until resume signal is sent over channel.
                if pause_state.lock().unwrap().is_paused() {
                    println!("Pausing...");
                    resume_receiver
                        .recv()
                        .expect("Could not receive from channel");
                    println!("Resuming...");
                }

                gameboy_state.tick();
            }
        }));
    }

    fn spawn_input_handler_thread(&mut self) {
        let pause_state = self.pause_state.clone();

        thread::spawn(move || {
            loop {
                let mut input = String::new();

                io::stdin().read_line(&mut input).expect("Failed to read line");

                pause_state.lock().unwrap().resume();
            }
        });
    }

    pub fn run(mut self) {
        self.pause_state.lock().unwrap().resume();

        self.spawn_input_handler_thread();

        let join_handle = self
            .gameboy_join_handle
            .expect("No currently running gameboy thread");
        join_handle.join().expect("Couldn't join on gameboy thread");
    }

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
}

impl PauseState {
    pub fn new() -> Self {
        Self {
            paused: true,
            resume_sender: None,
        }
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }

    pub fn resume(&mut self) {
        self.paused = false;
        self.resume_sender.as_mut().expect("Sender is null").try_send(());
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }
}
