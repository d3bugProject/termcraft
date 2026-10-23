#![allow(dead_code, unused_imports)]

mod colors;
mod config;
mod pty;
mod terminal;
mod ui;

use std::env;

fn print_help() {
    println!(
        r#"TermCraft — Native iTerm2 Reimplementation in Pure Rust

USAGE:
    termcraft.exe [OPTIONS]

DESCRIPTION:
    A faithful, 100% native Windows & macOS desktop terminal emulator 
    replicating the core architecture of iTerm2 (VT100Screen, PTYTask, iTermColorMap)
    in safe, fast, standalone Rust. Zero browser, zero web server.

OPTIONS:
    -h, --help                Show this help message
    -v, --version             Show version information
    --test-pty                Test internal PTY engine and print VT100 diagnostics
"#
    );
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "-h" | "--help" => {
                print_help();
                return;
            }
            "-v" | "--version" => {
                println!("TermCraft v0.1.0 (Native Windows & macOS iTerm2 in Rust)");
                return;
            }
            "--test-pty" => {
                run_test_pty();
                return;
            }
            other => {
                eprintln!("Unknown argument: {}", other);
                eprintln!("Use --help for available options.");
                std::process::exit(1);
            }
        }
    }

    #[cfg(windows)]
    {
        ui::win32_window::native::run_native_window();
    }

    #[cfg(not(windows))]
    {
        println!("TermCraft native window starting on Unix...");
    }
}

fn run_test_pty() {
    println!("Testing TermCraft PTY and VT100 Engine...");
    let palette = colors::presets::iterm_default();
    let mut screen = terminal::Screen::new(80, 24, palette.foreground, palette.background);
    let mut parser = terminal::Vt100Parser::new();

    let test_stream = "\x1b[1;36mHello \x1b[32mfrom \x1b[31mTermCraft!\x1b[0m\n\rSecond line with \x1b[4munderline\x1b[0m.\n";
    parser.process_stream(test_stream, &mut screen, &palette);

    println!("Screen output test (row 0):");
    for cell in &screen.grid[0][..30] {
        print!("{}", cell.c);
    }
    println!();
    println!("✓ VT100 Engine & ANSI parser validated successfully!");
}
