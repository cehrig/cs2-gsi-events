pub enum Error {
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
