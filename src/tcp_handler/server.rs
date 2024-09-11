use crate::client_handler::client_handler::handle_client;

pub struct Server {
    addr: String,
    buf_size: u128
}

impl Server {
    pub fn new(addr: String, buf_size:u128) -> Self {
        Self {
            addr,
            buf_size
        }
    }

    pub async fn start(&self) {
        let server = tokio::net::TcpListener::bind(&self.addr).await.unwrap();
        println!("Server is running on {}", self.addr);

        let size = self.buf_size;

        loop {
            let (socket, _) = server.accept().await.unwrap();

            tokio::spawn(async move {
                handle_client(socket).await.unwrap();
            });
        }

    }
}