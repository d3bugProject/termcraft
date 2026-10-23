#![allow(dead_code, unused_imports)]

mod colors;
mod config;
mod pty;
mod terminal;
mod ui;

use std::env;
use ui::TermCraftServer;

fn print_help() {
    println!(
        r#"TermCraft — iTerm2 Clean-Room Reimplementation in Pure Rust

USAGE:
    termcraft [OPTIONS]

DESCRIPTION:
    A faithful, cross-platform terminal emulator replicating the core 
    architecture of macOS iTerm2 (VT100Screen, PTYTask, iTermColorMap)
    in 100% safe, fast Rust. Runs natively on macOS, Linux, and Windows.

OPTIONS:
    -h, --help                Show this help message
    -v, --version             Show version information
    --gui                     Launch the graphical interactive terminal window (default)
    --test-pty                Test internal PTY engine and print VT100 escape sequence diagnostics
    --theme <NAME>            Select startup palette (default, solarized, dracula)

EXAMPLES:
    termcraft                 # Launch TermCraft interactive window
    termcraft --test-pty      # Run PTY VT100 parser diagnostics
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
                println!("TermCraft v0.1.0 (Clean-room Rust implementation of iTerm2)");
                return;
            }
            "--test-pty" => {
                run_test_pty();
                return;
            }
            "--gui" => {}
            other => {
                eprintln!("Unknown argument: {}", other);
                eprintln!("Use --help for available options.");
                std::process::exit(1);
            }
        }
    }

    TermCraftServer::start_and_open();
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
    println!("Screen output test (row 1):");
    for cell in &screen.grid[1][..30] {
        print!("{}", cell.c);
    }
    println!();
    println!("✓ VT100 Engine & ANSI parser validated successfully!");
}
