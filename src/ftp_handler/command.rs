use std::path::Path;
use tokio::fs::{read_dir, File};
use tokio::io::AsyncReadExt;

pub async fn process_command(command: &str) -> String {
    let parts: Vec<&str> = command.trim().split_whitespace().collect();
    match parts[0].to_uppercase().as_str() {
        "USER" => "331 User name okay, need password.\r\n".to_string(),
        "PASS" => "230 User logged in, proceed.\r\n".to_string(),
        "PWD" => {
            let current_dir = std::env::current_dir().unwrap();
            format!("257 \"{}\"\r\n", current_dir.to_string_lossy())
        },
        "LIST" => {
            let mut response = String::new();
            if let Ok(mut entries) = read_dir(".").await {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    response.push_str(&format!("{}\r\n", entry.path().to_string_lossy()));
                }
            }
            format!("150 Here comes the directory listing.\r\n{}226 Directory send OK.\r\n", response)
        },
        "RETR" => {
            if parts.len() < 2 {
                return "501 Syntax error in parameters or arguments.\r\n".to_string();
            }
            let filename = parts[1];
            if Path::new(filename).exists() {
                let mut file = File::open(filename).await.unwrap();
                let mut contents = Vec::new();
                file.read_to_end(&mut contents).await.unwrap();
                "150 Opening data connection.\r\n226 Transfer complete.\r\n".to_string()
            } else {
                "550 Requested action not taken. File unavailable.\r\n".to_string()
            }
        },
        "QUIT" => "221 Goodbye.\r\n".to_string(),
        _ => "502 Command not implemented.\r\n".to_string(),
    }
}