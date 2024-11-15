use std::env;
use std::ffi::OsString;
use std::fs::Metadata;
use std::os::unix::prelude::PermissionsExt;
use std::path::{Component, Path, PathBuf};
use std::time::UNIX_EPOCH;
use bytes::BytesMut;
use chrono::{DateTime, Datelike, Local};
use time::OffsetDateTime;
use tokio::fs::{metadata, read_dir, DirEntry, File};
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::error::FtpError;

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
        "{is_dir}{rights} {links} {owner} {group} {size} {month} {day} {hour}:{min} {path}\r\n", //{extra}
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
        // extra = extra
    );

    out.extend(file_info_str.as_bytes());

    // println!("\t\tFILE INFO ==> {}",&file_info_str);
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

pub fn invalid_path(path: &Path) -> bool {
    for component in path.components() {
        if let Component::ParentDir = component {
            return true;
        }
    }
    false
}

pub fn get_filename(path: PathBuf) -> Option<OsString> {
    path.file_name().map(|p| p.to_os_string())
}

pub async fn create_root_dir(path: &str) -> io::Result<()> {
    if !Path::new(path).exists() {
        tokio::fs::create_dir_all(path).await?;
        let mut permissions = std::fs::metadata(path)?.permissions();
        permissions.set_mode(0o755);
        tokio::fs::set_permissions(path, permissions).await?;
    }
    Ok(())
}

pub fn get_current_dir() -> PathBuf {
    // env::current_dir().unwrap_or_else(|_| FtpError::Msg("Unable to get current directory\r\n".to_string()))
    env::current_dir().unwrap_or_else(|_| {
        FtpError::Msg("Unable to get current directory\r\n".to_string());
        PathBuf::new()
    })
}

pub fn get_first_word_and_rest(input: &str) -> Option<(&str, &str)> {
    for (i,c) in input.chars().enumerate() {
        if c == ' ' {
            return Some((&input[..i], &input[i+1..]));
        }
    }
    Some((input, ""))
}

#[tokio::test]
async fn test_add_file_info() {
    let mut out  = Vec::new();
    let path = PathBuf::from("/home/hellsent/HRs/RR/ftp-rustified/ROOT/dir1");

    add_file_info(path, &mut out).await;

    println!("OUT ==> {:?}",String::from_utf8_lossy(&out));

}

#[tokio::test]
async fn test_multiple_file() {
    let mut out = Vec::new();
    let path = PathBuf::from("/home/hellsent/HRs/RR/ftp-rustified/ROOT");

    if path.is_dir() {
        if let Ok(mut read_dir) = read_dir(path).await {
            while let Some(entry) = read_dir.next_entry().await.unwrap() {
                add_file_info(entry.path(), &mut out).await;
                println!("@@OUT ==> {:?}",String::from_utf8_lossy(&out));
            }
        }
    }
}

#[tokio::test]
async fn test__read_path_2() {
    let path = PathBuf::from("/home/hellsent/HRs/RR/ftp-rustified/ROOT");
    // let mut out = Vec::new();

    if path.is_dir() {
        if let Ok(mut read_dir) = read_dir(path).await {
            while let Some(entry) = read_dir.next_entry().await.unwrap() {
                let data = get_file_info_2(entry).await;
                println!("@@FFT ==> {:?}",data);
            }
        }
    }
}

/// only for testing purposes, not realtime function for getting data from the server
async fn get_file_info_2(entry: DirEntry) -> String {
    let metadata = entry.metadata().await.unwrap();
    let content_type = if metadata.is_dir() { 'd' } else { '-' };

    let permissions = metadata.permissions();
    let mode = permissions.mode();

    let perm_str = format!(
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
    );

    let size = if metadata.is_dir() { 4096 } else { metadata.len() };

    let modified: DateTime<Local> = metadata.modified().unwrap().into();
    let date_str = if modified.year() == Local::now().year() {
        modified.format("%b %d %H:%M")
    } else {
        modified.format("%b %d  %Y")
    };

    let filename = entry.file_name().to_string_lossy().into_owned();
    // Ok(
        format!(
            "{}{} 1 ftp-rustified anonymous {:8} {} {}\r\n",
            content_type,
            perm_str,
            size,
            date_str,
            filename,
            // if metadata.is_dir() { "/" } else { "" }
        )
    // )
}

#[tokio::test]
async fn test_file_handling() {
    match File::create_new("/home/hellsent/HRs/RR/ftp-rustified/ROOT/test.txt").await {
        Ok(mut file ) => {
            let mut data = Vec::new();
            data.extend_from_slice(b"Hello, World!");
            file.write_all(&data).await.unwrap();
        },
        Err(err) => {

            eprintln!("Error creating file: {}", err);
        }
    }
}