use super::cell::Cell;
use crate::colors::RgbColor;

/// Transposition fidèle de VT100Screen.m dans iTerm2
pub struct Screen {
    pub cols: usize,
    pub rows: usize,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub cursor_visible: bool,
    pub current_fg: RgbColor,
    pub current_bg: RgbColor,
    pub default_fg: RgbColor,
    pub default_bg: RgbColor,
    pub current_bold: bool,
    pub current_italic: bool,
    pub current_underline: bool,
    pub current_dim: bool,
    pub current_inverse: bool,
    pub grid: Vec<Vec<Cell>>,
    pub scrollback: Vec<Vec<Cell>>,
    pub max_scrollback_lines: usize,
    pub title: String,
}

impl Screen {
    pub fn new(cols: usize, rows: usize, default_fg: RgbColor, default_bg: RgbColor) -> Self {
        let mut grid = Vec::with_capacity(rows);
        for _ in 0..rows {
            grid.push(vec![Cell::default_blank(default_fg, default_bg); cols]);
        }

        Self {
            cols,
            rows,
            cursor_x: 0,
            cursor_y: 0,
            cursor_visible: true,
            current_fg: default_fg,
            current_bg: default_bg,
            default_fg,
            default_bg,
            current_bold: false,
            current_italic: false,
            current_underline: false,
            current_dim: false,
            current_inverse: false,
            grid,
            scrollback: Vec::new(),
            max_scrollback_lines: 10_000,
            title: "TermCraft".to_string(),
        }
    }

    pub fn write_char(&mut self, c: char) {
        if self.cursor_x >= self.cols {
            self.cursor_x = 0;
            self.newline();
        }

        let mut cell = Cell::new(c, self.current_fg, self.current_bg);
        cell.bold = self.current_bold;
        cell.italic = self.current_italic;
        cell.underline = self.current_underline;
        cell.dim = self.current_dim;
        cell.inverse = self.current_inverse;

        if self.cursor_y < self.rows && self.cursor_x < self.cols {
            self.grid[self.cursor_y][self.cursor_x] = cell;
            self.cursor_x += 1;
        }
    }

    pub fn newline(&mut self) {
        if self.cursor_y + 1 < self.rows {
            self.cursor_y += 1;
        } else {
            // Défilement vers le haut : envoi de la ligne supérieure dans le scrollback
            let top_line = self.grid.remove(0);
            self.scrollback.push(top_line);
            if self.scrollback.len() > self.max_scrollback_lines {
                self.scrollback.remove(0);
            }
            self.grid.push(vec![Cell::default_blank(self.default_fg, self.default_bg); self.cols]);
        }
    }

    pub fn carriage_return(&mut self) {
        self.cursor_x = 0;
    }

    pub fn backspace(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
        }
    }

    pub fn set_cursor(&mut self, col: usize, row: usize) {
        self.cursor_x = col.min(self.cols.saturating_sub(1));
        self.cursor_y = row.min(self.rows.saturating_sub(1));
    }

    pub fn clear_screen(&mut self) {
        for row in &mut self.grid {
            for cell in row {
                *cell = Cell::default_blank(self.default_fg, self.default_bg);
            }
        }
        self.cursor_x = 0;
        self.cursor_y = 0;
    }

    pub fn clear_line(&mut self) {
        if self.cursor_y < self.rows {
            for cell in &mut self.grid[self.cursor_y] {
                *cell = Cell::default_blank(self.default_fg, self.default_bg);
            }
        }
    }

    pub fn clear_to_end_of_line(&mut self) {
        if self.cursor_y < self.rows {
            for col in self.cursor_x..self.cols {
                self.grid[self.cursor_y][col] = Cell::default_blank(self.default_fg, self.default_bg);
            }
        }
    }

    pub fn reset_styles(&mut self) {
        self.current_fg = self.default_fg;
        self.current_bg = self.default_bg;
        self.current_bold = false;
        self.current_italic = false;
        self.current_underline = false;
        self.current_dim = false;
        self.current_inverse = false;
    }

    pub fn resize(&mut self, new_cols: usize, new_rows: usize) {
        if new_cols == self.cols && new_rows == self.rows {
            return;
        }
        let mut new_grid = Vec::with_capacity(new_rows);
        for y in 0..new_rows {
            let mut row = Vec::with_capacity(new_cols);
            for x in 0..new_cols {
                if y < self.rows && x < self.cols {
                    row.push(self.grid[y][x]);
                } else {
                    row.push(Cell::default_blank(self.default_fg, self.default_bg));
                }
            }
            new_grid.push(row);
        }
        self.grid = new_grid;
        self.cols = new_cols;
        self.rows = new_rows;
        self.cursor_x = self.cursor_x.min(new_cols.saturating_sub(1));
        self.cursor_y = self.cursor_y.min(new_rows.saturating_sub(1));
    }
}
