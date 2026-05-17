pub mod elements;
pub mod events;
pub mod utility;
pub mod window;

use crate::error::Error;
pub use elements::*;
pub use events::*;
pub use utility::*;
pub use window::*;

impl From<windows_core::Error> for Error {
    fn from(value: windows_core::Error) -> Self {
        Error::Windows(value)
    }
}
