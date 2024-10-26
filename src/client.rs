use std::path::{Path, PathBuf, StripPrefixError};
use std::{io, result};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::fs::{create_dir, read_dir, remove_dir_all, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use crate::client_command::{Command, DataTransferType};
use crate::error::FtpError;
use crate::ftp_config::FtpConfig;
use crate::ftp_response_code::ResponseCode;
use crate::ftp_response::Response;
use crate::utils::{add_file_info, get_current_dir, get_filename, invalid_path, prefix_slash, CONFIG_FILE};

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
        println!("-> COMMAND: {:?}", &cmd);
        if self.is_logged_in() {
            match cmd {
                Command::CWD(directory) => return Ok(self.handle_cwd(directory).await?),
                Command::LIST(args) => return Ok(self.list(args).await?),
                Command::PASV => return Ok(self.pasv().await?),
                Command::PORT(port) => {
                    self.data_port = Some(port);
                    return Ok(self.send_response(Response::new(ResponseCode::Ok, &format!("PORT command successful, PORT: {}\r\n",port))).await?);
                },
                Command::PWD => {
                    let msg = format!("{}", self.cwd.to_str().unwrap_or(""));

                    if !msg.is_empty() {
                        let message = format!("\"{}\"\r\n",msg);
                        return Ok(self.send_response(Response::new(ResponseCode::PATHNAMECreated, &message)).await?);
                    } else {
                        return Ok(self.send_response(Response::new(ResponseCode::FileNotFound, "No such file or directory\r\n")).await?);
                    }
                },
                Command::RETR(file) => return Ok(self.retr(file).await?),
                Command::STOR(file) => return Ok(self.stor(file).await?),
                Command::CDUP => {
                    if let Some(path) = self.cwd.parent().map(Path::to_path_buf) {
                        self.cwd = path;
                        prefix_slash(&mut self.cwd);
                    }
                    return Ok(self.send_response(Response::new(ResponseCode::Ok, "CDUP command successful\r\n")).await?);
                },

                Command::MKD(path) => return Ok(self.mkd(path).await?),
                Command::RMD(path) => return Ok(self.rmd(path).await?),
                _ => ()
            }
        } else if self.name.is_some() && self.waiting_password {
            if let Command::PASS(content) = cmd {
                let mut ok = false;
                if self.is_admin {
                    ok = content == self.ftp_config.admin.as_ref().unwrap().password;
                } else {
                    for user in &self.ftp_config.users {
                        if Some(&user.username) == self.name.as_ref() {
                            if user.password == content {
                                ok = true;
                                break;
                            }
                        }
                    }
                }
                if ok {
                    self.waiting_password = false;
                    let name = self.name.clone().unwrap_or(String::new());
                    self = self.send_response(Response::new(ResponseCode::UserLoggedIn, &format!("Welcome {}!\t\n", name))).await?;
                } else {
                    self = self.send_response(Response::new(ResponseCode::NotLoggedIn, "Invalid Password\r\n")).await?;
                }
                return Ok(self);
            }
        }
        match cmd {
            Command::AUTH => self = self.send_response(Response::new(ResponseCode::CommandNotImplemented, "Not Implemented\r\n")).await?,
            Command::QUIT => self = self.quit().await?,
            Command::SYST => {
                self = self.send_response(Response::new(ResponseCode::Ok, "Bugger Off\r\n")).await?;
            },
            Command::TYPE(type_) => {
                self.data_transfer_type = type_;
                self = self.send_response(Response::new(ResponseCode::Ok, "Data Transfer Type Changed Successfully\r\n")).await?;
            },
            Command::USER(content) => {
                if content.is_empty() {
                    self = self.send_response(Response::new(ResponseCode::InvalidParameterOrArgument, "Invalid Username\r\n")).await?;
                } else {
                    let mut name = None;
                    let mut password_req = true;

                    self.is_admin = false;

                    if let Some(ref admin) = self.ftp_config.admin {
                        if admin.username == content {
                            name = Some(content.clone());
                            password_req = admin.password.is_empty() == false;
                            self.is_admin = true;
                        }
                    }

                    if name.is_none() {
                        for user in &self.ftp_config.users {
                            if user.username == content {
                                name = Some(content.clone());
                                password_req = user.password.is_empty() == false;
                                break;
                            }
                        }
                    }

                    if name.is_none() {
                        self = self.send_response(Response::new(ResponseCode::NotLoggedIn, "Unknown User!\r\n")).await?;
                    } else {
                        self.name = name.clone();

                        if password_req {
                            self.waiting_password = true;
                            self = self.send_response(Response::new(ResponseCode::UserNameOkayNeedPassword, &format!("Provide password for {}\r\n", name.unwrap()))).await?;
                        } else {
                            self.waiting_password = false;
                            self = self.send_response(Response::new(ResponseCode::UserLoggedIn, &format!("Welcome {}!\r\n", name.unwrap()))).await?; // name == content
                        }
                    }
                }
            },
            Command::NOOP => self = self.send_response(Response::new(ResponseCode::Ok, "No Operation\r\n")).await?,
            Command::UNKNOWN(s) => self = self.send_response(Response::new(ResponseCode::UnknownCommand, &format!("\"{}\": [Command Not Implemented]\r\n",s))).await?,
            _ => {
                // handling the Command when User is not logged in
                self = self.send_response(Response::new(ResponseCode::NotLoggedIn, "Please Log In first\r\n")).await?;
            }
        }
        Ok(self)
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
                    &format!("Directory changed to \"{}\"\r\n", directory.display())
                );

                self = self.send_response(resp).await?;

                Ok(self)
            } else {
                self = self.send_response(Response::new(ResponseCode::FileNotFound, "No such file or directory\r\n")).await?;
                Ok(self)
            }
        } else {
            self = self.send_response(Response::new(ResponseCode::FileNotFound, "No such file or directory\r\n")).await?;
            Ok(self)
        }

    }

    /// Handling the List command
    /// NOTE: todo("Need to implement handling for parameters and not directly pass PathBuf")
    async fn list(mut self, args: Option<String>) -> Result<Self> {
        // , path_buf: Option<PathBuf>
        if let Some(command) = args {
            if command.starts_with('-') {
                if String::from("-al").eq(&command) {
                    // IMPLEMENTATION FOR -al
                    if self.data_writer.is_some() {
                        println!("<><><>DATA is some");
                        // let path = self.cwd.join(get_current_dir());
                        let path = self.cwd.clone();
                        let directory = PathBuf::from(&path);
                        let (new_client, complete_path) = self.complete_path(directory);
                        self = new_client;
                        if let Ok(path) = complete_path {
                            self = self.send_response(
                                Response::new(ResponseCode::DataConnectionAlreadyOpen, "Starting to list directories\r\n")
                            ).await?;

                            let mut out = vec![];

                            if path.is_dir() {
                                if let Ok(mut dir_reader) = read_dir(path).await{
                                    while let Some(entry) = dir_reader.next_entry().await? {
                                        if self.is_admin || entry.path() != self.server_root_dir.join(CONFIG_FILE) {
                                            add_file_info(entry.path(), &mut out).await;
                                        }
                                    }
                                    // self = self.send_response(Response::new(ResponseCode::ClosingDataConnection, "Directory send OK\r\n")).await?;
                                } else {
                                    self = self.send_response(Response::new(ResponseCode::InvalidParameterOrArgument, "No such file or directory\r\n")).await?;
                                    return Ok(self);
                                }
                            } else if self.is_admin || path != self.server_root_dir.join(CONFIG_FILE) {
                                add_file_info(path, &mut out).await;
                            }
                            self = self.send_data(out).await?;
                            println!("-> DONE TRAVERSING DIRECTORIES");
                        } else {
                            self = self.send_response(Response::new(ResponseCode::InvalidParameterOrArgument, "No such file or directory1\r\n")).await?;
                        }
                    } else {
                        self = self.send_response(Response::new(ResponseCode::ConnectionClosed, "No opened data connection2\r\n")).await?;
                    }

                } else {
                    self = self.send_response(Response::new(ResponseCode::ConnectionClosed, "No opened data connection3\r\n")).await?;
                }
            }
        }

        if self.data_writer.is_some() {
            self.close_data_connection();
            self = self.send_response(Response::new(ResponseCode::ClosingDataConnection, "Directories Transfer done\r\n")).await?;
        }

        Ok(self)
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
            self = self.send_response(Response::new(ResponseCode::DataConnectionAlreadyOpen, "Data connection already open\r\n")).await?;
            return Ok(self);
        }

        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0,0,0,0)),port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        // new port
        let port = listener.local_addr()?.port();

        self = self.send_response(Response::new(ResponseCode::EnteringPassiveMode, &format!("Entering Passive Mode (0,0,0,0,{},{}).\r\n", port >> 8, port & 0xFF))).await?;

        println!("\t\tWaiting Incoming Clients on PORT: {}", port);

        while let Ok((stream, addr)) = listener.accept().await {
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
                    self = self.send_response(Response::new(ResponseCode::DataConnectionAlreadyOpen, "Starting to send the file\r\n")).await?;

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
                    let message = format!("\"{}\" doesnt exist", path.to_str().ok_or_else(|| FtpError::Msg("No Path\r\n".to_string()))?);
                    self = self.send_response(Response::new(ResponseCode::LocalErrorInProcessing, &message)).await?;
                }
            } else {
                let message = format!("\"{}\" doesnt exist", path.to_str().ok_or_else(|| FtpError::Msg("No Path\r\n".to_string()))?);
                self = self.send_response(Response::new(ResponseCode::LocalErrorInProcessing, &message)).await?;
            }
        } else {
            self = self.send_response(Response::new(ResponseCode::ConnectionClosed, "No opened data connection\r\n")).await?;
        }

        if self.data_writer.is_some() {
            self.close_data_connection();
            self = self.send_response(Response::new(ResponseCode::ClosingDataConnection, "Data connection closed, Transfer Done\r\n")).await?;
        }
        Ok(self)
    }

    async fn stor(mut self, path: PathBuf) -> Result<Self> {
        println!("-> STOR: {:?}", &path);
        if self.data_reader.is_some() {
            if invalid_path(&path)  || (!self.is_admin && path == self.server_root_dir.join(CONFIG_FILE)){
                let error: io::Error = io::ErrorKind::PermissionDenied.into();
                return Err(error.into());
            }

            let path = self.cwd.join(path);
            self = self.send_response(Response::new(ResponseCode::DataConnectionAlreadyOpen, "Starting to Store the file\r\n")).await?;
            let (new_client, file_data) = self.receive_data().await?;
            self = new_client;

            let mut file = File::create(path).await?;
            file.write_all(&file_data).await?;

            println!("\t\tTransfer Done <==");

            self.close_data_connection();

            self = self.send_response(Response::new(ResponseCode::ClosingDataConnection, "Data connection closed, Transfer Done\r\n")).await?;
        } else {
            self = self.send_response(Response::new(ResponseCode::ConnectionClosed, "No opened data connection\r\n")).await?;
        }

        Ok(self)
    }

    async fn mkd(mut self, path: PathBuf) -> Result<Self> {
        let path = self.cwd.join(&path);
        let parent = self.get_parent(path.clone());

        if let Some(parent) = parent {
            let parent = parent.to_path_buf();

            let (new_client, complete_path) = self.complete_path(parent);
            self = new_client;

            if let Ok(mut dir) = complete_path {
                if dir.is_dir() {
                    let filename = get_filename(path);

                    if let Some(filename) = filename {
                        dir.push(filename);

                        if create_dir(dir).await.is_ok() {
                            self = self.send_response(Response::new(ResponseCode::PATHNAMECreated, "Directory created\r\n")).await?;
                            return Ok(self);
                        }

                    }
                }
            }
        }

        self = self.send_response(Response::new(ResponseCode::FileNotFound, "Unable to create Folder\r\n")).await?;

        Ok(self)
    }
    async fn rmd(mut self, directory: PathBuf) -> Result<Self> {
        let path = self.cwd.join(&directory);
        let (new_client, complete_path) = self.complete_path(path);
        self = new_client;

        if let Ok(dir) = complete_path {
            if remove_dir_all(dir).await.is_ok() {
                self = self.send_response(Response::new(ResponseCode::RequestedFileActionOkay, "Folder Removed successfully\r\n")).await?;
                return Ok(self);
            }
        }
        self = self.send_response(Response::new(ResponseCode::FileNotFound, "Couldn't Remove Folder\r\n")).await?;
        Ok(self)
    }

    fn get_parent(&self, path: PathBuf) -> Option<PathBuf> {
        path.parent().map(|p| p.to_path_buf())
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
        let resp_string = resp.to_string();
        println!("\t\t RESPONSE TO STRING: {}", &resp_string);
        self.writer.write_all(resp_string.as_bytes()).await?;
        Ok(self)
    }

    async fn send_data(mut self, data: Vec<u8>) -> Result<Self> {
        if let Some(mut writer) = self.data_writer {
            writer.write_all(&data).await?;
            self.data_writer = Some(writer)
        }
        Ok(self)
    }

    async fn receive_data(mut self) -> Result<(Self, Vec<u8>)> {
        if self.data_reader.is_some() {
            let mut file_data = vec![];

            let mut reader = self.data_reader.take().ok_or_else(|| FtpError::Msg("No data reader\r\n".to_string()))?;

            // Read the entire file data in one go
            // reader.read_to_end(&mut file_data).await?;

            // read the file data in chunks (8KB)
            let mut buffer = [0; 8192];
            loop {
                let bytes_read = reader.read(&mut buffer).await?;
                if bytes_read == 0 {
                    break;
                }
                file_data.extend_from_slice(&buffer[..bytes_read]);
            }


            Ok((self, file_data))
        } else {
            Ok((self, vec![]))
        }

    }

    fn close_data_connection(&mut self) {
        self.data_reader = None;
        self.data_writer = None;
    }

    async fn quit(mut self) -> Result<Self> {
        if self.data_writer.is_some() {
            unimplemented!("Not implemented if the Data Writer for the Stream is Present")
        } else {
            self = self.send_response(Response::new(ResponseCode::ServiceClosingControlConnection, "Closing Connection...\r\n")).await?;
            if let Some(mut writer) = self.data_writer.take() {
                writer.shutdown().await?;
            }
        }
        Ok(self)
    }
}