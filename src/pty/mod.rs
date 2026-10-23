#[cfg(windows)]
pub mod conpty;

#[cfg(windows)]
pub use conpty::WinConPty;

#[cfg(not(windows))]
pub struct UnixPty;
