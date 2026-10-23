use crate::colors::RgbColor;
use serde::{Deserialize, Serialize};

/// Représente une cellule individuelle dans la grille du terminal (port de screen_char_t dans iTerm2)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cell {
    pub c: char,
    pub fg: RgbColor,
    pub bg: RgbColor,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub dim: bool,
    pub inverse: bool,
}

impl Cell {
    pub fn new(c: char, fg: RgbColor, bg: RgbColor) -> Self {
        Self {
            c,
            fg,
            bg,
            bold: false,
            italic: false,
            underline: false,
            dim: false,
            inverse: false,
        }
    }

    pub fn default_blank(fg: RgbColor, bg: RgbColor) -> Self {
        Self::new(' ', fg, bg)
    }

    pub fn reset_attrs(&mut self) {
        self.bold = false;
        self.italic = false;
        self.underline = false;
        self.dim = false;
        self.inverse = false;
    }
}
