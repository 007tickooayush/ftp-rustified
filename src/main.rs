mod client_command;
mod server;
mod ftp_config;
mod ftp_user;
mod utils;
mod client_handler;
mod ftp_response;
mod ftp_responce_code;

use dotenv::dotenv;
use crate::ftp_config::FtpConfig;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let addr = "0.0.0.0";
    let port = std::env::var("PORT").unwrap_or("2001".to_string());
    let host = format!("{}:{}", addr, port);

    let config = FtpConfig::new("ftp_server.json").await.unwrap();

}