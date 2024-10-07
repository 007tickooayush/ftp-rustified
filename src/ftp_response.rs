use std::io::Write;
use crate::ftp_responce_code::ResponseCode;

pub struct Response {
    pub code: ResponseCode,
    pub message: String
}

impl Response {
    pub fn new(code: ResponseCode, message: &str) -> Self {
        Response {
            code,
            message: message.to_string()
        }
    }

    pub fn to_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.code.to_bytes());
        bytes.extend_from_slice(self.message.as_bytes());
        bytes
    }
}