// use std::io::Write;
use crate::ftp_response_code::ResponseCode;

#[derive(Debug)]
pub struct Response {
    pub code: ResponseCode,
    pub message: String
}

impl Response {
    pub fn new(code: ResponseCode, message: &str) -> Self {
        println!("\t\tCreating new response: \"{:?}\"",Response {
            code: code.clone(), // code not getting converted to number
            message: message.to_string()
        });
        Response {
            code,
            message: message.to_string()
        }
    }

    pub fn to_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.code.to_bytes());
        bytes.extend_from_slice(self.message.as_bytes());

        println!("\t\tRESPONSE BYTES ==> {:?}",String::from_utf8_lossy(&bytes));

        bytes
    }

    pub fn to_string(self) -> String {
        format!("{:?} {}",self.code as u32,self.message)
    }
}

#[test]
fn test_response() {
    let response = Response::new(ResponseCode::Ok, "Hello, World!");
    assert_eq!(response.to_string(), "200 Hello, World!");
}