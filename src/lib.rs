#![allow(dead_code)]

pub mod error;
pub mod models;
pub mod state;

#[cfg(target_os = "windows")]
mod windows;

pub mod platform {
    #[cfg(target_os = "windows")]
    pub use super::windows::*;
}
