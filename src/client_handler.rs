use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use crate::ftp_config::FtpConfig;

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

    pub async fn handle_client(&mut self) {
        unimplemented!("Need codec implementation")
    }
}