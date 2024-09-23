use serde::{Deserialize, Serialize};
use crate::ftp_user::FtpUser;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FtpConfig {
    pub port: u16,
    pub host: String,
    pub admin: Option<FtpUser>,
    pub users: Vec<FtpUser>
}