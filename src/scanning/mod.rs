pub mod platform;

#[cfg(windows)]
#[path = "windows/mod.rs"]
mod platform_version;

#[cfg(not(target_os = "windows"))]
#[path = "unknown/mod.rs"]
mod platform_version;
