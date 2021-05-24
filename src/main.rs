#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let app = zen_sm::ZenSM::default();
    eframe::run_native(Box::new(app), eframe::NativeOptions::default());
}
