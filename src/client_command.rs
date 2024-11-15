use std::path::{Path, PathBuf};
use std::result;
use std::str::from_utf8;
use serde::__private::from_utf8_lossy;
use serde_json::from_str;
use crate::error::FtpError;
use crate::utils::{bytes_to_uppercase, get_first_word_and_rest};

pub type Result<T> = result::Result<T, FtpError>;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Command {
    AUTH,
    CWD(PathBuf),
    CDUP,
    LIST(Option<String>),
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
    SIZE(PathBuf),
    SYST,
    TYPE(DataTransferType),
    UNKNOWN(String),
    USER(String),
}

impl Command {
    pub fn new(input: &str) -> Result<Self> {
        // let mut iter = input.split(|&byte| byte == b' ');
        // let mut command = iter.next().ok_or_else(|| FtpError::Msg("Empty command\r\n".to_string()))?.to_vec();
        // bytes_to_uppercase(&mut command);
        // let data = iter.next().ok_or_else(|| FtpError::Msg("No Command Parameter\r\n".to_string()));

        // NEW METHOD to get Command and rest of the contents
        let (command,data) = get_first_word_and_rest(input).ok_or(FtpError::Msg("Empty command\r\n".to_string()))?;

        println!("||X||Command: {:?}", command);
        println!("||X||Data: {:?}", data);


        let command = match command.as_bytes() {
            b"AUTH" => Command::AUTH,
            // b"CWD" => Command::CWD(data.and_then(|bytes|  Ok(Path::new(from_utf8(bytes)?).to_path_buf()))?),
            b"CWD" => Command::CWD(Path::new(data).to_path_buf()),
            b"CDUP" => Command::CDUP,
            // b"LIST" => Command::LIST(data.and_then(|bytes| Ok(from_utf8(bytes)?.to_string())).ok()),
            b"LIST" => Command::LIST(Some(data.to_string())),
            b"PASV" => Command::PASV,
            // b"MKD" => Command::MKD(data.and_then(|bytes| Ok(Path::new(from_utf8(bytes)?).to_path_buf()))?),
            b"MKD" => Command::MKD(Path::new(data).to_path_buf()),
            b"PORT" => {
                let addr: Vec<u8> = data.split(',')
                    .filter_map(|s| from_utf8(s.as_bytes()).ok())
                    .flat_map(|s| s.bytes())
                    .collect();

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
            // b"RETR" => Command::RETR(data.and_then(|bytes| Ok(Path::new(from_utf8(bytes)?).to_path_buf()))?),
            b"RETR" => Command::RETR(Path::new(data).to_path_buf()),
            // b"RMD" => Command::RMD(data.and_then(|bytes| Ok(Path::new(from_utf8(bytes)?).to_path_buf()))?),
            // b"DELE" => Command::RMD(data.and_then(|bytes| Ok(Path::new(from_utf8(bytes)?).to_path_buf()))?),
            b"RMD" => Command::RMD(Path::new(data).to_path_buf()),
            b"DELE" => Command::RMD(Path::new(data).to_path_buf()),
            b"STOR" => Command::STOR(Path::new(data).to_path_buf()),
            // b"SIZE" => Command::SIZE(data.and_then(|bytes| Ok(PathBuf::from(from_utf8(bytes)?)))?),
            b"SIZE" => Command::SIZE(Path::new(data).to_path_buf()),
            b"SYST" => Command::SYST,
            b"TYPE" => {
                let err: Result<Command> = Err("Command not implemented".into());
                if data.is_empty() {
                    return err;
                }

                match DataTransferType::from(data.as_bytes()[0]) {
                    DataTransferType::UNKNOWN => return err,
                    typ => {
                        Command::TYPE(typ)
                    }
                }
            },
            // b"USER" => Command::USER(data.and_then(|bytes| String::from_utf8(bytes.to_vec()).map_err(Into::into))?),
            b"USER" => Command::USER(data.to_string()),
            // b"PASS" => Command::PASS(data.and_then(|bytes| String::from_utf8(bytes.to_vec()).map_err(Into::into))?),
            b"PASS" => Command::PASS(data.to_string()),
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
            Command::LIST(_) => "LIST",
            Command::MKD(_) => "MKD",
            Command::NOOP => "NOOP",
            Command::PORT(_) => "PORT",
            Command::PASS(_) => "PASS",
            Command::PASV => "PASV",
            Command::PWD => "PWD",
            Command::QUIT => "QUIT",
            Command::RETR(_) => "RETR",
            Command::STOR(_) => "STOR",
            Command::RMD(_) => "RMD",
            Command::SYST => "SYST",
            Command::SIZE(_) => "SIZE",
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