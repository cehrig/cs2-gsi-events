use std::ffi::NulError;
use std::fmt::{Display, Formatter};
use std::num::ParseFloatError;

#[derive(Debug)]
pub enum Error {
    UnknownEvent,
    MemoryAllocationFailed,
    LibraryLoaderNotFound,
    WindowsClassCreate,
    GetSystemMetrics,
    D3DeviceMissing,
    D3ContextMissing,
    D3RenderTargetMissing,
    LockFailed,
    VolumeNotInRange,
    Generic(Box<dyn std::error::Error + Send + Sync>),
    Nul(NulError),
    ParseFloat(ParseFloatError),
    Io(std::io::Error),
    #[cfg(target_os = "windows")]
    Windows(windows_core::Error),
}

impl From<NulError> for Error {
    fn from(value: NulError) -> Self {
        Error::Nul(value)
    }
}

impl From<ParseFloatError> for Error {
    fn from(value: ParseFloatError) -> Self {
        Error::ParseFloat(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Io(value)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}
