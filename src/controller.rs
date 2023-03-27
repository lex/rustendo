pub struct Controller {
    buttons: [bool; 8],           // Button states (A, B, Select, Start, Up, Down, Left, Right)
    strobe: bool,                 // Strobe state for handling button presses
    index: usize,                 // Current button index for reading button states in a serial manner
}

impl Controller {
    pub fn new() -> Self {
        Self {
            buttons: [false; 8],
            strobe: false,
            index: 0,
        }
    }

    pub fn press_button(&mut self, button: usize) {
        self.buttons[button] = true;
    }

    pub fn release_button(&mut self, button: usize) {
        self.buttons[button] = false;
    }

    pub fn write(&mut self, value: u8) {
        self.strobe = value & 0x01 != 0;
        if self.strobe {
            self.index = 0;
        }
    }

    pub fn read(&mut self) -> u8 {
        let button_state = if self.index < self.buttons.len() {
            self.buttons[self.index] as u8
        } else {
            0
        };

        if self.strobe {
            self.index = 0;
        } else {
            self.index += 1;
        }

        button_state
    }
}