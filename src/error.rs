use std::fmt::{Display, Formatter};
use std::str::Utf8Error;
use std::string::FromUtf8Error;
use std::{error, io, result};
use std::error::Error;
use crate::error::FtpError::Io;

pub type Result<T> = result::Result<T, FtpError>;

#[derive(Debug)]
pub enum FtpError {
    FromUtf8(FromUtf8Error),
    Io(tokio::io::Error),
    Msg(String),
    Utf8(Utf8Error)
}

impl FtpError {
    pub fn to_io_error(self) -> Self {
        match self {
            FtpError::Io(e) => e,
            FtpError::FromUtf8(_) | FtpError::Msg(_) | FtpError::Utf8(_) => tokio::io::ErrorKind::Other.into()
        }
    }
}


impl Display for  FtpError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            FtpError::FromUtf8(ref error) => error.fmt(f),
            FtpError::Io(ref error) => error.fmt(f),
            FtpError::Utf8(ref error) => error.fmt(f),
            FtpError::Msg(ref msg) => write!(f, "{}", msg),
        }
    }
}

/// Overriding the Error trait for FtpError
impl error::Error for FtpError {
    // fn description(&self) -> &str {
    //     match *self {
    //         FtpError::FromUtf8(ref e) => e.description(),
    //         FtpError::Io(ref e) => e.description(),
    //         FtpError::Utf8(ref e) => e.description(),
    //         FtpError::Msg(ref msg) => msg,
    //     }
    // }

    fn cause(&self) -> Option<&dyn Error> {
        let cause: &dyn error::Error = match *self {
            FtpError::FromUtf8(ref error) => error,
            FtpError::Io(ref error) => error,
            FtpError::Msg(ref error) => error,
            FtpError::Utf8(_) => return None,
        };

        Some(cause)
    }
}

// From trait implementation for each FtpError error type
impl From<io::Error> for FtpError {
    fn from(value: io::Error) -> Self {
        Io(value)
    }
}

impl<'a> From<&'a str> for FtpError {
    fn from(value: &'a str) -> Self {
        FtpError::Msg(value.to_string())
    }
}

impl From<Utf8Error> for FtpError {
    fn from(value: Utf8Error) -> Self {
        FtpError::Utf8(value)
    }
}

impl From<FromUtf8Error> for FtpError {
    fn from(value: FromUtf8Error) -> Self {
        FtpError::FromUtf8(value)
    }
}