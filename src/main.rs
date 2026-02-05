use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use eframe::{egui};
use eframe::egui::{include_image, Image, ImageButton, Vec2};
use egui_modal::Modal;
use serde::{Deserialize, Serialize};
//use serde_json::{Result, Value};

struct Global;
impl Global {
    const CONFIG_DIR: &str = ".config/wallpaper-gui";
    const CONFIG_FILE: &str = "wallpaper.conf";
    const WALLPAPER_DIR: &str = "wallpapers";
}

#[derive(Serialize, Deserialize, Clone)]
struct Config {
    /// Whether to auto start wallpaperengine.
    auto_start: bool,
    icon_size: f32,
    /// Current screen configuration, DP-2, HDMI-A-1, eDP-1, etc
    screens: Vec<String>,
    //wallpaper_engine_dir: Option<String>,
    //manager_dir: Option<String>,
    silent: bool,
    no_audio_processing: bool,
    wallpapers: HashMap<String, String>, // screen name, wallpaper id
    wallpaper_engine_assets: Option<PathBuf>,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            auto_start: false,
            icon_size: 200.0,
            screens: Vec::new(),
            //wallpaper_engine_dir: None,
            //manager_dir: ".config/wallpaper-".to_owned(),
            silent: false,
            no_audio_processing: false,
            wallpapers: HashMap::new(),
            wallpaper_engine_assets: None,
        }
    }
}
#[derive(Clone)]
struct WallpaperFileInfo {
    /// Id of wallpaper (directory without full path of other files)
    id: String,
    /// Full path of wallpaper files with id.
    full_path: PathBuf,
    /// Full path to preview file.
    preview_file: String,
}
impl WallpaperFileInfo {
    pub fn new(path: PathBuf) -> Result<Self, std::io::Error> {
        let id = path.as_path().file_name().unwrap().to_str().unwrap().to_owned();
        let paths = std::fs::read_dir(Path::new(path.as_path()))?;
        for path2 in paths {
            let path2 = path2?.path();
            //let file = path2.as_path().file_name().unwrap();
            let name = path2.as_path().file_stem().unwrap();
            if name == "preview" {
                return Ok(Self {
                    id: id.clone(),
                    full_path: path.clone(),
                    preview_file: path2.as_path().to_str().unwrap().to_owned(),
                });
            }
        }
        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"))
    }
}

#[derive(Clone)]
struct Wallpaper<'a> {
    wallpaper_info: WallpaperFileInfo,
    image: Option<Image<'a>>,
}

fn main() -> eframe::Result {
    env_logger::init();
    Command::new("pkill").arg("-f").arg("linux-wallpaperengine").output().expect("Failed to kill wallpaper process.");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_app_id("wallpaper-manager").with_inner_size([800.0, 500.0]),
        ..Default::default()
    };
    eframe::run_native(
        "WallpaperManager",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::<MainWindow>::default())
        }),
    )
}

fn read_config(config_file: String) -> Config {
    let config_data = std::fs::read_to_string(config_file).expect("Error reading config file");
    serde_json::from_str(config_data.as_str()).unwrap()
}

fn write_config(config_file: String, config: Config) {
    std::fs::write(config_file, serde_json::to_string(&config).expect("Error serializing config file")).expect("Error writing config file");
}

fn start_wallpaper_process(config: Config) -> Child {
    let mut proc = Command::new("linux-wallpaperengine");
    proc.arg("--assets-dir").arg("/home/rose/.steam/steam/steamapps/common/wallpaper_engine/assets");
    //proc.arg("--assets-dir").arg(format!("{0}/{1}/{2}", std::env::home_dir().expect("ERROR1").to_str().expect("ERROR2"), Global::CONFIG_DIR, Global::WALLPAPER_DIR));
    // TODO: include --fps 15
    if config.silent {
        proc.arg("--silent");
    }
    if config.no_audio_processing {
        proc.arg("--no-audio-processing");
    }
    for (mon, wp) in config.wallpapers.iter() {
        // TODO: Include other values, like '--scaling' and '--clamp'.
        proc.arg("--screen-root").arg(mon).arg("--bg").arg(wp);
    }
    proc.spawn().expect("Failed to start wallpaper process.")
}

