// No sdl2 imports here anymore

pub const LOW_RES_WIDTH: usize = 64;
pub const LOW_RES_HEIGHT: usize = 32;
pub const HIGH_RES_WIDTH: usize = 128;
pub const HIGH_RES_HEIGHT: usize = 64;

pub const FRAME_WIDTH: u32 = 640;
pub const FRAME_HEIGHT: u32 = 320;

pub trait Display {
    fn cls(&mut self);
    fn draw(&mut self, x: usize, y: usize, sprite: &[u8]) -> bool;
    fn set_pixel(&mut self, x: usize, y: usize, val: u8);
    fn get_pixel(&self, x: usize, y: usize) -> bool;
}

pub struct Ppu {
    memory: [u8; HIGH_RES_WIDTH * HIGH_RES_HEIGHT],
    display_updated: bool,
    width: usize,
    height: usize,
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            memory: [0; HIGH_RES_WIDTH * HIGH_RES_HEIGHT],
            display_updated: false,
            width: LOW_RES_WIDTH,
            height: LOW_RES_HEIGHT,
        }
    }

    pub fn set_resolution(&mut self, high_res: bool) {
        if high_res {
            self.width = HIGH_RES_WIDTH;
            self.height = HIGH_RES_HEIGHT;
        } else {
            self.width = LOW_RES_WIDTH;
            self.height = LOW_RES_HEIGHT;
        }
        self.cls(); // Resolution change usually implies clearing the screen
    }

    pub fn get_width(&self) -> usize {
        self.width
    }

    pub fn get_height(&self) -> usize {
        self.height
    }

    pub fn get_display_memory(&self) -> &[u8] {
        &self.memory[0..self.width * self.height]
    }

    pub fn take_display_update_flag(&mut self) -> bool {
        let updated = self.display_updated;
        self.display_updated = false;
        updated
    }

    pub fn scroll_down(&mut self, rows: usize) {
        if rows == 0 || rows >= self.height {
            if rows != 0 {
                self.cls();
            }
            return;
        }
        let size = self.width * self.height;
        let mut next = vec![0u8; size];
        for y in 0..self.height {
            let src_y = y.checked_sub(rows);
            if let Some(sy) = src_y {
                let dst_row = y * self.width;
                let src_row = sy * self.width;
                next[dst_row..dst_row + self.width]
                    .copy_from_slice(&self.memory[src_row..src_row + self.width]);
            }
        }
        self.memory[0..size].copy_from_slice(&next);
        self.display_updated = true;
    }

    pub fn scroll_right(&mut self, pixels: usize) {
        if pixels == 0 || pixels >= self.width {
            if pixels != 0 {
                self.cls();
            }
            return;
        }
        let size = self.width * self.height;
        let mut next = vec![0u8; size];
        for y in 0..self.height {
            let row = y * self.width;
            for x in 0..self.width {
                if x >= pixels {
                    next[row + x] = self.memory[row + x - pixels];
                }
            }
        }
        self.memory[0..size].copy_from_slice(&next);
        self.display_updated = true;
    }

    pub fn scroll_left(&mut self, pixels: usize) {
        if pixels == 0 || pixels >= self.width {
            if pixels != 0 {
                self.cls();
            }
            return;
        }
        let size = self.width * self.height;
        let mut next = vec![0u8; size];
        for y in 0..self.height {
            let row = y * self.width;
            for x in 0..self.width {
                let src_x = x + pixels;
                if src_x < self.width {
                    next[row + x] = self.memory[row + src_x];
                }
            }
        }
        self.memory[0..size].copy_from_slice(&next);
        self.display_updated = true;
    }
}

impl Display for Ppu {
    fn cls(&mut self) {
        self.memory[0..self.width * self.height].iter_mut().for_each(|x| *x = 0);
        self.display_updated = true;
    }

    fn draw(&mut self, x: usize, y: usize, sprite: &[u8]) -> bool {
        let rows = sprite.len();
        let mut collision = false;
        
        // CHIP-8 classic behavior: wrap sprites around screen edges
        let start_x = x % self.width;
        let start_y = y % self.height;

        for j in 0..rows {
            let row = sprite[j];
            for i in 0..8 {
                let new_value = (row >> (7 - i)) & 0x01;
                if new_value == 1 {
                    let xi = (start_x + i) % self.width;
                    let yj = (start_y + j) % self.height;

                    let old_value = self.get_pixel(xi, yj);
                    if old_value {
                        collision = true;
                    }
                    // XOR logic
                    let display_value = if old_value { 0 } else { 1 };
                    self.set_pixel(xi, yj, display_value);
                }
            }
        }
        return collision;
    }

    fn set_pixel(&mut self, x: usize, y: usize, val: u8) {
        let index = x + y * self.width;
        if index < self.width * self.height {
            if self.memory[index] != val {
                self.memory[index] = val;
                self.display_updated = true;
            }
        }
    }

    fn get_pixel(&self, x: usize, y: usize) -> bool {
        let index = x + y * self.width;
        if index < self.width * self.height {
            self.memory[index] == 1
        } else {
            false
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
    use super::{Ppu, Display};

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
