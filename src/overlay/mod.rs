mod elements;
mod events;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

pub use elements::*;
pub use events::*;

pub mod platform {
    #[cfg(target_os = "windows")]
    pub use super::windows::{
        utility,
        window::{setup, Window},
    };

    #[cfg(target_os = "linux")]
    pub use super::linux::{setup, Window};
}

pub use platform::*;
