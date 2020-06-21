pub struct Keypad {
    pub keys: [bool; 16],
}

impl Keypad {
    pub fn new() -> Keypad {
        Keypad { keys: [false; 16] }
    }

    pub fn press(&mut self, indexes: Vec<u8>) {
        self.reset();
        for i in indexes {
            self.keys[i as usize] = true;
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
}
