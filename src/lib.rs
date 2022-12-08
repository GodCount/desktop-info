#[macro_use]
extern crate napi_derive;

#[cfg(target_os = "windows")]
pub mod window;

#[cfg(target_os = "macos")]
pub mod macos;