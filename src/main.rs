#![windows_subsystem = "windows"]
#![allow(dead_code, unused_imports)]

mod colors;
mod config;
mod pty;
mod terminal;
mod ui;

use std::env;

fn main() {
    #[cfg(windows)]
    {
        ui::win32_window::native::run_native_window();
    }

    #[cfg(not(windows))]
    {
        println!("TermCraft starting on Unix...");
    }
}
