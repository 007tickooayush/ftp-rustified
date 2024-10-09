use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use crate::client::Client;
use crate::client_command::Command;
use crate::ftp_config::FtpConfig;
use crate::ftp_response_code::ResponseCode;
use crate::ftp_response::Response;

pub struct ClientHandler {
    pub stream: TcpStream,
    pub server_root_dir: PathBuf,
    pub ftp_config: FtpConfig
}

impl ClientHandler {
    pub fn new(stream: TcpStream, server_root_dir: PathBuf, ftp_config: FtpConfig) -> Self {
        ClientHandler {
            stream,
            server_root_dir,
            ftp_config
        }
    }

    pub async fn handle_client(mut self) {
        // Using the tokio Framed implementation to handle the client
        let (mut reader, mut writer) = tokio::io::split(self.stream);

        let resp = &Response::new(ResponseCode::ServiceReadyForNewUser, "Welcome to the FTP Server").to_bytes();
        writer.write(resp).await.unwrap();

        let mut client = Client::new(writer, self.server_root_dir.clone(), self.ftp_config.clone());

        // client.handle_command(reader).await.unwrap();
        let mut reader = BufReader::new(reader).lines();

        while let Some(line) = reader.next_line().await.unwrap() {
            let command = line.trim().to_string();
            let cmd = Command::new(command.as_bytes().to_vec()).unwrap();
            client = client.handle_command(cmd).await.unwrap();
        }

        println!("CLIENT CLOSED");
    }
}