use tokio::io::AsyncReadExt;

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

        loop {
            let (mut tcp_stream, _) = server.accept().await.unwrap();
            let size = self.buf_size;

            tokio::spawn(async move {
                let mut buffer =vec![0;size as usize];

                match tcp_stream.read(&mut buffer).await {
                    Ok(_) => {
                        println!("-----------------------------------------------Incoming request------------------------------------\n{}", String::from_utf8_lossy(&buffer[..]));
                    },
                    Err(_) => {
                        eprintln!("Failed to read from connection");
                    }
                }
            });
        }
    }
}