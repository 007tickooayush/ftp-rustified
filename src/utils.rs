use std::path::Path;
use bytes::BytesMut;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

pub async fn get_content<P: AsRef<Path>>(file_path:&P) -> Option<String> {
    let mut file = File::open(file_path).await.ok()?;
    let mut content = String::new();
    file.read_to_string(&mut content).await.ok()?;
    Some(content)
}

pub fn find_crlf(buf: &mut BytesMut) -> Option<usize> {
    buf.windows(2).position(|bytes| bytes == b"\r\n")
}

pub fn bytes_to_uppercase(data: &mut [u8]) {
    for byte in data {
        if *byte >= b'a' as u8 && *byte <= b'z' as u8 {
            *byte -= 32;
        }
    }
}