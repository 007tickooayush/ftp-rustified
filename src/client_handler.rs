use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use crate::client::Client;
use crate::client_command::Command;
use crate::ftp_config::FtpConfig;
use crate::ftp_response_code::ResponseCode;
use crate::ftp_response::Response;

pub struct ClientHandler {
    pub server_root_dir: PathBuf,
    pub ftp_config: FtpConfig
}

impl ClientHandler {
    pub fn new(server_root_dir: PathBuf, ftp_config: FtpConfig) -> Self {
        println!("\t\tCreated new ClientHandler");
        ClientHandler {
            server_root_dir,
            ftp_config
        }
    }

    pub async fn handle_client(self, writer:WriteHalf<TcpStream>, reader: ReadHalf<TcpStream>) {
        println!("\t\tHandling client");
        // let (reader, mut writer) = tokio::io::split(self.stream);
        // let (mut reader, mut writer) = self.stream.split();

        let mut client = Client::new(writer, self.server_root_dir.clone(), self.ftp_config.clone());

        // client.handle_command(reader).await.unwrap();
        let mut reader = BufReader::new(reader).lines();

        println!("outside while");
        // todo("unable to read the commands possibly")
        // loop {
        //     match reader.next_line().await {
        //         Ok(op_line) => {
        //             if let Some(line) = op_line {
        //                 let command = line.to_string();
        //                 println!("Command: {}", command);
        //             } else {
        //                 eprintln!("No line to read found");
        //                 break;
        //             }
        //         },
        //         Err(e) => {
        //           eprintln!("Error reading line: {}", e);
        //             break;
        //         }
        //     }
        // }

        while let Some(line) = reader.next_line().await.unwrap() {
            // todo("code not reaching here");
            println!("inside while");
            let command = line.trim().to_string();
            let cmd = Command::new(command.as_bytes().to_vec()).unwrap();
            client = client.handle_command(cmd).await.unwrap();
        }

        println!("\t\tCLIENT CLOSED<===");
    }

    pub async fn welcome_user(&mut self, writer: &mut WriteHalf<TcpStream>) {
        let resp = Response::new(ResponseCode::ServiceReadyForNewUser, "Welcome to the FTP Server").to_string();
        println!("\t\tRESPONSE in run: ==> {:?}", &resp);

        writer.write_all(resp.as_bytes()).await.unwrap();
    }
}