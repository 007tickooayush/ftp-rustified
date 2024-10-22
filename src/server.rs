use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use dotenv::dotenv;
use tokio::net::TcpListener;
use crate::client_handler::ClientHandler;
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
        println!("\t\tRunning server at: {}", socket_addr);
        println!("\t\tClient expected at {}", socket_addr);

        let listener = TcpListener::bind(&socket_addr).await.unwrap();

        loop {
            while let  Ok((stream, addr)) = listener.accept().await {
                let address = format!("[address: {}]",addr);
                println!("====New client connected: {}", address);

                let root_dir_server = self.root_dir_server.clone();
                let ftp_config = self.ftp_config.clone();

                tokio::spawn(async move {
                    println!("\t\tHandling client: {}", address);
                    let client = ClientHandler::new(stream, root_dir_server.clone(), ftp_config.clone());
                    client.handle_client().await;
                });
            }
        }

    }
}

#[tokio::test]
async fn test_server() {
    dotenv().ok();
    let config = FtpConfig::new("ftp_server.json").await.unwrap();
    let server = Server::new(PathBuf::from("test"), config);
    let server_handle = tokio::spawn( async move {
        server.run().await;
    });
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    server_handle.abort();
}