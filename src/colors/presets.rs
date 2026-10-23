use super::{ColorPalette, RgbColor};

/// Palette officielle par défaut d'iTerm2
pub fn iterm_default() -> ColorPalette {
    ColorPalette {
        name: "iTerm2 Default".to_string(),
        background: RgbColor::new(24, 24, 24),
        foreground: RgbColor::new(204, 204, 204),
        cursor: RgbColor::new(199, 199, 199),
        selection_bg: RgbColor::new(65, 65, 65),
        selection_fg: RgbColor::new(255, 255, 255),
        ansi_colors: [
            RgbColor::new(0, 0, 0),       // 0: Black
            RgbColor::new(194, 54, 33),   // 1: Red
            RgbColor::new(37, 188, 36),   // 2: Green
            RgbColor::new(173, 173, 39),  // 3: Yellow
            RgbColor::new(73, 114, 237),  // 4: Blue
            RgbColor::new(211, 54, 130),  // 5: Magenta
            RgbColor::new(42, 161, 152),  // 6: Cyan
            RgbColor::new(234, 234, 234), // 7: White
            RgbColor::new(128, 128, 128), // 8: Bright Black
            RgbColor::new(239, 41, 41),   // 9: Bright Red
            RgbColor::new(138, 226, 52),  // 10: Bright Green
            RgbColor::new(252, 233, 79),  // 11: Bright Yellow
            RgbColor::new(52, 101, 164),  // 12: Bright Blue
            RgbColor::new(173, 127, 168), // 13: Bright Magenta
            RgbColor::new(52, 226, 226),  // 14: Bright Cyan
            RgbColor::new(255, 255, 255), // 15: Bright White
        ],
    }
}

/// Palette Solarized Dark (Preset emblématique d'iTerm2)
pub fn solarized_dark() -> ColorPalette {
    ColorPalette {
        name: "Solarized Dark".to_string(),
        background: RgbColor::new(0, 43, 54),
        foreground: RgbColor::new(131, 148, 150),
        cursor: RgbColor::new(147, 161, 161),
        selection_bg: RgbColor::new(7, 54, 66),
        selection_fg: RgbColor::new(147, 161, 161),
        ansi_colors: [
            RgbColor::new(7, 54, 66),
            RgbColor::new(220, 50, 47),
            RgbColor::new(133, 153, 0),
            RgbColor::new(181, 137, 0),
            RgbColor::new(38, 139, 210),
            RgbColor::new(211, 54, 130),
            RgbColor::new(42, 161, 152),
            RgbColor::new(238, 232, 213),
            RgbColor::new(0, 43, 54),
            RgbColor::new(203, 75, 22),
            RgbColor::new(88, 110, 117),
            RgbColor::new(101, 123, 131),
            RgbColor::new(147, 161, 161),
            RgbColor::new(108, 113, 196),
            RgbColor::new(147, 161, 161),
            RgbColor::new(253, 246, 227),
        ],
    }
}

/// Palette Dracula
pub fn dracula() -> ColorPalette {
    ColorPalette {
        name: "Dracula".to_string(),
        background: RgbColor::new(40, 42, 54),
        foreground: RgbColor::new(248, 248, 242),
        cursor: RgbColor::new(248, 248, 242),
        selection_bg: RgbColor::new(68, 71, 90),
        selection_fg: RgbColor::new(248, 248, 242),
        ansi_colors: [
            RgbColor::new(33, 34, 44),
            RgbColor::new(255, 85, 85),
            RgbColor::new(80, 250, 123),
            RgbColor::new(241, 250, 140),
            RgbColor::new(189, 147, 249),
            RgbColor::new(255, 121, 198),
            RgbColor::new(139, 233, 253),
            RgbColor::new(248, 248, 242),
            RgbColor::new(98, 114, 164),
            RgbColor::new(255, 110, 110),
            RgbColor::new(105, 255, 148),
            RgbColor::new(255, 255, 165),
            RgbColor::new(214, 172, 255),
            RgbColor::new(255, 146, 223),
            RgbColor::new(164, 255, 255),
            RgbColor::new(255, 255, 255),
        ],
    }
}
