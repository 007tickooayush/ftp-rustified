use std::path::{Path, PathBuf};
use dotenv::dotenv;
use crate::client_handler::client_handler::handle_client;

#[derive(Debug, Clone)]
pub struct Server {
    addr: String,
    buf_size: u128,
    root_dir: PathBuf,
    curr_dir:PathBuf
}

impl Server {
    pub fn new(addr: String, buf_size:u128,) -> Self {
        dotenv().ok();
        let def_root_dir = PathBuf::from(std::env::var("ROOT_DIR").unwrap_or("PUBLIC".to_string()));
        Self {
            addr,
            buf_size,
            root_dir: def_root_dir.clone(),
            curr_dir: def_root_dir
        }
    }

    pub fn get_root_dir(&self) -> PathBuf {
        self.root_dir.clone()
    }
    pub fn get_curr_dir(&self) -> PathBuf {
        self.curr_dir.clone()
    }

    pub async fn run(&mut self) {
        let server = tokio::net::TcpListener::bind(&self.addr).await.unwrap();
        println!("Server is running on {}", self.addr);

        let size = self.buf_size;

        loop {
            let (socket, _) = server.accept().await.unwrap();
            let mut server_clone = self.clone();
            tokio::spawn(async move {
                handle_client(&mut server_clone,socket,size).await.unwrap();
            });
        }

    }

    pub async fn change_directory(&mut self, path: &str) -> Result<String, std::io::Error> {
        let new_path = if path.starts_with("/") {
            // println!("IF Root dir: {:?}", self.root_dir);
            // println!("IF Path: {:?}", Path::new(path).file_name().unwrap());
            self.root_dir.join(Path::new(path).file_name().unwrap())
        } else {
            println!("EL Root dir: {:?}", self.root_dir);
            self.curr_dir.join(path)
        };


        if let Ok(canonical_path) = tokio::fs::canonicalize(&new_path).await {
            println!("Canonical path: {:?}", canonical_path);
            //canonical_path.starts_with(&self.root_dir) && tokio::fs::metadata(&canonical_path).await?.is_dir()

            // todo!("PROVIDE SECURE METHOD FOR CHANGING DIRECTORY")
            if tokio::fs::metadata(&canonical_path).await?.is_dir() {
                self.curr_dir = canonical_path.clone();
                Ok(format!(
                    "250 Directory changed to \"{}\"\r\n",
                    canonical_path.to_string_lossy()
                ))
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "Directory unavailable or outside of root directory"
                ))
            }
        } else {
            println!("New path: {:?}", new_path);
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Directory not found"))
        }
    }
}