use std::path::Path;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use crate::ftp_user::FtpUser;
use crate::utils::get_content;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FtpConfig {
    pub port: u16,
    pub addr: String,
    pub admin: Option<FtpUser>,
    pub users: Vec<FtpUser>
}

impl FtpConfig {

    /// Function to load the config file
    pub async fn new<P: AsRef<Path>>(file_path: P) -> Option<FtpConfig> {
        dotenv().ok();
        // let addr = "0.0.0.0";
        // let port = std::env::var("PORT").unwrap_or("2001".to_string());
        // let host = format!("{}:{}", addr, port);

        let default_port = std::env::var("DEFAULT_PORT").unwrap_or("2001".to_string());
        let default_addr = std::env::var("DEFAULT_ADDR").unwrap_or("0.0.0.0".to_string());

        if let Some(content) = get_content(&file_path).await {
            // using serde_json to parse the FTP Server config
            serde_json::from_str(&content).ok()
        } else {
            eprintln!("Error reading file");
            let default_server_config = FtpConfig {
                port: default_port.parse().unwrap(),
                addr: default_addr,
                admin: None,
                users: vec![
                    FtpUser {
                        username: "admin".to_string(),
                        password: "admin".to_string()
                    }
                ]
            };

            let content = serde_json::to_string(&default_server_config).unwrap();
            let mut file = tokio::fs::File::create(file_path.as_ref()).await.unwrap();
            file.write_all(content.as_bytes()).await.ok()?;

            Some(default_server_config)
        }
    }
}


#[test]
fn test_ftp_config() {
    let config = FtpConfig {
        port: 2001,
        addr: "0.0.0.0".to_string(),
        admin: Some(FtpUser {
            username: "admin".to_string(),
            password: "admin".to_string()
        }),
        users: vec![
            FtpUser {
                username: "user1".to_string(),
                password: "user1".to_string()
            },
            FtpUser {
                username: "user2".to_string(),
                password: "user2".to_string()
            }
        ]
    };
    assert_eq!(config.port, 2001);
    assert_eq!(config.addr, "0.0.0.0".to_string());
    if let Some(admin) = config.admin {
        assert_eq!(admin.username, "admin".to_string());
        assert_eq!(admin.password, "admin".to_string());
    } else {
        panic!("Admin user not found");
    }
    assert_eq!(config.users[0].username, "user1".to_string());
}

#[tokio::test]
async fn test_read_file() {
    let config = FtpConfig::new("ftp_config_test.json").await.expect("File not Found");
    println!("Config: {:?}", config);
    assert_eq!(config.port, 2001);
    assert_eq!(config.addr, "0.0.0.0".to_string());
    if let Some(admin) = &config.admin {
        assert_eq!(admin.username, "admin".to_string());
        assert_eq!(admin.password, "admin".to_string());
    } else {
        panic!("Admin user not found");
    }
    assert_eq!(config.users[0].username, "user".to_string());
    assert_eq!(config.users[0].password, "user".to_string());

}