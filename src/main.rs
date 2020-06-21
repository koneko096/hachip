extern crate env_logger;
extern crate log;

use sdl2::keyboard::Keycode;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Result};
use crate::cpu::Cpu;
use sdl2::Sdl;
use crate::ppu::CanvasWindow;
use std::{thread, time, env};
use crate::errors::EmulateCycleError;

mod cpu;
mod keypad;
mod ppu;
mod errors;

fn main() {
    env_logger::init();

    let KEYMAP: HashMap<Keycode, u8> = [
        (Keycode::Num1, 0x1),
        (Keycode::Num2, 0x2),
        (Keycode::Num3, 0x3),
        (Keycode::Num4, 0xc),
        (Keycode::Q, 0x4),
        (Keycode::W, 0x5),
        (Keycode::E, 0x6),
        (Keycode::R, 0xd),
        (Keycode::A, 0x7),
        (Keycode::S, 0x8),
        (Keycode::D, 0x9),
        (Keycode::F, 0xe),
        (Keycode::Z, 0xa),
        (Keycode::X, 0x0),
        (Keycode::C, 0xb),
        (Keycode::V, 0xf),
    ].iter().cloned().collect();

    let sdl = sdl2::init().unwrap();
    let canvas = get_canvas(&sdl);
    let mut event_pump = sdl.event_pump().unwrap();
    let mut cpu = match init_cpu(canvas) {
        Ok(cpu) => cpu,
        Err(error) => panic!("Problem initiating cpu: {:?}", error),
    };

    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'main,
                _ => {}
            }
        }

        // Create a set of pressed Keys.
        let keys = event_pump
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .filter_map(|x| KEYMAP.get(&x))
            .cloned()
            .collect::<Vec<u8>>();

        cpu.keypad.press(keys);
        cpu.execute_cycle();

        let display_sync = time::Duration::from_millis(8);
        thread::sleep(display_sync);
    }
}

fn init_cpu(canvas: Canvas<Window>) -> Result<Cpu> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Invalid argument: no ROM specified")
    }

    let rom = args.get(1).unwrap();
    let ppu = ppu::Ppu::new(Box::new(CanvasWindow::new(canvas)));
    let mut cpu = cpu::Cpu::new(Box::new(ppu));
    let game = open_rom(rom).unwrap();
    cpu.reset();
    cpu.load(game);

    Result::Ok(cpu)
}

fn open_rom(file_name: &str) -> Result<Vec<u8>> {
    println!("load_game() {}", file_name);

    let file_metadata = std::fs::metadata(file_name)?;
    println!("{} is {} bytes in size", file_name, file_metadata.len());

    let mut f = File::open(file_name)?;
    let mut buffer: Vec<u8> = vec![0; file_metadata.len() as usize];
    f.read_exact(&mut buffer)?;

    Ok(buffer)
}

fn get_canvas(sdl: &Sdl) -> Canvas<Window> {
    let video_subsystem = sdl.video().unwrap();
    let _window = video_subsystem
        .window("hachip", ppu::FRAME_WIDTH, ppu::FRAME_HEIGHT)
        .resizable()
        .build()
        .unwrap();
    let canvas: Canvas<Window> = _window
        .into_canvas()
        // .present_vsync() //< this means the screen cannot
        // render faster than your display rate (usually 60Hz or 144Hz)
        .build()
        .unwrap();
    canvas
}
