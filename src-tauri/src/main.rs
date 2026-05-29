// Entry point. Hide the console window on Windows (no effect on macOS).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    libway_lib::run()
}
