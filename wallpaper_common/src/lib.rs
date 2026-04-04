pub mod wallpaper;

use std::collections::HashMap;
use std::io::{ErrorKind, Read, Write};
use std::os::unix::net::UnixStream;
use serde::{Deserialize, Serialize};

pub const CONFIG_DIR: &str = ".config/wallpaper-engine";
pub const CONFIG_FILE: &str = "wallpaper.conf";
pub const WALLPAPER_DIR: &str = "wallpapers";
pub const WALLPAPER_ENGINE_SOCKET: &str = "/tmp/wallpaper-engine.sock";

pub struct Ipc {
    buffer: Vec<u8>,
    stream: UnixStream,
}

impl Ipc {
    pub fn connect() -> Result<Self, std::io::Error> {
        let stream = UnixStream::connect(WALLPAPER_ENGINE_SOCKET);
        if stream.is_err() {
            return Err(stream.unwrap_err());
        }
        Ok(Self {
            buffer: Vec::new(),
            stream: stream?,
        })
    }


    pub fn send_change_wallpaper(&mut self, output: String, wallpaper: String) -> Result<String, std::io::Error> {
        let v = format!("set:{},{}", output, wallpaper);
        self.stream.write(v.as_bytes())?;
        self.read_inner()
    }

    pub fn send_list_outputs(&mut self) -> Result<String, std::io::Error> {
        self.stream.write(b"list-outputs>")?;
        self.read_inner()
    }

    pub fn send_list_wallpapers(&mut self) -> Result<String, std::io::Error> {
        self.stream.write(b"list-wallpapers>")?;
        self.read_inner()
    }

    /// Sets an option, can be anything in the config file except monitor values.
    pub fn send_option(&mut self, setting: String, value: String) -> Result<String, std::io::Error> {
        let v = format!("option:{},{}", setting, value);
        self.stream.write(v.as_bytes())?;
        self.read_inner()
    }

    fn read_inner(&mut self) -> Result<String, std::io::Error> {
        let ret = self.stream.read(&mut self.buffer)?;
        if ret != 0 {
            let mut buf = self.buffer.clone();
            buf.truncate(ret);
            Ok(String::from_utf8(buf).unwrap().trim().to_string())
        }
        else{
            Err(std::io::Error::new(ErrorKind::Other, "Couldn't receive message."))
        }
    }
}

pub fn read_socket(sock: &mut UnixStream) -> Option<String> {
    let mut out = vec![0; 255];
    let y = sock.read(out.as_mut_slice()).unwrap();
    println!("{}", y);
    if y != 0 {
        return Some(String::from_utf8(out).unwrap());
    }
    None
}


pub struct Config {
    /// All config values without monitors.
    values: HashMap<String, String>,
    /// All monitor entries
    monitors: HashMap<String, String>,
}
impl Config {
    pub fn from_file(file: &str) -> Result<Config, std::io::Error> {
        let mut buf = String::new();
        let mut file = std::fs::File::open(file)?;
        let res = file.read_to_string(&mut buf);
        match res {
            Ok(data) => {
                let mut s = Self {
                    values: HashMap::new(),
                    monitors: HashMap::new(),
                };
                let mut line = 0;
                for val in buf.lines() {
                    if val.starts_with("#") { // Comment
                        continue;
                    }
                    let val = val.to_lowercase();
                    let split: Vec<&str> = val.split(":").collect();
                    if split[0] == "monitor" {
                        if split.len() != 3 {
                            return Err(std::io::Error::new(ErrorKind::Other, format!("Invalid monitor config at line {}", line)));
                        }
                        s.monitors.insert(split[1].to_string(), split[2].to_string());
                    }
                    else {
                        s.values.insert(split[0].to_string(), split[1].to_string());
                    }
                    line += 1;
                }
                Ok(s)
            }
            Err(e) => {
                Err(e)
            }
        }
    }

    pub fn save(&self, file: &str) -> Result<(), std::io::Error> {
        let mut file = std::fs::File::open(file)?;
        for (key, value) in &self.values {
            file.write(format!("{}:{}\n", key, value).as_bytes())?;
        }
        for (key, value) in &self.monitors {
            file.write(format!("monitor:{}:{}\n", key, value).as_bytes())?;
        }
        Ok(())
    }
}

fn get_wallpaper_dir(wp_dir: Option<String>) -> String {
    if wp_dir.is_some() {
        format!("{0}/{1}/{2}/{3}", std::env::home_dir().expect("ERROR1").to_str().expect("ERROR2"), CONFIG_DIR, WALLPAPER_DIR, wp_dir.unwrap())
    }
    else {
        format!("{0}/{1}/{2}", std::env::home_dir().expect("ERROR1").to_str().expect("ERROR2"), CONFIG_DIR, WALLPAPER_DIR)
    }
}

pub fn get_wallpapers() -> Result<Vec<wallpaper::WallpaperInfo>, std::io::Error> {
    let path = get_wallpaper_dir(None);
    if (std::fs::exists(path.clone())).is_ok() {
        std::fs::create_dir_all(path.clone()).expect("Unable to create wallpaper dir");
    }
    let paths = std::fs::read_dir(path)?;
    let mut result: Vec<wallpaper::WallpaperInfo> = Vec::new();
    for path in paths {
        let path = path?.path();
        result.push(wallpaper::WallpaperInfo::from_path(path)?);
    }
    Ok(result)
}