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
    Generic(Box<dyn std::error::Error + Send + Sync>),
}

impl<E> From<E> for Error
where
    E: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    fn from(value: E) -> Self {
        Error::Generic(value.into())
    }
}
