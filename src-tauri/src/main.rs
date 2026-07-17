// Allbright Defi V60/2026 - Sovereign Desktop Application
// Main entry point for Tauri desktop application

#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

fn main() {
    allbright_desktop_lib::run();
}
