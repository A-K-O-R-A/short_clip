#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub use windows::create_listener;

#[cfg(target_os = "linux")]
pub mod x11;
#[cfg(target_os = "linux")]
pub use x11::create_listener;