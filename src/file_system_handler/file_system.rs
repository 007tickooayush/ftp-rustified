use std::env::current_dir;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use chrono::{DateTime, Datelike, Local};
use tokio::fs::DirEntry;
use crate::tcp_handler::server::Server;

type CmdResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn get_root_dir(server: &Server) -> PathBuf {
    if server.get_curr_dir().ne(&server.get_root_dir()) {
        server.get_curr_dir().clone()
    } else {
        let mut current_dir = current_dir().unwrap();
        current_dir.push("PUBLIC");
        current_dir
    }
    // let mut current_dir = current_dir().unwrap();
    // current_dir.push("PUBLIC");
    // current_dir
}

pub async fn get_file_info(entry: DirEntry) -> CmdResult<String> {
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
    Ok(
        format!(
            "{}{} 1 owner group {:8} {} {}\r\n",
            content_type,
            perm_str,
            size,
            date_str,
            filename,
            // if metadata.is_dir() { "/" } else { "" }
        )
    )
}

pub async fn list_dir(path: &str) -> CmdResult<String> {
    let mut response = String::new();
    response.push_str("150 Opening ASCII mode data connection for file list.\r\n");

    let mut entries = tokio::fs::read_dir(path).await.unwrap();

    while let Some(entry) = entries.next_entry().await.unwrap() {
        // called for each traversal of the directory
        let info = get_file_info(entry).await?;
        response.push_str(&info);
        // response.push_str("\r\n");
    }
    response.push_str("226 Transfer complete.\r\n");
    Ok(response)
}

pub async fn handle_cwd_command(server: &mut Server, path: &str) -> String {
    server.change_directory(path).await.unwrap_or_else(|err| match err.kind() {
        std::io::ErrorKind::PermissionDenied => String::from("550 Permission denied.\r\n"),
        std::io::ErrorKind::NotFound => String::from("550 Directory not found.\r\n"),
        _ => String::from("550 Requested action not taken.\r\n"),
    })
}