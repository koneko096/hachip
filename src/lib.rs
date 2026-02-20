pub mod cpu;
pub mod keypad;
pub mod ppu;
pub mod errors;

#[cfg(feature = "wasm_build")]
pub mod emulator; // Expose emulator module for WASM build

#[cfg(feature = "wasm_build")]
use wasm_bindgen::prelude::*;

// When the `console_error_panic_hook` feature is enabled, we can call the
// `set_panic_hook` function to get better error messages if we ever panic.
#[cfg(feature = "wasm_build")]
#[wasm_bindgen]
pub fn init_wasm() {
    console_error_panic_hook::set_once();
}

// A macro to provide `println!(..)`-style syntax for `console.log` in WASM.
#[cfg(feature = "wasm_build")]
#[macro_export]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}