use std::path::{PathBuf, StripPrefixError};
use std::{io, result};
use tokio::io::{AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use crate::client_command::{Command, DataTransferType};
use crate::error::FtpError;
use crate::ftp_config::FtpConfig;
use crate::ftp_responce_code::ResponseCode;
use crate::ftp_response::Response;
use crate::utils::prefix_slash;

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

    pub async fn handle_command(mut self, cmd: Command) -> Result<Self> {
        if self.is_logged_in() {
            match cmd {
                Command::CWD(directory) => return Ok(self.handle_cwd(directory).await?),
                _ => unimplemented!()
            }
        } else if self.name.is_some() && self.waiting_password {
            unimplemented!("");
        }
        unimplemented!("Match implementation for Command")
    }

    async fn handle_cwd(mut self, directory: PathBuf) -> Result<Self> {
        let path = self.cwd.join(&directory);
        let (new_client, dir) = self.complete_path(path);

        self = new_client;

        if let Ok(dir) = dir {
            let (new_client, new_path) = self.strip_prefix(dir);
            self = new_client;

            if let Ok(prefix) = new_path {
                self.cwd = prefix.to_path_buf();
                prefix_slash(&mut self.cwd);
                let resp = Response::new(
                    ResponseCode::RequestedFileActionOkay,
                    &format!("Directory changed to \"{}\"", directory.display())
                );

                self = self.send_response(resp).await?;

                Ok(self)
            } else {
                self = self.send_response(Response::new(ResponseCode::FileNotFound, "No such file or directory")).await?;
                Ok(self)
            }
        } else {
            self = self.send_response(Response::new(ResponseCode::FileNotFound, "No such file or directory")).await?;
            Ok(self)
        }

    }

    fn complete_path(self, path: PathBuf) -> (Self, result::Result<PathBuf, io::Error>) {
        let directory = self.server_root_dir.join( if path.has_root() {
            path.iter().skip(1).collect()
        } else {
            path
        });
        let dir = directory.canonicalize();

        if let Ok(ref dir) = dir {
            if !dir.starts_with(&self.server_root_dir) {
                return (self, Err(io::ErrorKind::PermissionDenied.into()))
            }
        }

        (self,dir)
    }

    fn strip_prefix(self, dir: PathBuf) -> (Self, result::Result<PathBuf, StripPrefixError>) {
        let res = dir.strip_prefix(&self.server_root_dir).map(|p| p.to_path_buf());
        (self, res)
    }


    async fn send_response(mut self, resp: Response) -> Result<Self> {
        self.writer.write(&resp.to_bytes()).await?;
        Ok(self)
    }
}