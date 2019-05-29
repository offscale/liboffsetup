pub mod platform;

#[cfg(windows)]
#[path = "windows/mod.rs"]
mod os;

#[cfg(not(target_os = "windows"))]
#[path = "unknown/mod.rs"]
mod os;
