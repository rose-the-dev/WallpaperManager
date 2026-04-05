use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::{wallpaper, CONFIG_DIR, WALLPAPER_DIR};

#[derive(Clone)]
pub struct WallpaperInfo {
    /// Id of wallpaper (directory without full path of other files)
    pub id: String,
    /// Full path of wallpaper files with id.
    pub full_path: PathBuf,
    pub project_file: String,
    pub data: ProjectInfo,
}

impl WallpaperInfo {
    pub fn from_path(path: PathBuf) -> Result<Self, std::io::Error> {
        let id = path.as_path().file_name().unwrap().to_str().unwrap().to_owned();
        let _paths = std::fs::read_dir(Path::new(path.as_path()))?;
        let file = format!("{}/{}", path.as_path().to_str().unwrap(), "project.json");
        if std::fs::exists(file.clone())? {
            Ok(Self {
                id: id.clone(),
                full_path: path.clone(),
                data: ProjectInfo::from_file(file.as_str())?,
                project_file: file,
            })
        }
        else {
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"))
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ProjectInfo {
    pub content_rating: String,
    pub file: String,
    pub preview: String,
    pub tags: Vec<String>,
    pub title: String,

    pub background_type: String,
    pub version: u8,
}

impl ProjectInfo {
    pub fn from_file(file: &str) -> serde_json::error::Result<ProjectInfo> {
        let config_data = std::fs::read_to_string(file).expect("Error reading config file");
        serde_json::from_str(config_data.as_str())
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