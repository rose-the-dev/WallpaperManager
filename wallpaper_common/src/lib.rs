use std::io::Read;
use std::os::unix::net::UnixStream;

pub const CONFIG_DIR: &str = ".config/wallpaper-engine";
pub const CONFIG_FILE: &str = "wallpaper.conf";
pub const WALLPAPER_DIR: &str = "wallpapers";

pub fn read_socket(sock: &mut UnixStream) -> Option<String> {
    let mut out = Vec::with_capacity(255);
    let y = sock.read(out.as_mut_slice()).unwrap();
    println!("{}", y);
    if y != 0 {
        return Some(String::from_utf8(out).unwrap());
    }
    None
}

pub struct SocketReader {
    buffer: Vec<u8>,
}

impl SocketReader {
    pub fn new(buffer_size: usize) -> SocketReader {
        SocketReader {
            buffer: vec![0; buffer_size]
        }
    }

    pub fn read_socket(&mut self, sock: &mut UnixStream) -> Option<String> {
        let y = sock.read(&mut self.buffer).unwrap();
        if y != 0 {
            self.buffer.truncate(y);
            return Some(String::from_utf8(self.buffer.clone()).unwrap().trim().to_string());
        }
        None
    }
}