fn get_wallpaper_dir(wp_dir: Option<String>) -> String {
    if wp_dir.is_some() {
        format!("{0}/{1}/{2}/{3}", std::env::home_dir().expect("ERROR1").to_str().expect("ERROR2"), Global::CONFIG_DIR, Global::WALLPAPER_DIR, wp_dir.unwrap())
    }
    else {
        format!("{0}/{1}/{2}", std::env::home_dir().expect("ERROR1").to_str().expect("ERROR2"), Global::CONFIG_DIR, Global::WALLPAPER_DIR)
    }
}

fn get_wallpapers() -> Result<Vec<WallpaperFileInfo>, std::io::Error> {
    let path = format!("{0}/{1}/{2}", std::env::home_dir().expect("ERROR1").to_str().expect("ERROR2"), Global::CONFIG_DIR, Global::WALLPAPER_DIR);
    if (std::fs::exists(path.clone())).is_ok() {
        std::fs::create_dir_all(path.clone()).expect("Unable to create wallpaper dir");
    }
    let paths = std::fs::read_dir(path)?;
    let mut result: Vec<WallpaperFileInfo> = Vec::new();
    for path in paths {
        let path = path?.path();
        result.push(WallpaperFileInfo::new(path)?);
    }
    //if result.is_empty() {
    //    return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Directory empty").into());
    //}
    Ok(result)
}

fn get_wallpaper_previews() -> Result<Vec<String>, std::io::Error> {
    //let paths = std::fs::read_dir(Path::new(self.config.wallpaper_engine_dir.clone().unwrap().as_str()))?; // WRONG THIS NEEDS TO BE CHANGED.
    let path = format!("{0}/{1}/{2}", std::env::home_dir().expect("ERROR1").to_str().expect("ERROR2"), Global::CONFIG_DIR, Global::WALLPAPER_DIR);
    if (std::fs::exists(path.clone())).is_ok() {
        std::fs::create_dir_all(path.clone()).expect("Unable to create wallpaper dir");
    }
    let paths = std::fs::read_dir(path).unwrap(); // WRONG THIS NEEDS TO BE CHANGED.
    let mut result: Vec<String> = Vec::new();
    for path in paths {
        let path = path.unwrap().path();
        result.push(path.file_name().unwrap().to_str().unwrap().to_owned());
    }
    if result.is_empty() {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Directory empty").into());
    }
    Ok(result)
}

