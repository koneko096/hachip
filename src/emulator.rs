use wasm_bindgen::prelude::*;
use crate::cpu::Cpu;

#[wasm_bindgen]
pub struct Emulator {
    cpu: Cpu,
}

#[wasm_bindgen]
impl Emulator {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Emulator {
        Emulator {
            cpu: Cpu::new(),
        }
    }

    pub fn load_rom(&mut self, rom_data: &[u8]) {
        self.cpu.load(rom_data.to_vec());
        self.cpu.reset();
    }

    pub fn tick(&mut self) {
        self.cpu.execute_cycle();
    }

    pub fn tick_timers(&mut self) {
        self.cpu.tick_timers();
    }

    pub fn reset(&mut self) { // Added reset method
        self.cpu.reset();
    }

    pub fn get_display_ptr(&self) -> *const u8 {
        self.cpu.ppu.get_display_memory().as_ptr()
    }

    pub fn get_display_width(&self) -> u32 {
        self.cpu.ppu.get_width() as u32
    }

    pub fn get_display_height(&self) -> u32 {
        self.cpu.ppu.get_height() as u32
    }

    pub fn set_key_state(&mut self, key_index: u8, is_pressed: bool) {
        self.cpu.keypad.set_key(key_index, is_pressed);
    }

    pub fn display_updated(&mut self) -> bool {
        self.cpu.ppu.take_display_update_flag()
    }
}
