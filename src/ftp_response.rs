pub struct Response {
    pub code: u32,
    pub message: String
}

impl Response {
    pub fn new(code: u32, message: String) -> Self {
        Response {
            code,
            message
        }
    }
}