fn get_wallpaper_preview(wallpaper_dir: String) -> Result<String, std::io::Error> {
    let paths = std::fs::read_dir(wallpaper_dir);
    if paths.is_ok() {
        for path2 in paths.unwrap() {
            let path2 = path2?.path();
            //let file = path2.as_path().file_name().unwrap();
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

fn get_column_count(window_width: f32, icon_width: f32) -> i32 {
    (window_width / icon_width) as i32
}

struct MainWindow<'a> {
    config: Config,
    //icon_size: f32,
    wallpaper: Option<WallpaperFileInfo>,
    //preview_images: HashMap<String, Option<Image<'a>>>,
    wallpapers: HashMap<String, Wallpaper<'a>>,
    default_preview_image: Image<'a>,

    wallpaper_process: Option<Child>,
    //loader_thread: Option<JoinHandle<HashMap<String, Option<Image<'static>>>>>,
    //load_images: bool,

    /// The current selected screen to set wallpaper.
    select_current_screen: Option<String>,
    ///// Might not be required.
    //select_current_wallpaper: Option<String>,
}

impl Default for MainWindow<'static> {
    fn default() -> Self {
        let binding = std::env::home_dir().unwrap();
        let config_dir = format!("{0}/{1}", binding.to_str().unwrap(), Global::CONFIG_DIR);
        let config_file = format!("{0}/{1}", config_dir, Global::CONFIG_FILE);
        if std::fs::exists(config_dir.clone()).is_ok_and(|x| x == false) {
            std::fs::create_dir(config_dir).expect("Unable to create config dir");
        }
        if std::fs::exists(config_file.clone()).is_ok_and(|x| x == false) {
            let conf = Config::default();
            write_config(config_file.clone(), conf);
        }

        let config = read_config(config_file.clone());
        let mut wallpaper_process: Option<Child> = None;
        if config.auto_start {
            wallpaper_process = Some(start_wallpaper_process(config.clone()));
        }
        let icon_size = config.icon_size;

        let default_preview_image = Image::new(include_image!("UnknownImage.png")).fit_to_exact_size(Vec2::new(icon_size, icon_size));


        //let loader_handle = reload_images();
        let x = Self {
            config,
            //icon_size: 200.0,
            wallpaper: None,
            wallpapers: HashMap::new(),
            //preview_images: HashMap::new(),
            default_preview_image,
            wallpaper_process,
            //loader_thread: None,
            //load_images: true,

            select_current_screen: None,
            //select_current_wallpaper: None,
        };
        x
    }
}


impl MainWindow<'static> {
    fn load_next_image(&mut self) -> bool {
        // TODO: Find first image in list that is None and load it.  Hint: List is probably not from self.preview_images and that needs a bit of an overhaul.
        let wps = get_wallpapers().unwrap();
        for wp in wps {
            if !self.wallpapers.contains_key(&wp.id) {
                self.wallpapers.insert(wp.id.clone(), Wallpaper { wallpaper_info: wp.clone(), image: Some(Image::new(format!("file://{0}", wp.preview_file.clone()))) });
                return true;
            }
            else {
                let x = self.wallpapers.get_mut(&wp.id).unwrap();
                if x.image.is_none() {
                    x.image =Some(Image::new(format!("file://{0}", wp.preview_file.clone())));
                    return true;
                }
            }
        }
        false
    }

    //fn check_wallpapers(&mut self) {
    //    let path = format!("{0}/{1}/{2}", std::env::home_dir().expect("ERROR1").to_str().expect("ERROR2"), Global::CONFIG_DIR, Global::WALLPAPER_DIR);
    //    if (std::fs::exists(path.clone())).is_ok() {
    //        std::fs::create_dir_all(path.clone()).expect("Unable to create wallpaper dir");
    //    }
    //    let paths = std::fs::read_dir(path)?;
    //    let mut result: Vec<WallpaperFileInfo> = Vec::new();
    //    for path in paths {
    //        let path = path?.path();
    //        result.push(WallpaperFileInfo::new(path)?);
    //    }
    //    if result.is_empty() {
    //        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Directory empty").into());
    //    }
    //    Ok(result)
    //}

    fn set_screen_wallpaper(&mut self, screen: String, wallpaper: String) {
        if self.wallpaper_process.is_some() {
            self.wallpaper_process.as_mut().unwrap().kill().expect("Unable to kill child process.");
        }
        //let mut config = self.config.clone();
        //let wp = self.config.wallpapers.get(&screen);
        if self.config.wallpapers.contains_key(&screen) {
            *self.config.wallpapers.get_mut(&screen).unwrap() = wallpaper;
        }
        else {
            self.config.wallpapers.insert(screen, wallpaper);
        }

        let config_file = format!("{0}/{1}/{2}", std::env::home_dir().unwrap().to_str().unwrap(), Global::CONFIG_DIR, Global::CONFIG_FILE);
        write_config(config_file, self.config.clone());

        //self.config = config;
        let wp_proc = Some(start_wallpaper_process(self.config.clone()));
        self.wallpaper_process = wp_proc;
    }

    fn delete_wallpaper(wallpaper: WallpaperFileInfo) {
        println!("Deleting wallpaper: {}", wallpaper.full_path.to_str().unwrap());
        std::fs::remove_dir_all(wallpaper.full_path).unwrap()
    }
}

