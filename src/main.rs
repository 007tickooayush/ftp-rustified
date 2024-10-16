
#[macro_use]
extern crate cfg_if;


mod client_command;
mod server;
mod ftp_config;
mod ftp_user;
mod utils;
mod client_handler;
mod ftp_response;
mod ftp_response_code;
mod codec;
mod error;
mod client;

use std::path::PathBuf;
use dotenv::dotenv;
use tokio::fs::create_dir_all;
use crate::ftp_config::FtpConfig;
use crate::server::Server;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let addr = "0.0.0.0";
    let port = std::env::var("PORT").unwrap_or("2001".to_string());
    let host = format!("{}:{}", addr, port);

    let config = FtpConfig::new("ftp_server.json").await.unwrap();

    let root_dir = std::env::var("ROOT_DIR").unwrap_or("ROOT".to_string());

    let root_dir = PathBuf::from(root_dir);
    create_dir_all(&root_dir).await.unwrap();


    let server = Server::new(root_dir, config);
    server.run().await;

}