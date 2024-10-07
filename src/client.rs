use std::path::PathBuf;
use std::result;
use tokio::io::{ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use crate::client_command::{Command, DataTransferType};
use crate::error::FtpError;
use crate::ftp_config::FtpConfig;

pub type Result<T> = result::Result<T, FtpError>;

pub struct Client {
    cwd: PathBuf,
    data_port: Option<u16>,
    data_reader: Option<ReadHalf<TcpStream>>,
    data_writer: Option<WriteHalf<TcpStream>>,
    name: Option<String>,
    server_root_dir: PathBuf,
    data_transfer_type: DataTransferType,
    writer: WriteHalf<TcpStream>,
    is_admin: bool,
    ftp_config: FtpConfig,
    waiting_password: bool
}

impl Client {
    pub fn new(writer: WriteHalf<TcpStream>, server_root_dir: PathBuf, ftp_config: FtpConfig) -> Self {
        Client {
            cwd: PathBuf::from("/"),
            data_port: None,
            data_reader: None,
            data_writer: None,
            name: None,
            server_root_dir,
            data_transfer_type: DataTransferType::ASCII,
            writer,
            is_admin: false,
            ftp_config,
            waiting_password: false
        }
    }

    pub fn is_logged_in(&self) -> bool {
        self.name.is_some() && !self.waiting_password
    }

    pub async fn handle_command(&self, cmd: Command) -> Result<Self> {
        unimplemented!("");
    }

}