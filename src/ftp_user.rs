use serde::{Deserialize, Serialize};

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct FtpUser {
    pub username: String,
    pub password: String
}