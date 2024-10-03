use std::str::Utf8Error;
use std::string::FromUtf8Error;

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
