pub mod tabs;

#[cfg(windows)]
pub mod win32_window;

pub use tabs::{Tab, TabManager};
