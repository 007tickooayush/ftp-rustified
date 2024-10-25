use std::path::{Path, PathBuf};
use std::result;
use std::str::from_utf8;
use serde_json::from_str;
use crate::error::FtpError;
use crate::utils::bytes_to_uppercase;

pub type Result<T> = result::Result<T, FtpError>;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Command {
    AUTH,
    CWD(PathBuf),
    CDUP,
    LIST(Option<String>, Option<PathBuf>),
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
        let mut command = iter.next().ok_or_else(|| FtpError::Msg("Empty command\r\n".to_string()))?.to_vec();
        bytes_to_uppercase(&mut command);

        let data = iter.next().ok_or_else(|| FtpError::Msg("No Command Parameter\r\n".to_string()));

        let command = match command.as_slice() {
            b"AUTH" => Command::AUTH,
            b"CWD" => Command::CWD(data.and_then(|bytes|  Ok(Path::new(from_utf8(bytes)?).to_path_buf()))?),
            b"CDUP" => Command::CDUP,
            // b"LIST" => Command::LIST(data.and_then(|bytes| Ok(Path::new(from_utf8(bytes)?).to_path_buf())).ok()),
            b"LIST" => {
                // let args = data.and_then(|bytes| Ok(from_str(from_utf8(bytes)?).unwrap_or(0)))?.to_string();
                // let path = iter.next().ok_or_else(|| FtpError::Msg("No Path provided\r\n".to_string()))?;
                // let path = Path::new(from_utf8(path)?).to_path_buf();
                //
                // Command::LIST(Some(args), Some(path))

                unimplemented!("LIST command not implemented")
            },
            b"PASV" => Command::PASV,
            b"MKD" => Command::MKD(data.and_then(|bytes| Ok(Path::new(from_utf8(bytes)?).to_path_buf()))?),
            b"PORT" => {
                let addr = data?.split(|&byte| byte == b',')
                    .filter_map(
                        |bytes| {
                            from_utf8(bytes).ok().and_then(|string| from_str(string).ok())
                        }
                    ).collect::<Vec<u8>>();
                if addr.len() != 6 {
                    return Err("Invalid address/port".into())
                }

                // Shifting the high byte by a8 bits left and performing bitwise OR with the low byte
                // and then Combining the two bytes to 16-bit port number
                let port = (addr[4] as u16) << 8 | (addr[5] as u16);

                if port <= 1024 {
                    return Err("Port can't be less than 10025".into());
                }
                Command::PORT(port)
            },
            b"PWD" => Command::PWD,
            b"QUIT" => Command::QUIT,
            b"RETR" => Command::RETR(data.and_then(|bytes| Ok(Path::new(from_utf8(bytes)?).to_path_buf()))?),
            b"RMD" => Command::RMD(data.and_then(|bytes| Ok(Path::new(from_utf8(bytes)?).to_path_buf()))?),
            b"STOR" => Command::RETR(data.and_then(|bytes| Ok(Path::new(from_utf8(bytes)?).to_path_buf()))?),
            b"SYST" => Command::SYST,
            b"TYPE" => {
                let err: Result<Command> = Err("Command not implemented".into());

                let data = data?;

                if data.is_empty() {
                    return err;
                }

                match DataTransferType::from(data[0]) {
                    DataTransferType::UNKNOWN => return err,
                    typ => {
                        Command::TYPE(typ)
                    }
                }
            },
            b"USER" => Command::USER(data.and_then(|bytes| String::from_utf8(bytes.to_vec()).map_err(Into::into))?),
            b"PASS" => Command::PASS(data.and_then(|bytes| String::from_utf8(bytes.to_vec()).map_err(Into::into))?),
            b"NOOP" => Command::NOOP,
            cmd => Command::UNKNOWN(from_utf8(cmd).unwrap_or("").to_owned())
        };

        Ok(command)
    }
}
impl AsRef<str> for Command {
    fn as_ref(&self) -> &str {
        match *self {
            Command::AUTH => "AUTH",
            Command::CWD(_) => "CWD",
            Command::CDUP => "CDUP",
            Command::LIST(_,_) => "LIST",
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

#[derive(Debug)]
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