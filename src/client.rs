use std::path::{PathBuf, StripPrefixError};
use std::{io, result};
use std::fs::Metadata;
use std::time::UNIX_EPOCH;
use cfg_if::cfg_if;
use time::OffsetDateTime;
use tokio::fs::{metadata, read_dir};
use tokio::io::{AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use crate::client_command::{Command, DataTransferType};
use crate::error::FtpError;
use crate::ftp_config::FtpConfig;
use crate::ftp_response_code::ResponseCode;
use crate::ftp_response::Response;
use crate::utils::{add_file_info, prefix_slash, CONFIG_FILE};

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
                Command::LIST(path) => return Ok(self.list(path).await?),

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
    async fn list(mut self, path_buf: Option<PathBuf>) -> Result<Self> {
        if self.data_writer.is_some() {
            let path = self.cwd.join(path_buf.unwrap_or_default());
            let directory = PathBuf::from(&path);
            let (new_client, complete_path) = self.complete_path(directory);
            self = new_client;
            if let Ok(path) = complete_path {
                self = self.send_response(
                    Response::new(ResponseCode::DataConnectionAlreadyOpen, "Starting to list directories")
                ).await?;

                let mut out = vec![];

                if path.is_dir() {
                    if let Ok(mut dir_reader) = read_dir(path).await{
                        while let Some(entry) = dir_reader.next_entry().await? {
                            if self.is_admin || entry.path() != self.server_root_dir.join(CONFIG_FILE) {
                                add_file_info(entry.path(), &mut out).await;
                            }
                        }
                        // self = self.send_response(Response::new(ResponseCode::ClosingDataConnection, "Directory send OK")).await?;
                    } else {
                        self = self.send_response(Response::new(ResponseCode::InvalidParameterOrArgument, "No such file or directory")).await?;
                        return Ok(self);
                    }
                } else if self.is_admin || path != self.server_root_dir.join(CONFIG_FILE) {
                    add_file_info(path, &mut out).await;
                }
                self = self.send_data(out).await?;
                println!("-> DONE TRAVERSING DIRECTORIES");
            } else {
                self = self.send_response(Response::new(ResponseCode::InvalidParameterOrArgument, "No such file or directory")).await?;
            }
        } else {
            self = self.send_response(Response::new(ResponseCode::ConnectionClosed, "No opened data connection")).await?;
        }

        if self.data_writer.is_some() {
            self.close_data_connection();
            self = self.send_response(Response::new(ResponseCode::ClosingDataConnection, "Directories Transfer done")).await?;
        }

        Ok(self)
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
        self.writer.write_all(&resp.to_bytes()).await?;
        Ok(self)
    }

    async fn send_data(mut self, data: Vec<u8>) -> Result<Self> {
        if let Some(mut writer) = self.data_writer {
            writer.write_all(&data).await?;
            self.data_writer = Some(writer)
        }
        Ok(self)
    }

    fn close_data_connection(&mut self) {
        self.data_reader = None;
        self.data_writer = None;
    }
}
