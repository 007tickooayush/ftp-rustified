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
}

impl Into<Vec<u8>> for Response {
    fn into(self) -> Vec<u8> {
        let mut buffer: Vec<u8> = vec![];

        if self.message.is_empty() {
            write!(buffer, "{}\r\n",self.code as u32).unwrap();
        } else {
            write!(buffer, "{} {}\r\n", self.code as u32, self.message).unwrap();
        }

        buffer
    }
}
