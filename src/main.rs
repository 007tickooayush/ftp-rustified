mod tcp_handler;

use dotenv::dotenv;
use crate::tcp_handler::server::Server;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let addr = "0.0.0.0";
    let port = std::env::var("PORT").unwrap_or("2001".to_string());
    let host = format!("{}:{}", addr, port);

    let buf_size = std::env::var("BUF_SIZE").unwrap_or("1024".to_string());
    let server = Server::new(host,buf_size.parse().unwrap());
    server.start().await;

    // let server = tokio::net::TcpListener::bind(&host).await.unwrap();
    //
    // println!("Listening on: {}", host);
    //
    // loop {
    //     let (socket, addr) = server.accept().await.unwrap();
    //     tokio::spawn(async move {
    //         let (mut reader, mut writer) = tokio::io::split(socket);
    //         tokio::io::copy(&mut reader, &mut writer).await.unwrap();
    //     });
    // }
}