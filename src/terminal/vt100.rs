use super::screen::Screen;
use crate::colors::{ColorPalette, RgbColor};

enum ParserState {
    Normal,
    Escape,
    Csi,
    Osc,
}

/// Transposition fidèle de VT100Terminal.m dans iTerm2
pub struct Vt100Parser {
    state: ParserState,
    params_buffer: String,
    osc_buffer: String,
}

impl Vt100Parser {
    pub fn new() -> Self {
        Self {
            state: ParserState::Normal,
            params_buffer: String::new(),
            osc_buffer: String::new(),
        }
    }

    pub fn process_stream(&mut self, input: &str, screen: &mut Screen, palette: &ColorPalette) {
        for c in input.chars() {
            self.process_char(c, screen, palette);
        }
    }

    pub fn process_char(&mut self, c: char, screen: &mut Screen, palette: &ColorPalette) {
        match self.state {
            ParserState::Normal => match c {
                '\x1b' => {
                    self.state = ParserState::Escape;
                }
                '\r' => {
                    screen.carriage_return();
                }
                '\n' => {
                    screen.newline();
                }
                '\t' => {
                    // Tabulation standard 8 espaces (port de iTerm2 tab stops)
                    let next_tab = (screen.cursor_x + 8) & !7;
                    screen.cursor_x = next_tab.min(screen.cols.saturating_sub(1));
                }
                '\x08' => {
                    screen.backspace();
                }
                '\x07' => {
                    // Cloche sonore (Bell)
                }
                other => {
                    screen.write_char(other);
                }
            },

            ParserState::Escape => match c {
                '[' => {
                    self.params_buffer.clear();
                    self.state = ParserState::Csi;
                }
                ']' => {
                    self.osc_buffer.clear();
                    self.state = ParserState::Osc;
                }
                'c' => {
                    // Reset terminal
                    screen.clear_screen();
                    screen.reset_styles();
                    self.state = ParserState::Normal;
                }
                _ => {
                    self.state = ParserState::Normal;
                }
            },

            ParserState::Csi => {
                if c.is_ascii_digit() || c == ';' || c == '?' {
                    self.params_buffer.push(c);
                } else {
                    self.dispatch_csi(c, screen, palette);
                    self.state = ParserState::Normal;
                }
            }

            ParserState::Osc => {
                if c == '\x07' || c == '\x1b' {
                    self.dispatch_osc(screen);
                    self.state = ParserState::Normal;
                } else {
                    self.osc_buffer.push(c);
                }
            }
        }
    }

    fn dispatch_csi(&self, cmd: char, screen: &mut Screen, palette: &ColorPalette) {
        let params: Vec<u32> = self
            .params_buffer
            .split(';')
            .filter_map(|s| s.parse().ok())
            .collect();

        match cmd {
            'm' => self.handle_sgr(&params, screen, palette),
            'H' | 'f' => {
                let row = params.first().copied().unwrap_or(1).saturating_sub(1) as usize;
                let col = params.get(1).copied().unwrap_or(1).saturating_sub(1) as usize;
                screen.set_cursor(col, row);
            }
            'A' => {
                let count = params.first().copied().unwrap_or(1) as usize;
                screen.cursor_y = screen.cursor_y.saturating_sub(count);
            }
            'B' => {
                let count = params.first().copied().unwrap_or(1) as usize;
                screen.cursor_y = (screen.cursor_y + count).min(screen.rows.saturating_sub(1));
            }
            'C' => {
                let count = params.first().copied().unwrap_or(1) as usize;
                screen.cursor_x = (screen.cursor_x + count).min(screen.cols.saturating_sub(1));
            }
            'D' => {
                let count = params.first().copied().unwrap_or(1) as usize;
                screen.cursor_x = screen.cursor_x.saturating_sub(count);
            }
            'J' => {
                let mode = params.first().copied().unwrap_or(0);
                if mode == 2 || mode == 3 {
                    screen.clear_screen();
                }
            }
            'K' => {
                screen.clear_to_end_of_line();
            }
            'h' => {
                if self.params_buffer == "?25" {
                    screen.cursor_visible = true;
                }
            }
            'l' => {
                if self.params_buffer == "?25" {
                    screen.cursor_visible = false;
                }
            }
            _ => {}
        }
    }

    fn handle_sgr(&self, params: &[u32], screen: &mut Screen, palette: &ColorPalette) {
        if params.is_empty() {
            screen.reset_styles();
            return;
        }

        let mut i = 0;
        while i < params.len() {
            match params[i] {
                0 => screen.reset_styles(),
                1 => screen.current_bold = true,
                2 => screen.current_dim = true,
                3 => screen.current_italic = true,
                4 => screen.current_underline = true,
                7 => screen.current_inverse = true,
                22 => {
                    screen.current_bold = false;
                    screen.current_dim = false;
                }
                23 => screen.current_italic = false,
                24 => screen.current_underline = false,
                27 => screen.current_inverse = false,

                // Couleurs de premier plan standard (30 à 37)
                code @ 30..=37 => {
                    screen.current_fg = palette.get_ansi((code - 30) as usize);
                }
                39 => screen.current_fg = screen.default_fg,

                // Couleurs d'arrière-plan standard (40 à 47)
                code @ 40..=47 => {
                    screen.current_bg = palette.get_ansi((code - 40) as usize);
                }
                49 => screen.current_bg = screen.default_bg,

                // Couleurs de premier plan haute intensité (90 à 97)
                code @ 90..=97 => {
                    screen.current_fg = palette.get_ansi((code - 90 + 8) as usize);
                }

                // Couleurs d'arrière-plan haute intensité (100 à 107)
                code @ 100..=107 => {
                    screen.current_bg = palette.get_ansi((code - 100 + 8) as usize);
                }

                // Couleurs étendues 256 couleurs ou TrueColor (24-bit RGB)
                38 => {
                    if i + 1 < params.len() {
                        if params[i + 1] == 2 && i + 4 < params.len() {
                            // TrueColor RGB: 38;2;R;G;B
                            screen.current_fg = RgbColor::new(
                                params[i + 2] as u8,
                                params[i + 3] as u8,
                                params[i + 4] as u8,
                            );
                            i += 4;
                        } else if params[i + 1] == 5 && i + 2 < params.len() {
                            // 256 colors: 38;5;index
                            let idx = params[i + 2] as usize;
                            if idx < 16 {
                                screen.current_fg = palette.get_ansi(idx);
                            }
                            i += 2;
                        }
                    }
                }

                48 => {
                    if i + 1 < params.len() {
                        if params[i + 1] == 2 && i + 4 < params.len() {
                            // TrueColor RGB: 48;2;R;G;B
                            screen.current_bg = RgbColor::new(
                                params[i + 2] as u8,
                                params[i + 3] as u8,
                                params[i + 4] as u8,
                            );
                            i += 4;
                        } else if params[i + 1] == 5 && i + 2 < params.len() {
                            let idx = params[i + 2] as usize;
                            if idx < 16 {
                                screen.current_bg = palette.get_ansi(idx);
                            }
                            i += 2;
                        }
                    }
                }

                _ => {}
            }
            i += 1;
        }
    }

    fn dispatch_osc(&self, screen: &mut Screen) {
        // Ex: "0;Nouvelle Fenêtre" ou "2;Mon Titre"
        if let Some(idx) = self.osc_buffer.find(';') {
            let title = &self.osc_buffer[idx + 1..];
            screen.title = title.to_string();
        }
    }
}
