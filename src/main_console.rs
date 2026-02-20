#[cfg(feature = "console_ui")]
extern crate env_logger;
#[cfg(feature = "console_ui")]
extern crate log;

#[cfg(feature = "console_ui")]
use sdl2::keyboard::Keycode;
#[cfg(feature = "console_ui")]
use sdl2::pixels::Color;
#[cfg(feature = "console_ui")]
use sdl2::rect::Rect;
#[cfg(feature = "console_ui")]
use sdl2::render::Canvas;
#[cfg(feature = "console_ui")]
use sdl2::video::Window;
#[cfg(feature = "console_ui")]
use std::collections::HashMap;
#[cfg(feature = "console_ui")]
use std::fs::File;
#[cfg(feature = "console_ui")]
use std::io::{Read, Result};
#[cfg(feature = "console_ui")]
use sdl2::Sdl;
#[cfg(feature = "console_ui")]
use std::{thread, time, env};

use hachip_core::cpu::Cpu;
use hachip_core::ppu; // Import ppu module
use hachip_core::ppu::Display as CoreDisplayTrait; // Alias to avoid name collision with local Display trait

// Re-introduce PixelGrid trait and CanvasWindow struct for SDL2 rendering
#[cfg(feature = "console_ui")]
pub trait PixelGrid {
    fn set_draw_color(&mut self, color: Color);
    fn clear(&mut self);
    fn present(&mut self);
    fn fill_rect(&mut self, rect: Rect) -> Result<(), String>;
}
#[cfg(feature = "console_ui")]
pub struct CanvasWindow {
    canvas: Canvas<Window>
}
#[cfg(feature = "console_ui")]
impl CanvasWindow {
    pub fn new(canvas: Canvas<Window>) -> CanvasWindow {
        CanvasWindow {
            canvas
        }
    }
}
#[cfg(feature = "console_ui")]
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


#[cfg(feature = "console_ui")]
fn main() -> Result<()> {
    env_logger::init();

    let KEYMAP: HashMap<Keycode, u8> = [
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
            .filter_map(|x| KEYMAP.get(&x))
            .cloned()
            .collect();
        cpu.keypad.press(keys);

        // Execute CPU cycle
        cpu.execute_cycle();

        // Render if display updated
        if cpu.ppu.take_display_update_flag() {
            canvas_window.clear(); // Clear the entire canvas
            let display_memory = cpu.ppu.get_display_memory();

            canvas_window.set_draw_color(draw_color); // Set color for drawing active pixels
            for y in 0..ppu::HEIGHT {
                for x in 0..ppu::WIDTH {
                    let index = x + y * ppu::WIDTH;
                    if display_memory[index] == 1 {
                        // Draw pixel if it's "on"
                        canvas_window.fill_rect(Rect::new(
                            (x * ppu::FACTOR) as i32,
                            (y * ppu::FACTOR) as i32,
                            ppu::FACTOR as u32,
                            ppu::FACTOR as u32,
                        ))?;
                    }
                }
            }
            canvas_window.present(); // Present the rendered frame
        }

        // Sync display at roughly 60Hz (8ms per frame)
        let display_sync = time::Duration::from_millis(8);
        thread::sleep(display_sync);
    }

    Ok(())
}

#[cfg(feature = "console_ui")]
fn open_rom(file_name: &str) -> Result<Vec<u8>> {
    #[cfg(feature = "console_ui")] // log::info requires log feature
    log::info!("load_game() {}", file_name);

    let file_metadata = std::fs::metadata(file_name)?;
    #[cfg(feature = "console_ui")] // log::info requires log feature
    log::info!("{} is {} bytes in size", file_name, file_metadata.len());

    let mut f = File::open(file_name)?;
    let mut buffer: Vec<u8> = vec![0; file_metadata.len() as usize];
    f.read_exact(&mut buffer)?;

    Ok(buffer)
}

#[cfg(not(feature = "console_ui"))]
fn main() {
    // This main will be compiled if console_ui feature is not enabled.
    // This is useful for building only the library without a runnable binary.
    panic!("This is a library-only build. The console UI is not enabled.");
}