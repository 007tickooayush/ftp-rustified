mod tcp_handler;
mod ftp_handler;
mod client_handler;
mod file_system_handler;

use dotenv::dotenv;
use crate::tcp_handler::server::Server;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let addr = "0.0.0.0";
    let port = std::env::var("PORT").unwrap_or("2001".to_string());
    let host = format!("{}:{}", addr, port);

    let buf_size = std::env::var("BUF_SIZE").unwrap_or("1024".to_string());
    Server::new(host,buf_size.parse().unwrap()).run().await;

}