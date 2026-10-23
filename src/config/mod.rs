use serde::{Deserialize, Serialize};

/// Port fidèle de iTermProfile et iTermPreferences.m
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub font_family: String,
    pub font_size: u16,
    pub color_preset: String,
    pub scrollback_lines: usize,
    pub transparency: f32, // 0.0 (opaque) à 1.0 (transparent)
    pub blur_enabled: bool,
    pub bell_enabled: bool,
    pub cursor_shape: String, // "Block", "Underline", "VerticalBar"
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            font_family: "JetBrains Mono, SF Mono, Menlo, monospace".to_string(),
            font_size: 13,
            color_preset: "iTerm2 Default".to_string(),
            scrollback_lines: 10_000,
            transparency: 0.05,
            blur_enabled: true,
            bell_enabled: false,
            cursor_shape: "Block".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermCraftConfig {
    pub default_profile: String,
    pub profiles: Vec<Profile>,
    pub hotkey_window_enabled: bool,
    pub confirm_closing_multiple_tabs: bool,
}

impl Default for TermCraftConfig {
    fn default() -> Self {
        Self {
            default_profile: "Default".to_string(),
            profiles: vec![Profile::default()],
            hotkey_window_enabled: true,
            confirm_closing_multiple_tabs: true,
        }
    }
}
