use std::{cell::RefCell, rc::Rc};

use sdl2::{event::Event, keyboard::Keycode, render::BlendMode, Sdl};
use strum::IntoEnumIterator;

use crate::sdl2::engine::CanvasEngine;
use crate::{
    joypad::JoypadInput,
    ppu::BasePpu,
    sdl2::texture::TextureBook,
    Result,
};

use crate::mainloop::{Mainloop, MainloopBuilder};

pub struct Sdl2Mainloop {
    sdl_context: Sdl,
    ppu: Rc<RefCell<BasePpu>>,
}

pub struct Sdl2MainloopBuilder;

impl Sdl2Mainloop {
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
}

impl Mainloop for Sdl2Mainloop {
    fn get_ppu(&self) -> Rc<RefCell<BasePpu>> {
        self.ppu.clone()
    }

    fn mainloop<F: FnMut(JoypadInput, bool) -> (), G: FnMut() -> ()>(
        &mut self,
        mut on_joypad_input: F,
        mut do_work: G,
    ) {
        'mainloop: loop {
            for event in self.sdl_context.event_pump().unwrap().poll_iter() {
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
                        for joypad_input in JoypadInput::iter() {
                            if Self::map_joypad_to_keys(joypad_input).contains(&keycode) {
                                on_joypad_input(joypad_input, true);
                            }
                        }
                    }
                    Event::KeyUp {
                        keycode: Some(keycode),
                        ..
                    } => {
                        for joypad_input in JoypadInput::iter() {
                            if Self::map_joypad_to_keys(joypad_input).contains(&keycode) {
                                on_joypad_input(joypad_input, false);
                            }
                        }
                    }
                    _ => {}
                }
            }

            do_work();
        }
    }
}

impl MainloopBuilder for Sdl2MainloopBuilder {
    type Target = Sdl2Mainloop;

    fn init(self) -> Result<Sdl2Mainloop> {
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
            .set_logical_size((20 + 1 + 16 + 1 + 32 + 1 + 32) * 8, (32) * 8)
            .map_err(|e| e.to_string())?;
        canvas.set_blend_mode(BlendMode::Blend);
        let texture_book = TextureBook::new(&canvas)?;

        let graphics_engine = Box::new(CanvasEngine::new(canvas, texture_book)?);
        let ppu = Rc::new(RefCell::new(BasePpu::new(graphics_engine)));

        Ok(Sdl2Mainloop { sdl_context, ppu })
    }
}
