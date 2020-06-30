use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

pub const FRAME_WIDTH: u32 = 640;
pub const FRAME_HEIGHT: u32 = 320;

const FACTOR: usize = 10;

pub trait PixelGrid {
    fn set_draw_color(&mut self, color: Color);
    fn clear(&mut self);
    fn present(&mut self);
    fn fill_rect(&mut self, rect: Rect) -> Result<(), String>;
}
pub struct CanvasWindow {
    canvas: Canvas<Window>
}
impl CanvasWindow {
    pub fn new(canvas: Canvas<Window>) -> CanvasWindow {
        CanvasWindow {
            canvas
        }
    }
}
impl PixelGrid for CanvasWindow {
    fn set_draw_color(&mut self, color: Color) {
        self.canvas.set_draw_color(color);
    }
    fn clear(&mut self) {
        self.canvas.clear();
    }
    fn present(&mut self) {
        self.canvas.present();
    }
    fn fill_rect(&mut self, rect: Rect) -> Result<(), String> {
        self.canvas.fill_rect(rect)
    }
}

pub trait Display {
    fn cls(&mut self);
    fn draw(&mut self, x: usize, y: usize, sprite: &[u8]) -> bool;
    fn set_pixel(&mut self, x: usize, y: usize, val: u8);
    fn get_pixel(&mut self, x: usize, y: usize) -> bool;
}
pub struct Ppu {
    memory: [u8; 2048],
    canvas: Box<dyn PixelGrid>
}
impl Ppu {
    pub fn new(canvas: Box<dyn PixelGrid>) -> Ppu {
        Ppu {
            memory: [0; 2048],
            canvas,
        }
    }
}

impl Display for Ppu {
    fn cls(&mut self) {
        self.memory = [0; 2048];
        let black = Color::RGB(0, 0, 0);
        self.canvas.set_draw_color(black);
        self.canvas.clear();
        self.canvas.present();
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
                    let old_value = self.get_pixel(xi, yj);
                    if old_value {
                        collision = true;
                    }
                    let display_value = ((new_value == 1) ^ old_value) as u8;
                    self.set_pixel(xi, yj, display_value);
                }
            }
        }
        self.canvas.present();
        return collision;
    }

    fn set_pixel(&mut self, x: usize, y: usize, val: u8) {
        self.memory[x + y * WIDTH] = val;
        let col = if val == 1
            { Color::RGB(255, 255, 255) }
            else
            { Color::RGB(0, 0, 0) };
        self.canvas.set_draw_color(col);
        self.canvas.fill_rect(Rect::new(
            (x * FACTOR) as i32,
            (y * FACTOR) as i32,
            FACTOR as u32,
            FACTOR as u32)).unwrap();
    }

    fn get_pixel(&mut self, x: usize, y: usize) -> bool {
        self.memory[x + y * WIDTH] == 1
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
    use super::Ppu;
    use crate::ppu::{PixelGrid, Display};
    use sdl2::pixels::Color;
    use sdl2::rect::Rect;

    pub struct PixelGridMock {}
    impl PixelGrid for PixelGridMock {
        fn set_draw_color(&mut self, color: Color) {}
        fn clear(&mut self) {}
        fn present(&mut self) {}
        fn fill_rect(&mut self, rect: Rect) -> Result<(), String> {
            Result::Ok(())
        }
    }
    
    fn make_pixel_grid() -> Box<dyn PixelGrid> {
        Box::new(PixelGridMock{})
    }

    #[test]
    fn set_pixel() {
        let mut Ppu = Ppu::new(make_pixel_grid());

        Ppu.set_pixel(1, 1, 1);

        assert_eq!(true, Ppu.get_pixel(1, 1));
    }

    #[test]
    fn cls() {
        let mut Ppu = Ppu::new(make_pixel_grid());

        Ppu.set_pixel(1, 1, 1);
        Ppu.cls();

        assert_eq!(false, Ppu.get_pixel(1, 1));
    }

    #[test]
    fn draw() {
        let mut Ppu = Ppu::new(make_pixel_grid());

        let sprite: [u8; 2] = [0b00110011, 0b11001010];

        Ppu.draw(0, 0, &sprite);

        assert_eq!(false, Ppu.get_pixel(0, 0));
        assert_eq!(false, Ppu.get_pixel(1, 0));
        assert_eq!(true, Ppu.get_pixel(2, 0));
        assert_eq!(true, Ppu.get_pixel(3, 0));
        assert_eq!(false, Ppu.get_pixel(4, 0));
        assert_eq!(false, Ppu.get_pixel(5, 0));
        assert_eq!(true, Ppu.get_pixel(6, 0));
        assert_eq!(true, Ppu.get_pixel(7, 0));

        assert_eq!(true, Ppu.get_pixel(0, 1));
        assert_eq!(true, Ppu.get_pixel(1, 1));
        assert_eq!(false, Ppu.get_pixel(2, 1));
        assert_eq!(false, Ppu.get_pixel(3, 1));
        assert_eq!(true, Ppu.get_pixel(4, 1));
        assert_eq!(false, Ppu.get_pixel(5, 1));
        assert_eq!(true, Ppu.get_pixel(6, 1));
        assert_eq!(false, Ppu.get_pixel(7, 1));
    }

    #[test]
    fn draw_detects_collisions() {
        let mut Ppu = Ppu::new(make_pixel_grid());

        let mut sprite: [u8; 1] = [0b00110000];
        let mut collision = Ppu.draw(0, 0, &sprite);
        assert_eq!(false, collision);

        sprite = [0b00000011];
        collision = Ppu.draw(0, 0, &sprite);
        assert_eq!(false, collision);

        sprite = [0b00000001];
        collision = Ppu.draw(0, 0, &sprite);
        assert_eq!(true, collision);
    }
}
