extern crate env_logger;
extern crate log;

use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read};
use std::{thread, time, env};

use hachip_core::cpu::Cpu;
use hachip_core::ppu; // Import ppu module

// Re-introduce PixelGrid trait and CanvasWindow struct for SDL2 rendering
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


fn main() -> io::Result<()> {
    env_logger::init();

    let keymap: HashMap<Keycode, u8> = [
        (Keycode::Num1, 0x1), (Keycode::Num2, 0x2), (Keycode::Num3, 0x3), (Keycode::Num4, 0xc),
        (Keycode::Q, 0x4), (Keycode::W, 0x5), (Keycode::E, 0x6), (Keycode::R, 0xd),
        (Keycode::A, 0x7), (Keycode::S, 0x8), (Keycode::D, 0x9), (Keycode::F, 0xe),
        (Keycode::Z, 0xa), (Keycode::X, 0x0), (Keycode::C, 0xb), (Keycode::V, 0xf),
    ].iter().cloned().collect();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window(
            "hachip",
            ppu::FRAME_WIDTH,
            ppu::FRAME_HEIGHT,
        )
        .resizable()
        .build()
        .unwrap();

    let mut canvas_window = CanvasWindow::new(window.into_canvas().build().unwrap());
    let mut event_pump = sdl_context.event_pump().unwrap();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Invalid argument: no ROM specified")
    }

    let rom_path = args.get(1).unwrap();
    let game = open_rom(rom_path)?;

    let mut cpu = Cpu::new(); // Cpu::new() no longer takes a display argument
    cpu.reset();
    cpu.load(game);

    let clear_color = Color::RGB(0, 0, 0);
    let draw_color = Color::RGB(255, 255, 255);

    'main_loop: loop {
        // Event handling
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'main_loop,
                _ => {}
            }
        }

        // Keypad input
        let keys: Vec<u8> = event_pump
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .filter_map(|x| keymap.get(&x))
            .cloned()
            .collect();
        cpu.keypad.press(keys);

        // Execute CPU cycles (roughly 600 instructions per second at 60Hz)
        for _ in 0..10 {
            cpu.execute_cycle();
        }

        // Tick Delay and Sound timers at 60Hz
        cpu.tick_timers();

        // Render if display updated
        if cpu.ppu.take_display_update_flag() {
            canvas_window.set_draw_color(clear_color);
            canvas_window.clear(); // Clear the entire canvas
            let display_memory = cpu.ppu.get_display_memory();

            canvas_window.set_draw_color(draw_color); // Set color for drawing active pixels
            let width = cpu.ppu.get_width();
            let height = cpu.ppu.get_height();
            let factor = if width == ppu::HIGH_RES_WIDTH { 5 } else { 10 };

            for y in 0..height {
                for x in 0..width {
                    let index = x + y * width;
                    if display_memory[index] == 1 {
                        // Draw pixel if it's "on"
                        canvas_window.fill_rect(Rect::new(
                            (x * factor) as i32,
                            (y * factor) as i32,
                            factor as u32,
                            factor as u32,
                        )).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                    }
                }
            }
            canvas_window.present(); // Present the rendered frame
        }

        // Sync display at 60Hz (roughly 16.6ms per frame)
        let display_sync = time::Duration::from_millis(16);
        thread::sleep(display_sync);
    }

    Ok(())
}

fn open_rom(file_name: &str) -> io::Result<Vec<u8>> {
    log::info!("load_game() {}", file_name);

    let file_metadata = std::fs::metadata(file_name)?;
    log::info!("{} is {} bytes in size", file_name, file_metadata.len());

    let mut f = File::open(file_name)?;
    let mut buffer: Vec<u8> = vec![0; file_metadata.len() as usize];
    f.read_exact(&mut buffer)?;

    Ok(buffer)
}