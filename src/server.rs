use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use dotenv::dotenv;
use tokio::net::TcpListener;
use crate::ftp_config::FtpConfig;

pub struct Server {

    root_dir_server: PathBuf,
    ftp_config: FtpConfig
}

impl Server {
    pub fn new(root_dir_server: PathBuf, ftp_config: FtpConfig) -> Self {
        Server {
            root_dir_server,
            ftp_config
        }
    }

    pub async fn run(&self) {
        dotenv().ok();
        let addr = &self.ftp_config.addr;
        let port = &self.ftp_config.port;

        let socket_addr = SocketAddr::new(IpAddr::V4(addr.parse().unwrap()), *port);

        println!("Client expected at {}", socket_addr);

        let listener = TcpListener::bind(&socket_addr).await.unwrap();
        loop {

        }

    }
}