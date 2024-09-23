use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FtpConfig {
    pub port: u16,
    pub host: String,
    pub admin: Option<User>,
    pub users: Vec<User>
}