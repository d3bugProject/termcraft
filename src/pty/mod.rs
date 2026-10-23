use std::io::{Read, Write};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

/// Transposition de PTYTask.m dans iTerm2 (Gestion du processus Shell)
pub struct PtySession {
    child: Child,
    stdin_tx: Sender<Vec<u8>>,
    stdout_rx: Receiver<Vec<u8>>,
    pub shell_name: String,
}

impl PtySession {
    pub fn spawn(_cols: u16, _rows: u16) -> Result<Self, String> {
        let (shell_cmd, shell_name) = default_shell();

        let mut child = Command::new(&shell_cmd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Impossible de lancer le shell '{}' : {}", shell_cmd, e))?;

        let mut stdin = child.stdin.take().ok_or("Impossible d'attacher stdin")?;
        let mut stdout = child.stdout.take().ok_or("Impossible d'attacher stdout")?;

        let (stdin_tx, stdin_rx) = channel::<Vec<u8>>();
        let (stdout_tx, stdout_rx) = channel::<Vec<u8>>();

        // Thread d'écriture vers le shell (entrées clavier)
        thread::spawn(move || {
            while let Ok(data) = stdin_rx.recv() {
                if stdin.write_all(&data).is_err() || stdin.flush().is_err() {
                    break;
                }
            }
        });

        // Thread de lecture depuis le shell (sorties écran)
        thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match stdout.read(&mut buf) {
                    Ok(n) if n > 0 => {
                        if stdout_tx.send(buf[..n].to_vec()).is_err() {
                            break;
                        }
                    }
                    _ => break,
                }
            }
        });

        Ok(Self {
            child,
            stdin_tx,
            stdout_rx,
            shell_name,
        })
    }

    pub fn write_input(&self, data: &[u8]) {
        let _ = self.stdin_tx.send(data.to_vec());
    }

    pub fn read_output(&self) -> Option<Vec<u8>> {
        self.stdout_rx.try_recv().ok()
    }

    pub fn is_alive(&mut self) -> bool {
        match self.child.try_wait() {
            Ok(None) => true,
            _ => false,
        }
    }
}

fn default_shell() -> (String, String) {
    #[cfg(target_os = "windows")]
    {
        (
            std::env::var("COMSPEC").unwrap_or_else(|_| "powershell.exe".to_string()),
            "PowerShell".to_string(),
        )
    }

    #[cfg(target_os = "macos")]
    {
        (
            std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string()),
            "zsh".to_string(),
        )
    }

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        (
            std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string()),
            "bash".to_string(),
        )
    }
}
