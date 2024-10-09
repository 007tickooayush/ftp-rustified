
use std::fs::Metadata;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use bytes::BytesMut;
use time::OffsetDateTime;
use tokio::fs::{metadata, File};
use tokio::io::AsyncReadExt;

pub const CONFIG_FILE: &str = "config.json";

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

// Rest of the code...
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

    // get the file permissions
    let rights = if metadata.permissions().readonly() {
        "r--r--r--"
    } else {
        // getting the formatted permissions from the metadata of the file
        &get_permissions(&metadata)
    };

    let file_info_str = format!(
        "{is_dir}{rights} {links} {owner} {group} {size} {month} {day} {hour}:{min} {path}{extra}\r\n",
        is_dir = is_dir,
        rights = rights,
        links = 1,
        owner = "ftp-rustified",
        group = "anonymous",
        size = file_size,
        month = time.month(),
        day = time.day(),
        hour = time.hour(),
        min = time.minute(),
        path = path,
        extra = extra
    );

    out.extend(file_info_str.as_bytes());

    println!("\t\tFILE INFO ==> {}",&file_info_str);
}

pub fn get_permissions(metadata: &Metadata) -> String {
    use std::os::unix::prelude::*;
    let permissions = metadata.permissions();
    let mode = permissions.mode();

    format!(
        "{}{}{}{}{}{}{}{}{}",
        if mode & 0o400 != 0 { 'r' } else { '-' },
        if mode & 0o200 != 0 { 'w' } else { '-' },
        if mode & 0o100 != 0 { 'x' } else { '-' },
        if mode & 0o040 != 0 { 'r' } else { '-' },
        if mode & 0o020 != 0 { 'w' } else { '-' },
        if mode & 0o010 != 0 { 'x' } else { '-' },
        if mode & 0o004 != 0 { 'r' } else { '-' },
        if mode & 0o002 != 0 { 'w' } else { '-' },
        if mode & 0o001 != 0 { 'x' } else { '-' },
    )
}