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

