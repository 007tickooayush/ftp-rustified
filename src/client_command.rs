use std::path::PathBuf;
use std::str::FromStr;

pub enum Command {
    AUTH,
    CWD(PathBuf),
    CDUP,
    LIST(Option<PathBuf>),
    MKD(PathBuf),
    NOOP,
    PORT(u16),
    PASS(String),
    PASV,
    PWD,
    QUIT,
    RETR(PathBuf),
    RMD(PathBuf),
    STOR(PathBuf),
    SYST,
    TYPE(DataTransferType),
    UNKNOWN(String),
    USER(String),
}

impl AsRef<str> for Command {
    fn as_ref(&self) -> &str {
        match *self {
            Command::AUTH => "AUTH",
            Command::CWD(_) => "CWD",
            Command::CDUP => "CDUP",
            Command::LIST(_) => "LIST",
            Command::MKD(_) => "MKD",
            Command::NOOP => "NOOP",
            Command::PORT(_) => "PORT",
            Command::PASS(_) => "PASS",
            Command::PASV => "PASV",
            Command::PWD => "PWD",
            Command::QUIT => "QUIT",
            Command::RETR(_) => "RETR",
            Command::RMD(_) => "RMD",
            Command::STOR(_) => "STOR",
            Command::SYST => "SYST",
            Command::TYPE(_) => "TYPE",
            Command::USER(_) => "USER",
            Command::UNKNOWN(_) => "UNKN",
        }
    }
}

pub enum DataTransferType {
    ASCII,
    IMAGE,
    UNKNOWN,
}