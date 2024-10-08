use std::fs::Metadata;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use bytes::BytesMut;
use cfg_if::cfg_if;
use time::OffsetDateTime;
use tokio::fs::{metadata, File};
use tokio::io::AsyncReadExt;

pub const CONFIG_FILE: &str = "config.json";

#[macro_use]
extern crate cfg_if;
extern crate time;

cfg_if! {
    if #[cfg(windows)] {
        pub fn get_file_info(meta: &Metadata) -> (OffsetDateTime, u64) {
            use std::os::windows::prelude::*;
            (OffsetDateTime::from_unix_timestamp(meta.last_write_time() as i64).unwrap(), meta.file_size())
        }
    } else {
        pub fn get_file_info(meta: &Metadata) -> (OffsetDateTime, u64) {
            use std::os::unix::prelude::*;
            (OffsetDateTime::from_unix_timestamp(meta.modified().unwrap().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64).unwrap(), meta.size())
        }
    }
}



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

pub fn prefix_slash(path: &mut PathBuf) {
    if !path.is_absolute() {
        *path = Path::new("/").join(&path);
    }
}

/// Function to add the file details metadata to the output buffer.
/// If an error occurs when we try to get file's information, we just return and don't send its info.
pub async fn add_file_info(path: PathBuf, out: &mut Vec<u8>) {
    let extra = if path.is_dir() { "/" } else { "" };
    let is_dir = if path.is_dir() { "d" } else { "-" };

    // return if we can't get the metadata
    let metadata = match metadata(&path).await {
        Ok(meta_data) => meta_data,
        _ => return
    };

    let (time, file_size) = get_file_info(&metadata);

    let path = match path.to_str() {
        Some(path) => match path.split("/").last() {
            Some(path) => path,
            None => return
        },
        _ => return
    };




}