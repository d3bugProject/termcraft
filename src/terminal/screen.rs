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

    // Buffer alternatif pour applications plein écran (ex: nvim, less, top)
    pub alt_grid: Option<Vec<Vec<Cell>>>,
    pub saved_cursor: (usize, usize),
    pub scroll_top: usize,
    pub scroll_bottom: usize,
}

impl Screen {
    pub fn new(cols: usize, rows: usize, default_fg: RgbColor, default_bg: RgbColor) -> Self {
        let cols = cols.max(1);
        let rows = rows.max(1);
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
            alt_grid: None,
            saved_cursor: (0, 0),
            scroll_top: 0,
            scroll_bottom: rows.saturating_sub(1),
        }
    }

    pub fn enter_alternate_screen(&mut self) {
        if self.alt_grid.is_none() {
            let alt = vec![vec![Cell::default_blank(self.default_fg, self.default_bg); self.cols]; self.rows];
            let normal = std::mem::replace(&mut self.grid, alt);
            self.alt_grid = Some(normal);
            self.saved_cursor = (self.cursor_x, self.cursor_y);
            self.cursor_x = 0;
            self.cursor_y = 0;
        }
    }

    pub fn leave_alternate_screen(&mut self) {
        if let Some(normal) = self.alt_grid.take() {
            self.grid = normal;
            self.cursor_x = self.saved_cursor.0.min(self.cols.saturating_sub(1));
            self.cursor_y = self.saved_cursor.1.min(self.rows.saturating_sub(1));
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
        if self.cursor_y < self.scroll_bottom {
            self.cursor_y += 1;
        } else {
            self.scroll_up(1);
        }
    }

    pub fn scroll_up(&mut self, count: usize) {
        for _ in 0..count {
            if self.scroll_top < self.scroll_bottom && self.scroll_bottom < self.grid.len() {
                let removed = self.grid.remove(self.scroll_top);
                if self.alt_grid.is_none() && self.scroll_top == 0 {
                    self.scrollback.push(removed);
                    if self.scrollback.len() > self.max_scrollback_lines {
                        self.scrollback.remove(0);
                    }
                }
                self.grid.insert(
                    self.scroll_bottom,
                    vec![Cell::default_blank(self.default_fg, self.default_bg); self.cols],
                );
            }
        }
    }

    pub fn scroll_down(&mut self, count: usize) {
        for _ in 0..count {
            if self.scroll_top < self.scroll_bottom && self.scroll_bottom < self.grid.len() {
                self.grid.remove(self.scroll_bottom);
                self.grid.insert(
                    self.scroll_top,
                    vec![Cell::default_blank(self.default_fg, self.default_bg); self.cols],
                );
            }
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
        if self.cursor_y < self.rows && self.cursor_x < self.cols {
            for col in self.cursor_x..self.cols {
                self.grid[self.cursor_y][col] = Cell::default_blank(self.default_fg, self.default_bg);
            }
        }
    }

    pub fn clear_from_start_of_line(&mut self) {
        if self.cursor_y < self.rows {
            let end = (self.cursor_x + 1).min(self.cols);
            for col in 0..end {
                self.grid[self.cursor_y][col] = Cell::default_blank(self.default_fg, self.default_bg);
            }
        }
    }

    pub fn clear_to_end_of_screen(&mut self) {
        self.clear_to_end_of_line();
        if self.cursor_y + 1 < self.rows {
            for r in (self.cursor_y + 1)..self.rows {
                for c in 0..self.cols {
                    self.grid[r][c] = Cell::default_blank(self.default_fg, self.default_bg);
                }
            }
        }
    }

    pub fn clear_from_start_of_screen(&mut self) {
        let max_r = self.cursor_y.min(self.rows);
        for r in 0..max_r {
            for c in 0..self.cols {
                self.grid[r][c] = Cell::default_blank(self.default_fg, self.default_bg);
            }
        }
        self.clear_from_start_of_line();
    }

    pub fn erase_chars(&mut self, count: usize) {
        if self.cursor_y < self.rows && self.cursor_x < self.cols {
            let end = (self.cursor_x + count).min(self.cols);
            for col in self.cursor_x..end {
                self.grid[self.cursor_y][col] = Cell::default_blank(self.default_fg, self.default_bg);
            }
        }
    }

    pub fn delete_chars(&mut self, count: usize) {
        if self.cursor_y < self.rows && self.cursor_x < self.cols {
            let row = &mut self.grid[self.cursor_y];
            for _ in 0..count {
                if self.cursor_x < row.len() {
                    row.remove(self.cursor_x);
                    row.push(Cell::default_blank(self.default_fg, self.default_bg));
                }
            }
        }
    }

    pub fn insert_chars(&mut self, count: usize) {
        if self.cursor_y < self.rows && self.cursor_x < self.cols {
            let row = &mut self.grid[self.cursor_y];
            for _ in 0..count {
                if self.cursor_x <= row.len() {
                    row.insert(self.cursor_x, Cell::default_blank(self.default_fg, self.default_bg));
                    row.pop();
                }
            }
        }
    }

    pub fn insert_lines(&mut self, count: usize) {
        if self.cursor_y <= self.scroll_bottom {
            let old_top = self.scroll_top;
            self.scroll_top = self.cursor_y;
            self.scroll_down(count);
            self.scroll_top = old_top;
        }
    }

    pub fn delete_lines(&mut self, count: usize) {
        if self.cursor_y <= self.scroll_bottom {
            let old_top = self.scroll_top;
            self.scroll_top = self.cursor_y;
            self.scroll_up(count);
            self.scroll_top = old_top;
        }
    }

    pub fn set_margins(&mut self, top: usize, bottom: usize) {
        let t = top.min(self.rows.saturating_sub(1));
        let b = bottom.min(self.rows.saturating_sub(1));
        if t < b {
            self.scroll_top = t;
            self.scroll_bottom = b;
        } else {
            self.scroll_top = 0;
            self.scroll_bottom = self.rows.saturating_sub(1);
        }
    }

    pub fn save_cursor(&mut self) {
        self.saved_cursor = (self.cursor_x, self.cursor_y);
    }

    pub fn restore_cursor(&mut self) {
        self.cursor_x = self.saved_cursor.0.min(self.cols.saturating_sub(1));
        self.cursor_y = self.saved_cursor.1.min(self.rows.saturating_sub(1));
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
        let new_cols = new_cols.max(1);
        let new_rows = new_rows.max(1);
        if new_cols == self.cols && new_rows == self.rows {
            return;
        }
        let mut new_grid = Vec::with_capacity(new_rows);
        for y in 0..new_rows {
            let mut row = Vec::with_capacity(new_cols);
            for x in 0..new_cols {
                if y < self.rows && x < self.cols && y < self.grid.len() && x < self.grid[y].len() {
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
        self.scroll_top = 0;
        self.scroll_bottom = new_rows.saturating_sub(1);
        self.cursor_x = self.cursor_x.min(new_cols.saturating_sub(1));
        self.cursor_y = self.cursor_y.min(new_rows.saturating_sub(1));
    }
}
