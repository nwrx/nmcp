use super::{Error, ErrorCode};
use core::fmt::{Debug, Display};

pub type Result<T, E = Error> = core::result::Result<T, E>;

pub trait ResultExt {
    type Ok;

    fn with_name<A>(self, name: A) -> Result<Self::Ok>
    where
        A: Display + Debug + Send + Sync + 'static;

    fn with_message<A>(self, message: A) -> Result<Self::Ok>
    where
        A: Display + Debug + Send + Sync + 'static;

    fn with_status<U>(self, status: U) -> Result<Self::Ok>
    where
        U: Into<ErrorCode> + Send + Sync + 'static;
}

impl<T, E> ResultExt for core::result::Result<T, E>
where
    E: Into<Error> + Send + Sync + 'static,
{
    type Ok = T;

    fn with_name<A>(self, name: A) -> Result<T>
    where
        A: Display + Debug + Send + Sync + 'static,
    {
        match self {
            Ok(value) => Ok(value),
            Err(error) => Err(error.into().with_name(name)),
        }
    }

    fn with_message<A>(self, message: A) -> Result<T>
    where
        A: Display + Debug + Send + Sync + 'static,
    {
        match self {
            Ok(value) => Ok(value),
            Err(error) => Err(error.into().with_message(message)),
        }
    }

    fn with_status<U>(self, status: U) -> Result<T>
    where
        U: Into<ErrorCode> + Send + Sync + 'static,
    {
        match self {
            Ok(value) => Ok(value),
            Err(error) => Err(error.into().with_status(status)),
        }
    }
}
