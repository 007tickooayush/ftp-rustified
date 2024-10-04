use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio_io::_tokio_codec::Framed;
use crate::codec::FtpCodec;
use crate::ftp_config::FtpConfig;
use crate::ftp_responce_code::ResponseCode;
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

    pub async fn handle_client(&mut self) {
        // Using the tokio Framed implementation to handle the client
        let (mut reader, mut writer) = self.stream.split();
        let resp: Vec<u8> = Response::new(ResponseCode::ServiceReadyForNewUser, "Welcome to the FTP Server").into();
        writer.write_all(&resp).await.unwrap();


        unimplemented!("");
    }

    async fn handle_command(&self, cmd: Vec<u8>) -> Vec<u8> {
        unimplemented!("");
    }

}