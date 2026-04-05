use std::collections::HashMap;
use std::path::{Path, PathBuf};
use eframe::egui::Image;
use serde::{Deserialize, Serialize};
use wallpaper_common::{CONFIG_DIR, WALLPAPER_DIR};


//#[derive(Serialize, Deserialize, Clone)]
//pub struct Config {
//    /// Whether to auto start wallpaperengine.
//    //pub auto_start: bool,
//    pub debugging: bool,
//    /// Icon size
//    pub icon_size: f32,
//    pub silent: bool,
//    pub no_audio_processing: bool,
//    pub no_fullscreen_pause: bool,
//    pub fps: Option<u16>,
//    pub clamp: Clamp,
//    pub wallpapers: HashMap<String, ScreenInfo>,
//    pub wallpaper_engine_assets: Option<PathBuf>,
//}
//impl Default for Config {
//    fn default() -> Self {
//        Self {
//            //auto_start: false,
//            debugging: false,
//            icon_size: 200.0,
//            silent: false,
//            no_audio_processing: false,
//            no_fullscreen_pause: false,
//            fps: None,
//            clamp: Clamp::Clamp,
//            wallpapers: HashMap::new(),
//            wallpaper_engine_assets: None,
//        }
//    }
//}



//#[derive(Clone)]
//pub struct WallpaperInfo {
//    /// Id of wallpaper (directory without full path of other files)
//    pub id: String,
//    /// Full path of wallpaper files with id.
//    pub full_path: PathBuf,
//    /// Full path to preview file.
//    pub preview_file: String,
//    pub project_file: String,
//}

//impl WallpaperInfo {
//    pub fn new(path: PathBuf) -> Result<Self, std::io::Error> {
//        let id = path.as_path().file_name().unwrap().to_str().unwrap().to_owned();
//        let paths = std::fs::read_dir(Path::new(path.as_path()))?;
//        for path2 in paths {
//            let path2 = path2?.path();
//            let name = path2.as_path().file_stem().unwrap();
//            if name == "preview" {
//                return Ok(Self {
//                    id: id.clone(),
//                    full_path: path.clone(),
//                    preview_file: path2.as_path().to_str().unwrap().to_owned(),
//                    project_file: format!("{}/{}", path.as_path().to_str().unwrap(), "project.json"),
//                });
//            }
//        }
//        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"))
//    }
//}

//#[derive(Serialize, Deserialize, Clone)]
//pub struct ScreenInfo {
//    pub id: String,
//    pub scaling: Scaling,
//}

//#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
//pub enum Clamp { Clamp, Border, Repeat }

//#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
//pub enum Scaling { Stretch, Fit, Fill, Default }

#[derive(Clone)]
pub struct Wallpaper<'a> {
    pub wallpaper_info: wallpaper_common::wallpaper::WallpaperInfo,
    pub image: Option<Image<'a>>,
}

//pub fn read_config(config_file: String) -> Config {
//    let config_data = std::fs::read_to_string(config_file).expect("Error reading config file");
//    serde_json::from_str(config_data.as_str()).unwrap()
//}

//pub fn write_config(config_file: String, config: Config) {
//    std::fs::write(config_file, serde_json::to_string(&config).expect("Error serializing config file")).expect("Error writing config file");
//}

//pub fn get_wallpaper_dir(wp_dir: Option<String>) -> String {
//    if wp_dir.is_some() {
//        format!("{0}/{1}/{2}/{3}", std::env::home_dir().expect("ERROR1").to_str().expect("ERROR2"), CONFIG_DIR, WALLPAPER_DIR, wp_dir.unwrap())
//    }
//    else {
//        format!("{0}/{1}/{2}", std::env::home_dir().expect("ERROR1").to_str().expect("ERROR2"), CONFIG_DIR, WALLPAPER_DIR)
//    }
//}

//pub fn get_wallpapers() -> Result<Vec<WallpaperInfo>, std::io::Error> {
//    let path = get_wallpaper_dir(None);
//    if (std::fs::exists(path.clone())).is_ok() {
//        std::fs::create_dir_all(path.clone()).expect("Unable to create wallpaper dir");
//    }
//    let paths = std::fs::read_dir(path)?;
//    let mut result: Vec<WallpaperInfo> = Vec::new();
//    for path in paths {
//        let path = path?.path();
//        result.push(WallpaperInfo::new(path)?);
//    }
//    Ok(result)
//}

//pub fn get_wallpaper_preview(wallpaper_dir: String) -> Result<String, std::io::Error> {
//    let paths = std::fs::read_dir(wallpaper_dir);
//    if paths.is_ok() {
//        for path2 in paths? {
//            let path2 = path2?.path();
//            let name = path2.as_path().file_stem().unwrap();
//            if name == "preview" {
//                return Ok(path2.as_path().to_str().unwrap().to_owned());
//            }
//        }
//        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"))
//    }
//    else {
//        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"))
//    }
//}