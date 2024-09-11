use std::env::current_dir;
use std::net::IpAddr;
use std::path::Path;
use tokio::fs::{read_dir, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub async fn process_command(command_raw_str: &str) -> String {
    let parts: Vec<&str> = command_raw_str.trim().split_whitespace().collect();

    match parts[0].to_uppercase().as_str() {
        "USER" => {
            // todo!("Remove the hardcoded implementation for username")
            if parts[1] == "rustified_user" {
                String::from("331 User name okay.\r\n")
            } else {
                String::from("530 Authentication failed!\r\n")
            }
        }
        "PASS" => {
            // todo!("Remove the hardcoded implementation for password")
            if parts[1] == "maddogmaguire" {
                String::from("230 User logged in, proceed.\r\n")
            } else {
                String::from("530 Authentication failed!\r\n")
            }
        }
        "PWD" => {
            let current_dir = current_dir().unwrap();
            format!("257 \"{}\"\r\n", current_dir.to_string_lossy())
        }
        "LIST" => {
            let mut response = String::new();

            if let Ok(mut entries) = read_dir("./PUBLIC").await {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    response.push_str(&format!("{}\r\n", entry.path().to_string_lossy()));
                }
            }
            format!("150 Here comes the directory listing.\r\n{}\r\n226 Directory send OK.\r\n", response)
        }
        "RETR" => {
            if parts.len() < 2 {
                return String::from("501 No file name provided.\r\n");
            }

            let filename = parts[1];
            let file_path = Path::new("./PUBLIC").join(filename);

            if Path::new(filename).exists() {
                let mut file = File::open(file_path).await.unwrap();
                let mut contents = Vec::new();
                file.read_to_end(&mut contents).await.unwrap();
                format!("150 Opening BINARY mode data connection for {}\r\n{}\r\n226 Transfer complete.\r\n", filename, String::from_utf8_lossy(&contents))
            } else {
                String::from("550 Requested action not taken. File unavailable.\r\n")
            }
        }
        "PASV" => {
            //     Establishing a passive connection, for data channel
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let local_addr = listener.local_addr().unwrap();
            let port = local_addr.port();

            // Convert the address to the format required by the FTP protocol
            let (octet1, octet2, octet3, octet4) = match local_addr.ip() {
                IpAddr::V4(ipv4) => {
                    let octets = ipv4.octets();
                    (octets[0], octets[1], octets[2], octets[3])
                }
                IpAddr::V6(_) => return String::from("552 Network Protocol not supported.\r\n")
            };
            let p1 = port / 256;
            let p2 = port % 256;

            // Spawn a task to handle the incoming data connection
            // todo!("Provide an actual implementation for handling the scenarios)
            tokio::task::spawn(async move {
                if let Ok((mut socket, _)) = listener.accept().await {
                    // Handle the data connection here
                    // For example, you can read/write data to the socket
                    socket.write_all(b"Hello from the server!\r\n").await.unwrap();
                }
            });

            // Respond with the address and port
            format!("227 Entering Passive Mode ({},{},{},{},{},{}).\r\n", octet1, octet2, octet3, octet4, p1, p2)
        }
        _ => String::from("502 Command not implemented \r\n")
    }
}