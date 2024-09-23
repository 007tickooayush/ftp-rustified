mod client_command;
mod server;
mod ftp_config;
mod ftp_user;

use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let addr = "0.0.0.0";
    let port = std::env::var("PORT").unwrap_or("2001".to_string());
    let host = format!("{}:{}", addr, port);



}