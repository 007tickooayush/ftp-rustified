
#[macro_use]
extern crate cfg_if;


mod client_command;
mod server;
mod ftp_config;
mod ftp_user;
mod utils;
mod ftp_response;
mod ftp_response_code;
mod codec;
mod error;
mod client;

use std::path::PathBuf;
use dotenv::dotenv;
use crate::ftp_config::FtpConfig;
use crate::server::Server;
use crate::utils::{create_root_dir, get_current_dir};

#[tokio::main]
async fn main() {
    dotenv().ok();

    let config = FtpConfig::new("ftp_server.json").await.unwrap();

    let default_dir = get_current_dir();
    let default_dir = default_dir.to_str().unwrap().to_string();
    let root_dir = std::env::var("ROOT_DIR").unwrap_or(default_dir);


    // create_dir_all(&root_dir).await.unwrap();

    match create_root_dir(&root_dir).await {
        Ok(_) => {
            let root_dir = PathBuf::from(root_dir);
            let server = Server::new(root_dir, config);
            server.run().await;
        },
        Err(err) => {
            eprintln!("Error creating root directory: {}", err);
        }
    }

}