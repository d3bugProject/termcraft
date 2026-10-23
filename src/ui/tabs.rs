use crate::colors::ColorPalette;
use crate::terminal::{Screen, Vt100Parser};

#[cfg(windows)]
use crate::pty::WinConPty;

pub struct Tab {
    pub id: usize,
    pub title: String,
    pub screen: Screen,
    pub parser: Vt100Parser,
    #[cfg(windows)]
    pub conpty: Option<WinConPty>,
}

impl Tab {
    pub fn new(id: usize, title: String, cols: usize, rows: usize, palette: &ColorPalette) -> Self {
        let screen = Screen::new(cols, rows, palette.foreground, palette.background);
        let parser = Vt100Parser::new();

        #[cfg(windows)]
        let conpty = WinConPty::spawn(cols as i16, rows as i16, "cmd.exe").ok();

        Self {
            id,
            title,
            screen,
            parser,
            #[cfg(windows)]
            conpty,
        }
    }

    #[cfg(windows)]
    pub fn poll_pty(&mut self, palette: &ColorPalette) -> bool {
        let mut updated = false;
        if let Some(ref conpty) = self.conpty {
            while let Some(bytes) = conpty.read_bytes() {
                let text = String::from_utf8_lossy(&bytes);
                self.parser.process_stream(&text, &mut self.screen, palette);
                updated = true;
            }
        }
        updated
    }

    #[cfg(windows)]
    pub fn send_input(&mut self, text: &str) {
        if let Some(ref conpty) = self.conpty {
            conpty.write_bytes(text.as_bytes());
        }
    }

    #[cfg(windows)]
    pub fn send_bytes(&mut self, bytes: &[u8]) {
        if let Some(ref conpty) = self.conpty {
            conpty.write_bytes(bytes);
        }
    }

    pub fn resize(&mut self, cols: usize, rows: usize) {
        self.screen.resize(cols, rows);
        #[cfg(windows)]
        if let Some(ref conpty) = self.conpty {
            conpty.resize(cols as i16, rows as i16);
        }
    }
}

pub struct TabManager {
    pub tabs: Vec<Tab>,
    pub active_tab_index: usize,
    next_tab_id: usize,
    pub default_cols: usize,
    pub default_rows: usize,
}

impl TabManager {
    pub fn new(cols: usize, rows: usize, palette: &ColorPalette) -> Self {
        let first_tab = Tab::new(1, "Tab 1".to_string(), cols, rows, palette);
        Self {
            tabs: vec![first_tab],
            active_tab_index: 0,
            next_tab_id: 2,
            default_cols: cols,
            default_rows: rows,
        }
    }

    pub fn new_tab(&mut self, palette: &ColorPalette) {
        let id = self.next_tab_id;
        self.next_tab_id += 1;
        let title = format!("Tab {}", id);
        let tab = Tab::new(id, title, self.default_cols, self.default_rows, palette);
        self.tabs.push(tab);
        self.active_tab_index = self.tabs.len() - 1;
    }

    pub fn close_current_tab(&mut self) {
        if self.tabs.len() > 1 {
            let idx = self.active_tab_index;
            self.tabs.remove(idx);
            if self.active_tab_index >= self.tabs.len() {
                self.active_tab_index = self.tabs.len().saturating_sub(1);
            }
        }
    }

    pub fn active_tab(&self) -> &Tab {
        &self.tabs[self.active_tab_index]
    }

    pub fn active_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.active_tab_index]
    }

    pub fn switch_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active_tab_index = index;
        }
    }
}
