use std::path::PathBuf;
use tokio::runtime::Handle;

pub struct Server {
    handle: Handle,
    root_dir_server: PathBuf,
}

impl Server {
    pub fn new(handle: Handle, root_dir_server: PathBuf) -> Self {
        Server {
            handle,
            root_dir_server,
        }
    }
}