use crate::gameboy::GameBoyState;
use std::collections::HashSet;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

/// Manages GameBoy CPU exectution, adding breakpoint functionality.
/// Runs the GameBoy in a separate thread.
pub struct ExecutionManager {
    /// A JoinHandle on the current gameboy thread, if one exists.
    gameboy_join_handle: Option<JoinHandle<String>>,
    breakpoints: Arc<Mutex<HashSet<u16>>>,
    paused: Arc<Mutex<bool>>,
    resume_sender: Sender<()>,
    resume_receiver: Receiver<()>,
}

impl ExecutionManager {
    pub fn new(gameboy_state: GameBoyState) -> Self {
        let (resume_sender, resume_receiver) = channel();

        let mut manager = Self {
            gameboy_join_handle: None,
            breakpoints: Arc::new(Mutex::new(HashSet::new())),
            paused: Arc::new(Mutex::new(true)),
            resume_sender,
            resume_receiver,
        };

        manager.spawn_gameboy_thread(gameboy_state);
        manager
    }

    /// Spawns gameboy thread. This thread ticks the gameboy's cpu as long
    /// as the manager is not paused.
    fn spawn_gameboy_thread(&mut self, mut gameboy_state: GameBoyState) {
        let breakpoints = self.breakpoints.clone();
        self.gameboy_join_handle = Some(thread::spawn(move || {
            loop {
                //if *self.paused.lock().unwrap() {
                //    // Wait until resume signal is sent
                //    self.resume_receiver.recv().expect("Could not receive from channel");
                //}
                gameboy_state.tick();
                println!("PC: {}", gameboy_state.cpu.pc);

                //if breakpoints.lock().unwrap().contains(&gameboy_state.cpu.pc) {
                //    self.pause();
                //}
            }
        }));
    }

    pub fn join(self) {
        let join_handle = self
            .gameboy_join_handle
            .expect("No currently running gameboy thread");
        join_handle.join().expect("Couldn't join on gameboy thread");
    }

    pub fn pause(&mut self) {
        let mut paused = self.paused.lock().unwrap();
        *paused = true;
    }

    pub fn resume(&mut self) {
        let mut paused = self.paused.lock().unwrap();
        *paused = false;
        self.resume_sender
            .send(())
            .expect("Count not send on channel");
    }

    pub fn add_breakpoint(&mut self, breakpoint: u16) {
        self.breakpoints.lock().unwrap().insert(breakpoint);
    }

    pub fn remove_breakpoint(&mut self, breakpoint: u16) {
        self.breakpoints.lock().unwrap().remove(&breakpoint);
    }
}
