use std::path::{PathBuf, StripPrefixError};
use std::{io, result};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};
use tokio::fs::{metadata, read_dir, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
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
                Command::PASV => return Ok(self.pasv().await?),
                Command::PORT(port) => {
                    self.data_port = Some(port);
                    return Ok(self.send_response(Response::new(ResponseCode::Ok, &format!("PORT command successful, PORT: {}",port))).await?);
                },
                Command::PWD => {
                    let msg = format!("{}", self.cwd.to_str().unwrap_or(""));

                    if !msg.is_empty() {
                        let message = format!("\"{}\"",msg);
                        return Ok(self.send_response(Response::new(ResponseCode::PATHNAMECreated, &message)).await?);
                    } else {
                        return Ok(self.send_response(Response::new(ResponseCode::FileNotFound, "No such file or directory")).await?);
                    }
                },
                Command::RETR(file) => return Ok(self.retr(file).await?),
                _ => ()
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

    async fn pasv(mut self) -> Result<Self> {
        // Ok(self)
        // provide implementation for PASSIVE connection
        let port = if let Some(port) = self.data_port {
            port
        } else {
            0
        };

        if self.data_writer.is_some() {
            self = self.send_response(Response::new(ResponseCode::DataConnectionAlreadyOpen, "Data connection already open")).await?;
            return Ok(self);
        }

        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0,0,0,0)),port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        // new port
        let port = listener.local_addr()?.port();

        self = self.send_response(Response::new(ResponseCode::EnteringPassiveMode, &format!("Entering Passive Mode (0,0,0,0,{},{}).", port >> 8, port & 0xFF))).await?;

        println!("\t\tWaiting Incoming Clients on PORT: {}", port);

        for (stream, addr) in listener.accept().await {
            println!("\t\tNew Client Connected: {}", addr);
            let (reader, writer) = tokio::io::split(stream);
            self.data_reader = Some(reader);
            self.data_writer = Some(writer);

            break;
        }

        Ok(self)
    }

    async fn retr(mut self, path: PathBuf) -> Result<Self> {
        // checking for multiple data connections
        if self.data_writer.is_some() {
            let path = self.cwd.join(path);
            let (new_client, complete_path) = self.complete_path(path.clone());
            self = new_client;

            if let Ok(path) = complete_path {
                if path.is_file() && (self.is_admin || path != self.server_root_dir.join(CONFIG_FILE)) {
                    self = self.send_response(Response::new(ResponseCode::DataConnectionAlreadyOpen, "Starting to send the file")).await?;

                    let mut file = File::open(path).await?;

                    // reading the file all at once, but works for small files
                    // let mut outbound = vec![];
                    // file.read_to_end(&mut outbound).await?;


                    // reading File chunk by chunk (8KB chunk) and sending via buffer
                    let mut buffer = [0; 8192];
                    loop {
                        let bytes_read = file.read(&mut buffer).await?;
                        if bytes_read == 0 {
                            break;
                        }
                        self = self.send_data(buffer[..bytes_read].to_vec()).await?;
                    }
                    println!("\t\tTransfer Done ==>");
                } else {
                    let message = format!("\"{}\" doesnt exist", path.to_str().ok_or_else(|| FtpError::Msg("No Path".to_string()))?);
                    self = self.send_response(Response::new(ResponseCode::LocalErrorInProcessing, &message)).await?;
                }
            } else {
                let message = format!("\"{}\" doesnt exist", path.to_str().ok_or_else(|| FtpError::Msg("No Path".to_string()))?);
                self = self.send_response(Response::new(ResponseCode::LocalErrorInProcessing, &message)).await?;
            }
        } else {
            self = self.send_response(Response::new(ResponseCode::ConnectionClosed, "No opened data connection")).await?;
        }

        if self.data_writer.is_some() {
            self.close_data_connection();
            self = self.send_response(Response::new(ResponseCode::ClosingDataConnection, "Data connection closed, Transfer Done")).await?;
        }
        Ok(self)
    }
}
