use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use dotenv::dotenv;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use crate::client::Client;
use crate::client_command::Command;
use crate::ftp_config::FtpConfig;
use crate::ftp_response::Response;
use crate::ftp_response_code::ResponseCode;

pub struct Server {
    root_dir_server: PathBuf,
    ftp_config: FtpConfig,
}

impl Server {
    pub fn new(root_dir_server: PathBuf, ftp_config: FtpConfig) -> Self {
        Server {
            root_dir_server,
            ftp_config,
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

        // corrected loop hierarchy
        loop {
            let (mut stream, addr) = listener.accept().await.unwrap();
            let root_dir_server = self.root_dir_server.clone();
            let ftp_config = self.ftp_config.clone();

            tokio::spawn(async move {
                // let (mut reader, mut writer) = stream.split();
                let (mut reader, mut writer) = tokio::io::split(stream);
                let mut reader_lines = BufReader::new(reader).lines();

                // NOTE: required to add \r\n after the message compulsorily for the client to be able to parse it
                let resp = Response::new(ResponseCode::ServiceReadyForNewUser, "Welcome to the FTP Server\r\n").to_string();
                println!("\t\tRESPONSE in run: ==> {:?}", &resp);

                writer.write_all(resp.as_bytes()).await.unwrap();
                // writer.write_all(b"220 Welcome to the FTP Server\r\n").await.unwrap();

                let mut client = Client::new(writer, root_dir_server.clone(), ftp_config.clone());
                loop {
                    // let mut buffer = [0; 1024];
                    // let n = reader.read(&mut buffer).await.unwrap();
                    // if n == 0 {
                    //     println!("No data received from client");
                    //     break;
                    // }
                    // let command = String::from_utf8_lossy(&buffer[..n]).to_string();
                    // println!("=====|INCOMING COMMAND:\n{}\n", &command);
                    // let resp = Response::new(ResponseCode::Ok, "Hello, World!").to_string();
                    // println!("=====|RESPONSE:\n{}\n", &resp);

                    while let Ok(line) = reader_lines.next_line().await {
                        // let command = line.unwrap();
                        if let Some(command) = line {
                            println!("--------inside while Reading Command");
                            println!("|||||| RAW Command: {} ||||||||", &command);
                            let command = command.trim().to_string();
                            let cmd = Command::new(command.as_bytes().to_vec()).unwrap();
                            client = client.handle_command(cmd).await.unwrap();
                        }
                        // else {
                        //     println!("-------No data received from client");
                        //     break;
                        // }
                    }
                }
            });
        }
    }
}

#[tokio::test]
async fn test_server() {
    dotenv().ok();
    let config = FtpConfig::new("ftp_server.json").await.unwrap();
    let server = Server::new(PathBuf::from("test"), config);
    let server_handle = tokio::spawn(async move {
        server.run().await;
    });
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    server_handle.abort();
}