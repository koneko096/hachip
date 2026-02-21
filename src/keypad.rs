pub struct Keypad {
    keys: [bool; 16], // Made private
}

impl Keypad {
    pub fn new() -> Keypad {
        Keypad { keys: [false; 16] }
    }

    // This method is still useful for the console_ui where multiple keys might be pressed simultaneously
    pub fn press(&mut self, indexes: Vec<u8>) {
        self.reset();
        for i in indexes {
            // Ensure index is within bounds (0-15)
            if (i as usize) < self.keys.len() {
                self.keys[i as usize] = true;
            }
        }
    }

    // New method for setting individual key state, useful for WASM
    pub fn set_key(&mut self, index: u8, is_pressed: bool) {
        if (index as usize) < self.keys.len() {
            self.keys[index as usize] = is_pressed;
        }
    }

    fn reset(&mut self) {
        for i in 0..16 {
            self.keys[i] = false
        }
    }

    pub fn is_key_down(&self, index: u8) -> bool {
        self.keys[index as usize]
    }

    // New: Check if any key is currently pressed
    pub fn any_key_down(&self) -> bool {
        self.keys.iter().any(|&k| k)
    }

    // New: Get the index of the first pressed key, if any
    pub fn get_first_key_down(&self) -> Option<u8> {
        self.keys.iter().position(|&k| k).map(|idx| idx as u8)
    }
}
