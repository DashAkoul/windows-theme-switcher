// Hide the console window in release builds (this is a GUI tray app).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    windows_theme_switcher_lib::run();
}
