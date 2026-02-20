// No sdl2 imports here anymore

pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

pub const FRAME_WIDTH: u32 = 640;
pub const FRAME_HEIGHT: u32 = 320;

pub const FACTOR: usize = 10; // FACTOR needs to be public for console_ui

// Removed PixelGrid trait and CanvasWindow struct from here

pub trait Display {
    fn cls(&mut self);
    fn draw(&mut self, x: usize, y: usize, sprite: &[u8]) -> bool;
    fn set_pixel(&mut self, x: usize, y: usize, val: u8);
    fn get_pixel(&self, x: usize, y: usize) -> bool; // get_pixel can be immutable
}

pub struct Ppu {
    memory: [u8; WIDTH * HEIGHT], // Use WIDTH * HEIGHT for clarity
    display_updated: bool, // New flag to signal updates
}

impl Ppu {
    pub fn new() -> Ppu { // No longer takes PixelGrid
        Ppu {
            memory: [0; WIDTH * HEIGHT],
            display_updated: false,
        }
    }

    pub fn get_display_memory(&self) -> &[u8] {
        &self.memory
    }

    pub fn take_display_update_flag(&mut self) -> bool {
        let updated = self.display_updated;
        self.display_updated = false; // Reset the flag after being read
        updated
    }
}

impl Display for Ppu {
    fn cls(&mut self) {
        self.memory.iter_mut().for_each(|x| *x = 0); // Clear memory efficiently
        self.display_updated = true;
    }

    fn draw(&mut self, x: usize, y: usize, sprite: &[u8]) -> bool {
        let rows = sprite.len();
        let mut collision = false;
        for (j, _) in sprite.iter().enumerate().take(rows) {
            let row = &sprite[j];
            for i in 0..8 {
                let new_value = row >> (7 - i) & 0x01;
                if new_value == 1 {
                    let xi = (x + i) % WIDTH;
                    let yj = (y + j) % HEIGHT;
                    let old_value = self.get_pixel(xi, yj); // Use immutable get_pixel
                    if old_value {
                        collision = true;
                    }
                    let display_value = ((new_value == 1) ^ old_value) as u8;
                    self.set_pixel(xi, yj, display_value); // set_pixel will mark display_updated
                }
            }
        }
        // display_updated is already set by set_pixel if any pixel changed
        return collision;
    }

    fn set_pixel(&mut self, x: usize, y: usize, val: u8) {
        let index = x + y * WIDTH;
        if index < self.memory.len() { // Boundary check
            if self.memory[index] != val { // Only mark as updated if value actually changes
                self.memory[index] = val;
                self.display_updated = true;
            }
        }
    }

    fn get_pixel(&self, x: usize, y: usize) -> bool {
        let index = x + y * WIDTH;
        if index < self.memory.len() { // Boundary check
            self.memory[index] == 1
        } else {
            false // Or handle error
        }
    }
}

pub static FONT_SET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

#[cfg(test)]
mod tests {
    use super::{Ppu, Display, WIDTH, HEIGHT};

    // No more PixelGridMock needed in ppu.rs tests

    #[test]
    fn set_pixel() {
        let mut ppu = Ppu::new(); // No arg
        ppu.set_pixel(1, 1, 1);
        assert_eq!(true, ppu.get_pixel(1, 1));
        assert_eq!(ppu.display_updated, true);
    }

    #[test]
    fn cls() {
        let mut ppu = Ppu::new(); // No arg
        ppu.set_pixel(1, 1, 1);
        ppu.display_updated = false; // Reset for testing cls
        ppu.cls();
        assert_eq!(false, ppu.get_pixel(1, 1));
        assert_eq!(ppu.display_updated, true);
    }

    #[test]
    fn draw() {
        let mut ppu = Ppu::new(); // No arg
        let sprite: [u8; 2] = [0b00110011, 0b11001010];
        ppu.draw(0, 0, &sprite);

        assert_eq!(false, ppu.get_pixel(0, 0));
        assert_eq!(false, ppu.get_pixel(1, 0));
        assert_eq!(true, ppu.get_pixel(2, 0));
        assert_eq!(true, ppu.get_pixel(3, 0));
        assert_eq!(false, ppu.get_pixel(4, 0));
        assert_eq!(false, ppu.get_pixel(5, 0));
        assert_eq!(true, ppu.get_pixel(6, 0));
        assert_eq!(true, ppu.get_pixel(7, 0));

        assert_eq!(true, ppu.get_pixel(0, 1));
        assert_eq!(true, ppu.get_pixel(1, 1));
        assert_eq!(false, ppu.get_pixel(2, 1));
        assert_eq!(false, ppu.get_pixel(3, 1));
        assert_eq!(true, ppu.get_pixel(4, 1));
        assert_eq!(false, ppu.get_pixel(5, 1));
        assert_eq!(true, ppu.get_pixel(6, 1));
        assert_eq!(false, ppu.get_pixel(7, 1));
    }

    #[test]
    fn draw_detects_collisions() {
        let mut ppu = Ppu::new();

        // Draw a sprite, no collision initially
        let sprite1: [u8; 1] = [0b11000000];
        let collision1 = ppu.draw(0, 0, &sprite1);
        assert_eq!(false, collision1);
        assert_eq!(ppu.get_pixel(0, 0), true);
        assert_eq!(ppu.get_pixel(1, 0), true);

        // Draw another sprite that overlaps, causing a collision
        let sprite2: [u8; 1] = [0b01100000]; // Overlaps at (1,0)
        let collision2 = ppu.draw(0, 0, &sprite2);
        assert_eq!(true, collision2); // Collision should be true
        assert_eq!(ppu.get_pixel(0, 0), true); // (0,0) remains on (sprite1)
        assert_eq!(ppu.get_pixel(1, 0), false); // (1,0) turned off (XOR with sprite2)
        assert_eq!(ppu.get_pixel(2, 0), true); // (2,0) turned on (sprite2)
    }
}
