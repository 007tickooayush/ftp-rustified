use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use crate::ftp_handler::command::process_command;

pub async fn handle_client(mut socket: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = [0; 1024];
    socket.write_all(b"220 Welcome to Rust FTP Server\r\n").await?;

    loop {
        let n = socket.read(&mut buffer).await?;
        if n == 0 {
            return Ok(());
        }

        let command = String::from_utf8_lossy(&buffer[..n]);
        println!("=====|INCOMING COMMAND:\n{}\n",&command);
        let response = process_command(&command).await;
        socket.write_all(response.as_bytes()).await?;
    }
}