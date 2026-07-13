use crate::error::Error;

pub mod utility;
pub mod window;

impl From<windows_core::Error> for Error {
    fn from(value: windows_core::Error) -> Self {
        Error::Windows(value)
    }
}
