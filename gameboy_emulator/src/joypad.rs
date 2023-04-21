use strum_macros::EnumIter;

#[derive(Debug, Clone, Copy, EnumIter)]
pub enum JoypadInput {
    A,
    B,
    Start,
    Select,
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug)]
pub struct Joypad {
    /// Only bits 5 and 6 are used
    state_byte: u8,
    action_nibble: u8,
    direction_nibble: u8,
}

impl Joypad {
    pub fn new() -> Joypad {
        Joypad {
            state_byte: 0x0f,
            action_nibble: 0xf,
            direction_nibble: 0xf,
        }
    }

    /// Notify the joypad that an input was pressed. Returns true iff the input
    /// was previously pressed.
    pub fn key_pressed(&mut self, input: JoypadInput) -> bool {
        println!("pressed {:?}", input);
        let nibble = {
            use JoypadInput::*;
            match input {
                A | B | Select | Start => &mut self.action_nibble,
                Right | Left | Up | Down => &mut self.direction_nibble,
            }
        };

        if *nibble & (1 << Joypad::get_input_bit(input)) != 0 {
            // Set bit from high to low to indicate input pressed
            *nibble &= !(1 << Joypad::get_input_bit(input));

            false
        } else {
            true
        }
    }

    /// Notify the joypad that an input was released. Returns true iff the input
    /// was previously pressed.
    pub fn key_released(&mut self, input: JoypadInput) -> bool {
        println!("released {:?}", input);
        let nibble = {
            use JoypadInput::*;
            match input {
                A | B | Select | Start => &mut self.action_nibble,
                Right | Left | Up | Down => &mut self.direction_nibble,
            }
        };

        if *nibble & (1 << Joypad::get_input_bit(input)) == 0 {
            // Set bit from low to high to indicate input not pressed
            *nibble |= 1 << Joypad::get_input_bit(input);

            true
        } else {
            false
        }
    }

    fn get_input_bit(input: JoypadInput) -> u8 {
        use JoypadInput::*;
        match input {
            A | Right => 0,
            B | Left => 1,
            Select | Up => 2,
            Start | Down => 3,
        }
    }

    /// Use keyboard input to get the byte at 0xff00.
    fn get_state(&self) -> u8 {
        let mut input_nibble = 0u8;

        if self.select_action() {
            input_nibble |= self.action_nibble;
        }
        if self.select_direction() {
            input_nibble |= self.direction_nibble;
        }

        // Mask out everything but the select bits and add the inputs
        self.state_byte & 0b11_0000 | input_nibble
    }

    fn select_action(&self) -> bool {
        self.state_byte & (1 << 5) == 0
    }

    fn select_direction(&self) -> bool {
        self.state_byte & (1 << 4) == 0
    }

    pub fn read(&self) -> u8 {
        self.get_state()
    }

    pub fn write(&mut self, data: u8) {
        self.state_byte = data;
    }
}
