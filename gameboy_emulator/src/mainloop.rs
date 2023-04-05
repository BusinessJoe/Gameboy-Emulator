use std::cell::RefCell;
use std::rc::Rc;

use crate::joypad::JoypadInput;
use crate::ppu::BasePpu;
use crate::Result;

pub trait Mainloop {
    fn get_ppu(&self) -> Rc<RefCell<BasePpu>>;
    fn mainloop<F: FnMut(JoypadInput, bool) -> (), G: FnMut() -> ()>(
        &mut self,
        on_joypad_input: F,
        do_work: G,
    );
}

pub trait MainloopBuilder: Send {
    type Target: Mainloop;
    fn init(self) -> Result<Self::Target>;
}
