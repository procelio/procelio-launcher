#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] //Hide console window in release builds on Windows, this blocks stdout.

use eframe::egui;
// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let mut app = procelio_launcher::ProcelioLauncher::default();
    let mut native_options = eframe::NativeOptions::default();
    native_options.viewport.inner_size = Some(egui::vec2(960.0, 540.0));
    native_options.viewport.min_inner_size = Some(egui::vec2(960.0, 540.0));
    native_options.viewport.max_inner_size = Some(egui::vec2(960.0, 540.0));
    native_options.viewport.resizable = Some(false);
    native_options.hardware_acceleration = eframe::HardwareAcceleration::Off;
    println!("Result: {:?}", eframe::run_native(
        &app.launcher_name.clone(),
        native_options,
        Box::new(|cc| {
            app.setup(cc);
            Ok(Box::new(app))
        })
    ));
}
