use std::path::PathBuf;
use std::result;

pub type Result<T> = result::Result<T, Error>;

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

impl Command {
    pub fn new(input: Vec<u8>) -> Result<Self> {
        let mut iter = input.split(|&byte| byte == b' ');
        let command = iter.next().ok_or_else(|| "Empty command")?.to_vec();

        unimplemented!("Implement Error Struct");
    }
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

impl From<u8> for DataTransferType {
    fn from(val: u8) -> Self {
        match val {
            b'A' => DataTransferType::ASCII,
            b'I' => DataTransferType::IMAGE,
            _ => DataTransferType::UNKNOWN,
        }
    }
}