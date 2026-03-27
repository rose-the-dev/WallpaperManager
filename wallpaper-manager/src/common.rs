use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output};
use eframe::egui::Image;
use serde::{Deserialize, Serialize};


pub const CONFIG_DIR: &str = ".config/wallpaper-engine";
pub const CONFIG_FILE: &str = "wallpaper.conf";
pub const WALLPAPER_DIR: &str = "wallpapers";

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    /// Whether to auto start wallpaperengine.
    //pub auto_start: bool,
    pub debugging: bool,
    /// Icon size
    pub icon_size: f32,
    pub silent: bool,
    pub no_audio_processing: bool,
    pub no_fullscreen_pause: bool,
    pub fps: Option<u16>,
    pub clamp: Clamp,
    pub wallpapers: HashMap<String, ScreenInfo>,
    pub wallpaper_engine_assets: Option<PathBuf>,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            //auto_start: false,
            debugging: false,
            icon_size: 200.0,
            silent: false,
            no_audio_processing: false,
            no_fullscreen_pause: false,
            fps: None,
            clamp: Clamp::Clamp,
            wallpapers: HashMap::new(),
            wallpaper_engine_assets: None,
        }
    }
}

pub enum ServiceType {Service, None}

#[derive(Serialize, Deserialize, Clone)]
pub struct ScreenInfo {
    pub id: String,
    pub scaling: Scaling,
}

#[derive(Clone)]
pub struct WallpaperInfo {
    /// Id of wallpaper (directory without full path of other files)
    pub id: String,
    /// Full path of wallpaper files with id.
    pub full_path: PathBuf,
    /// Full path to preview file.
    pub preview_file: String,
    pub project_file: String,
}

impl WallpaperInfo {
    pub fn new(path: PathBuf) -> Result<Self, std::io::Error> {
        let id = path.as_path().file_name().unwrap().to_str().unwrap().to_owned();
        let paths = std::fs::read_dir(Path::new(path.as_path()))?;
        for path2 in paths {
            let path2 = path2?.path();
            let name = path2.as_path().file_stem().unwrap();
            if name == "preview" {
                return Ok(Self {
                    id: id.clone(),
                    full_path: path.clone(),
                    preview_file: path2.as_path().to_str().unwrap().to_owned(),
                    project_file: format!("{}/{}", path.as_path().to_str().unwrap(), "project.json"),
                });
            }
        }
        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"))
    }
}

#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
pub enum Clamp { Clamp, Border, Repeat }

#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
pub enum Scaling { Stretch, Fit, Fill, Default }

#[derive(Clone)]
pub struct Wallpaper<'a> {
    pub wallpaper_info: WallpaperInfo,
    pub image: Option<Image<'a>>,
}

#[derive(Serialize, Deserialize)]
pub struct Schemecolor {
    pub order: u32,
    pub text: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub value: String,
}

#[derive(Serialize, Deserialize)]
pub struct Properties {
    pub schemecolor: Schemecolor,
}

#[derive(Serialize, Deserialize)]
pub struct General {
    pub properties: Properties,
}

#[derive(Serialize, Deserialize)]
pub struct WallpaperConfig {
    pub contentrating: String,
    pub description: String,
    pub file: String,
    pub general: General,
    pub preview: String,
    pub tags: Vec<String>,
    pub title: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub visibility: String,
}

pub fn read_config(config_file: String) -> Config {
    let config_data = std::fs::read_to_string(config_file).expect("Error reading config file");
    serde_json::from_str(config_data.as_str()).unwrap()
}

pub fn write_config(config_file: String, config: Config) {
    std::fs::write(config_file, serde_json::to_string(&config).expect("Error serializing config file")).expect("Error writing config file");
}

pub fn start_wallpaper_process(config: Config) -> Child {
    let mut proc = Command::new("linux-wallpaperengine");
    if config.wallpaper_engine_assets.is_some() {
        proc.arg("--assets-dir").arg(config.wallpaper_engine_assets.as_ref().unwrap());
    }
    if config.no_fullscreen_pause {
        proc.arg("--no-fullscreen-pause");
    }
    if config.fps.is_some() {
        proc.arg("--fps").arg(&config.fps.unwrap().to_string());
    }
    if config.silent {
        proc.arg("--silent");
    }
    if config.no_audio_processing {
        proc.arg("--no-audio-processing");
    }
    for (mon, wp) in config.wallpapers.iter() {
        proc.arg("--screen-root").arg(mon).arg("--bg").arg(get_wallpaper_dir(Some(wp.id.clone())));
        proc.arg("--scaling").arg(format!("{:?}", wp.scaling).to_lowercase());
    }
    proc.arg("--clamp").arg(format!("{:?}", config.clamp).to_lowercase());
    println!("{:?}", proc.get_args());
    proc.spawn().expect("Failed to start wallpaper process.")
}

//pub fn kill_wallpaper(wallpaper_process: Option<&mut Child>) {
pub fn kill_wallpaper() {
    Command::new("pkill").arg("-f").arg("linux-wallpaperengine").output().expect("Failed to kill wallpaper process.");
    //if wallpaper_process.is_some() {
    //    wallpaper_process.unwrap().kill().expect("Unable to kill child process.");
    //}
}

pub fn restart_wallpaper_service(service_type: ServiceType) -> std::io::Result<Output> {
    match service_type {
        ServiceType::Service => Command::new("systemctl").arg("--user").arg("restart").arg("wallpaper-engine.service").output(),
        ServiceType::None => {
            //Command::new("pkill").arg("-f").arg("linux-wallpaperengine").output().expect("Failed to kill wallpaper process.");
            //start_wallpaper_process(read_config(CONFIG_FILE.to_string()));
            panic!("Service only for now.")
        },
    }
}

pub fn get_wallpaper_dir(wp_dir: Option<String>) -> String {
    if wp_dir.is_some() {
        format!("{0}/{1}/{2}/{3}", std::env::home_dir().expect("ERROR1").to_str().expect("ERROR2"), CONFIG_DIR, WALLPAPER_DIR, wp_dir.unwrap())
    }
    else {
        format!("{0}/{1}/{2}", std::env::home_dir().expect("ERROR1").to_str().expect("ERROR2"), CONFIG_DIR, WALLPAPER_DIR)
    }
}

pub fn get_wallpapers() -> Result<Vec<WallpaperInfo>, std::io::Error> {
    let path = get_wallpaper_dir(None);
    if (std::fs::exists(path.clone())).is_ok() {
        std::fs::create_dir_all(path.clone()).expect("Unable to create wallpaper dir");
    }
    let paths = std::fs::read_dir(path)?;
    let mut result: Vec<WallpaperInfo> = Vec::new();
    for path in paths {
        let path = path?.path();
        result.push(WallpaperInfo::new(path)?);
    }
    Ok(result)
}

pub fn get_wallpaper_preview(wallpaper_dir: String) -> Result<String, std::io::Error> {
    let paths = std::fs::read_dir(wallpaper_dir);
    if paths.is_ok() {
        for path2 in paths? {
            let path2 = path2?.path();
            let name = path2.as_path().file_stem().unwrap();
            if name == "preview" {
                return Ok(path2.as_path().to_str().unwrap().to_owned());
            }
        }
        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"))
    }
    else {
        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"))
    }
}

pub fn get_column_count(window_width: f32, icon_width: f32) -> i32 {
    (window_width / icon_width) as i32
}