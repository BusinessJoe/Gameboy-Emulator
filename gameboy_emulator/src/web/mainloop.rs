use crate::mainloop::{MainloopBuilder, Mainloop};

pub struct WebMainloop {}

pub struct WebMainloopBuilder {}

impl MainloopBuilder for WebMainloopBuilder {
    type Target = WebMainloop;

    fn init(self) -> crate::Result<Self::Target> {
        todo!()
    }
}

impl Mainloop for WebMainloop {
    fn get_ppu(&self) -> std::rc::Rc<std::cell::RefCell<crate::ppu::BasePpu>> {
        todo!()
    }

    fn mainloop<F: FnMut(crate::joypad::JoypadInput, bool) -> (), G: FnMut() -> ()>(
        &mut self,
        on_joypad_input: F,
        do_work: G,
    ) {
        todo!()
    }
}