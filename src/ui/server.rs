use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::colors::presets;
use crate::ui::TabManager;

const HTML_CONTENT: &str = include_str!("index.html");

pub struct TermCraftServer;

impl TermCraftServer {
    pub fn start_and_open() {
        let listener = match TcpListener::bind("127.0.0.1:0") {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Failed to bind server: {}", e);
                return;
            }
        };

        let port = listener.local_addr().unwrap().port();
        let url = format!("http://127.0.0.1:{}", port);

        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║            TERMCRAFT — iTerm2 Reimplementation in Rust       ║");
        println!("╚══════════════════════════════════════════════════════════════╝");
        println!("Opening TermCraft window at: {}", url);

        open_browser(&url);

        let default_palette = presets::iterm_default();
        let tab_manager = Arc::new(Mutex::new(TabManager::new(120, 36, &default_palette)));

        for stream in listener.incoming() {
            if let Ok(stream) = stream {
                let tm = Arc::clone(&tab_manager);
                thread::spawn(move || {
                    handle_client(stream, tm);
                });
            }
        }
    }
}

fn open_browser(url: &str) {
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("open").arg(url).spawn();
    }

    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("cmd").args(["/c", "start", url]).spawn();
    }

    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    {
        let _ = Command::new("xdg-open").arg(url).spawn();
    }
}

fn handle_client(mut stream: TcpStream, _tab_manager: Arc<Mutex<TabManager>>) {
    let mut buffer = [0u8; 8192];
    let bytes_read = match stream.read(&mut buffer) {
        Ok(n) if n > 0 => n,
        _ => return,
    };

    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    let mut lines = request.lines();
    let req_line = lines.next().unwrap_or("");
    let parts: Vec<&str> = req_line.split_whitespace().collect();

    if parts.len() < 2 {
        return;
    }

    let method = parts[0];
    let path = parts[1];

    if method == "GET" && (path == "/" || path.is_empty()) {
        send_response(&mut stream, 200, "text/html; charset=utf-8", HTML_CONTENT.as_bytes());
    } else if method == "POST" && path.starts_with("/api/exec") {
        // Extract command payload from body
        let body = if let Some(idx) = request.find("\r\n\r\n") {
            &request[idx + 4..]
        } else {
            ""
        };

        let cmd_to_run = if let Ok(json) = serde_json::from_str::<serde_json::Value>(body) {
            json["cmd"].as_str().unwrap_or("").to_string()
        } else {
            "".to_string()
        };

        let output = execute_shell_command(&cmd_to_run);
        let resp = serde_json::json!({
            "output": output,
            "status": "ok"
        });
        let json_bytes = serde_json::to_vec(&resp).unwrap_or_default();
        send_response(&mut stream, 200, "application/json", &json_bytes);
    } else {
        send_response(&mut stream, 404, "text/plain", b"Not Found");
    }
}

fn execute_shell_command(cmd: &str) -> String {
    if cmd.trim().is_empty() {
        return String::new();
    }

    #[cfg(target_os = "windows")]
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", cmd])
        .output();

    #[cfg(not(target_os = "windows"))]
    let output = Command::new("/bin/sh")
        .args(["-c", cmd])
        .output();

    match output {
        Ok(out) => {
            let mut s = String::from_utf8_lossy(&out.stdout).to_string();
            if !out.stderr.is_empty() {
                s.push_str("\n\x1b[31m");
                s.push_str(&String::from_utf8_lossy(&out.stderr));
                s.push_str("\x1b[0m");
            }
            if s.is_empty() {
                s = "(Command completed with no output)".to_string();
            }
            s
        }
        Err(e) => format!("\x1b[31mExecution Error: {}\x1b[0m", e),
    }
}

fn send_response(stream: &mut TcpStream, status: u16, content_type: &str, body: &[u8]) {
    let status_line = match status {
        200 => "HTTP/1.1 200 OK",
        404 => "HTTP/1.1 404 Not Found",
        _ => "HTTP/1.1 500 Internal Server Error",
    };

    let response_headers = format!(
        "{}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\n\r\n",
        status_line,
        content_type,
        body.len()
    );

    let _ = stream.write_all(response_headers.as_bytes());
    let _ = stream.write_all(body);
    let _ = stream.flush();
}