impl eframe::App for MainWindow<'static> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        //if self.load_images {
        //    self.load_images = false;
        //    self.preview_images.clear();
        //    let wps = get_wallpaper_previews().unwrap();
        //    for wp in wps {
        //        println!("{0}", wp);
        //        self.preview_images.insert(wp, Some(self.default_preview_image.clone()));
        //    }
        //}
        //if self.loader_thread.is_none() && self.load_images {
        //    //self.load_images = true;
        //    println!("Start loader thread.");
        //    self.loader_thread = Some(reload_images(get_wallpaper_previews().unwrap()));
        //}
        //else if self.loader_thread.is_some() {
        //    let thread = self.loader_thread.take().unwrap();
        //    //let thread = self.loader_thread.as_ref();

        //    println!("Loading finished. {0}", thread.is_finished());
        //    if thread.is_finished() {
        //        let x = thread.join();
        //        if x.is_ok() {
        //            for (wp, image) in x.unwrap() {
        //                self.preview_images.insert(wp, image);
        //            }
        //            self.loader_thread = None;
        //            self.load_images = false;
        //        }
        //        else{
        //            println!("Error");
        //        }
        //        self.load_images = false;
        //    }
        //}
        let delete_modal = Modal::new(ctx, "delete_modal");
        delete_modal.show(|ui| {
            delete_modal.title(ui, "Delete wallpaper?");
            delete_modal.frame(ui, |ui| {
                delete_modal.body(ui, "Are you sure you want to permanently delete this wallpaper?");
            });
            delete_modal.buttons(ui, |ui| {
                delete_modal.button(ui, "Close");
                if delete_modal.button(ui, "Delete").clicked() {
                    Self::delete_wallpaper(self.wallpaper.clone().unwrap())
                };
            });
        });

        let about_modal = Modal::new(ctx, "about_modal");
        about_modal.show(|ui| {
            about_modal.title(ui, "About wallpaper manager");
            about_modal.frame(ui, |ui| {
                about_modal.body(ui, "wallpaper manager to be used with linux-wallpaperengine");
            });
            about_modal.buttons(ui, |ui| {
                about_modal.button(ui, "Close");
            });
        });

        let import_wallpapers_modal = Modal::new(ctx, "wallpapers_modal");
        import_wallpapers_modal.show(|ui| {
            import_wallpapers_modal.title(ui, "Wallpapers manager");
            import_wallpapers_modal.frame(ui, |ui| {
                // TODO: Implement import wallpapers from steam/wallpaper-engine.
            });
            import_wallpapers_modal.buttons(ui, |ui| {
                import_wallpapers_modal.button(ui, "Close");
            })
        });

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Get wallpapers").clicked() {
                    }
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("Get wallpapers", |ui| {
                    if ui.button("WallpaperHub").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Import from wallpaperengine").clicked() {
                        ui.close_menu();
                    }
                });
                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        about_modal.open();
                        ui.close_menu();
                    }
                });
            });
            egui::Grid::new("top_panel_grid").show(ui, |ui| {
                for screen in self.config.screens.clone() {
                    if ui.button(screen.clone()).clicked() {
                        self.select_current_screen = Some(screen);
                    }
                }
                if self.select_current_screen.is_some() {
                    ui.label(format!("Selected screen: {0}", self.select_current_screen.clone().unwrap()));
                }
            });
            //egui::containers::Window::new("").show(ctx, |ui| {
            //    ui.label("tsæøhkølfgtjhlftg");
            //})
        });

        egui::CentralPanel::default().show(ctx, |ui| { // FIGURE OUT HOW TO ASYNC LOAD IMAGES, 15 SEC WAIT IS UNACCEPTABLE.
            egui::containers::ScrollArea::new([false, true]).show(ui, |ui| {
                egui::Grid::new("WallpaperGrid").show(ui, |ui| {
                    self.load_next_image();
                    let wallpapers = self.wallpapers.clone();
                    let mut column = 0;
                    for (id, wallpaper) in wallpapers {
                        //let image_box = ui.add(ImageButton::new(Image::new(format!("file://{}", wallpaper.preview_file)).fit_to_exact_size(Vec2::new(self.icon_size, self.icon_size))));
                        let mut image = wallpaper.image;
                        if image.is_none() {
                            image = Some(self.default_preview_image.clone());
                        }
                        let image_box = ui.add(ImageButton::new(image.unwrap()));
                        if image_box.clicked() && self.select_current_screen.is_some() {
                            println!("Wallpaper {} clicked.", id.clone());
                            self.set_screen_wallpaper(self.select_current_screen.clone().unwrap(), id.clone());
                            //wallpapers_modal.close();
                        }
                        column += 1;
                        if column == get_column_count(ctx.input(|i: &egui::InputState| i.screen_rect()).width(), self.config.icon_size) {
                            column = 0;
                            ui.end_row();
                        }
                    }
                });
            });
        });
    }
}