# 8chip

8chip (read: ha-chip) is an experimental CHIP-8 emulator written in Rust.

## Requirements
- Rust (Stable)
- SDL2 (for console UI)
  - macOS: `brew install sdl2`
  - Linux: `sudo apt install libsdl2-dev`
  - Windows: SDL2 is bundled or can be installed via vcpkg

## Building

### Console UI
To build the console emulator with system SDL2:
```shell
cargo build --release --features console_ui
```

If you don't have SDL2 installed, you can use the bundled version (requires CMake):
```shell
cargo build --release --features console_ui,bundled_sdl
```

> **Note for Windows**: It is recommended to use the `bundled_sdl` feature.

### WASM
To build for Web/WASM:
```shell
wasm-pack build -- --features wasm_build
```

## Usage
Run the console binary with a ROM path:
```shell
./target/release/hachip_console <path_to_rom>
```
On Windows:
```shell
target\release\hachip_console.exe <path_to_rom>
```

## Keymap
The default keymap is:
```
1 2 3 4
Q W E R
A S D F
Z X C V
```
Which maps to the CHIP-8 hex keypad:
```
1 2 3 C
4 5 6 D
7 8 9 E
A 0 B F
```
