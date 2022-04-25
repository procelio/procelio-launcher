#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] //Hide console window in release builds on Windows, this blocks stdout.

use eframe::egui;
// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let app = procelio_launcher::ProcelioLauncher::default();
    let mut native_options = eframe::NativeOptions::default();
    native_options.initial_window_size = Some(egui::vec2(960.0, 540.0));
    native_options.resizable = false;
    eframe::run_native(Box::new(app), native_options);
}
