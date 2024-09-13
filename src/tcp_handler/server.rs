use std::path::PathBuf;
use dotenv::dotenv;
use crate::client_handler::client_handler::handle_client;

pub struct Server {
    addr: String,
    buf_size: u128,
    root_dir: PathBuf,
    curr_dir:PathBuf
}

impl Server {
    pub fn new(addr: String, buf_size:u128,) -> Self {
        dotenv().ok();
        let root_dir = PathBuf::from(std::env::var("ROOT_DIR").unwrap_or("PUBLIC".to_string()));
        Self {
            addr,
            buf_size,
            root_dir: root_dir.clone(),
            curr_dir: root_dir
        }
    }

    pub async fn start(&self) {
        let server = tokio::net::TcpListener::bind(&self.addr).await.unwrap();
        println!("Server is running on {}", self.addr);

        let size = self.buf_size;

        loop {
            let (socket, _) = server.accept().await.unwrap();

            tokio::spawn(async move {
                handle_client(socket,size).await.unwrap();
            });
        }

    }
}