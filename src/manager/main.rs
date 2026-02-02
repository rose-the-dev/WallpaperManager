use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use eframe::{egui};
use eframe::egui::{include_image, Image, ImageButton, ImageSource, Vec2};
use egui_modal::Modal;
use serde::{Deserialize, Serialize};
//use serde_json::{Result, Value};

struct Global;
impl Global {
    const CONFIG_DIR: &str = ".config/wallpaper-gui";
    const CONFIG_FILE: &str = "wallpaper.conf";
    const WALLPAPER_DIR: &str = "wallpapers";
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

#[derive(Serialize, Deserialize)]
struct Config {
    /// Whether to auto start wallpaperengine.
    auto_start: bool,
    /// Current screen configuration, DP-2, HDMI-A-1, eDP-1, etc
    screens: Vec<String>,
    //wallpaper_engine_dir: Option<String>,
    //manager_dir: Option<String>,
    silent: bool,
    no_audio_processing: bool,
    wallpapers: HashMap<String, String>, // screen name, wallpaper id
}
impl Clone for Config {
    fn clone(&self) -> Self {
        Self {
            auto_start: self.auto_start,
            screens: self.screens.clone(),
            //wallpaper_engine_dir: self.wallpaper_engine_dir.clone(),
            //manager_dir: self.manager_dir.clone(),
            silent: self.silent,
            no_audio_processing: self.no_audio_processing,
            wallpapers: self.wallpapers.clone(), // screen name, wallpaper id
        }
    }
}
impl Default for Config {
    fn default() -> Self {
        Self {
            auto_start: false,
            screens: Vec::new(),
            //wallpaper_engine_dir: None,
            //manager_dir: ".config/wallpaper-".to_owned(),
            silent: false,
            no_audio_processing: false,
            wallpapers: HashMap::new(),
        }
    }
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
    //--scaling fill --clamp border --fps 15
    if config.silent {
        proc.arg("--silent");
    }
    if config.no_audio_processing {
        proc.arg("--no-audio-processing");
    }

    for (mon, wp) in config.wallpapers.iter() {
        proc.arg("--screen-root").arg(mon).arg("--bg").arg(wp);
    }

    proc.spawn().expect("Failed to start wallpaper process.")
}

struct WallpaperInfo {
    /// Id of wallpaper (directory without full path of other files)
    id: String,
    /// Full path of wallpaper files with id.
    full_path: PathBuf,
    /// Full path to preview file.
    preview_file: String,
}

impl Clone for WallpaperInfo {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            full_path: self.full_path.clone(),
            preview_file: self.preview_file.clone(),
        }
    }
}

impl WallpaperInfo {
    pub fn new(path: PathBuf) -> core::result::Result<Self, std::io::Error> {
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

struct MainWindow<'a> {
    config: Config,
    page: i32,
    icon_size: f32,
    wallpaper: Option<WallpaperInfo>,
    preview_images: HashMap<String, Option<Image<'a>>>,
    default_preview_image: Image<'a>,

    wallpaper_process: Option<Child>,

    /// The current selected screen to set wallpaper.
    select_current_screen: Option<String>,
    ///// Might not be required.
    //select_current_wallpaper: Option<String>,
}

impl Default for MainWindow<'_> {
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

        // CHECK IF AUTOSTART WALLPAPER PROCESS.
        let config = read_config(config_file.clone());
        //let mut proc = Command::new("linux-wallpaperengine");
        ////--scaling fill --clamp border --fps 15
        //if config.silent {
        //    proc.arg("--silent");
        //}
        //if config.no_audio_processing {
        //    proc.arg("--no-audio-processing");
        //}
        //for (mon, wp) in config.wallpapers.iter() {
        //    proc.arg("--screen-root").arg(mon).arg("--bg").arg(wp);
        //}
        //let wallpaper_process = Some(proc.spawn().expect("Failed to start wallpaper process."));
        let mut wallpaper_process: Option<Child> = None;
        if config.auto_start {
            wallpaper_process = Some(start_wallpaper_process(config));
        }

        let default_preview_image = Image::new(include_image!("UnknownImage.png"));

        Self {
            config: read_config(config_file),
            page: 0,
            icon_size: 200.0,
            wallpaper: None,
            preview_images: HashMap::new(),
            default_preview_image,
            wallpaper_process,

            select_current_screen: None,
            //select_current_wallpaper: None,
        }
    }
}

impl MainWindow<'_> {
    fn get_wallpapers(&self) -> core::result::Result<Vec<WallpaperInfo>, std::io::Error> {
        //let paths = std::fs::read_dir(Path::new(self.config.wallpaper_engine_dir.clone().unwrap().as_str()))?; // WRONG THIS NEEDS TO BE CHANGED.
        let path = format!("{0}/{1}/{2}", std::env::home_dir().expect("ERROR1").to_str().expect("ERROR2"), Global::CONFIG_DIR, Global::WALLPAPER_DIR);
        if (std::fs::exists(path.clone())).is_ok() {
            std::fs::create_dir_all(path.clone()).expect("Unable to create wallpaper dir");
        }
        let paths = std::fs::read_dir(path)?; // WRONG THIS NEEDS TO BE CHANGED.
        let mut result: Vec<WallpaperInfo> = Vec::new();
        for path in paths {
            let path = path?.path();
            result.push(WallpaperInfo::new(path)?);
        }
        if result.is_empty() {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Directory empty").into());
        }
        Ok(result)
    }

    fn get_column_count(window_width: f32, icon_width: f32) -> i32 {
        (window_width / icon_width) as i32
    }
    fn set_screen_wallpaper(&mut self, screen: String, wallpaper: String) {
        if self.wallpaper_process.is_some() {
            self.wallpaper_process.as_mut().unwrap().kill().expect("Unable to kill child process.");
        }

        //let mut config = self.config.clone();
        if self.config.wallpapers.get(&screen).is_none() {
            self.config.wallpapers.insert(screen, wallpaper);
        }
        else {
            *self.config.wallpapers.get_mut(&screen).unwrap() = wallpaper;
        }
        //self.config = config;

        //let mut proc = Command::new("linux-wallpaperengine");
        ////--scaling fill --clamp border --fps 15
        //if self.config.silent {
        //    proc.arg("--silent");
        //}
        //if self.config.no_audio_processing {
        //    proc.arg("--no-audio-processing");
        //}
        //for (mon, wp) in self.config.wallpapers.iter() {
        //    proc.arg("--screen-root").arg(mon).arg("--bg").arg(wp);
        //}
        //self.wallpaper_process = Some(proc.spawn().expect("Failed to start wallpaper process."));
        let wp_proc = Some(start_wallpaper_process(self.config.clone()));
        self.wallpaper_process = wp_proc;
    }

    fn delete_wallpaper(wallpaper: WallpaperInfo) {
        println!("Deleting wallpaper: {}", wallpaper.full_path.to_str().unwrap());
        std::fs::remove_dir_all(wallpaper.full_path).unwrap()
    }
}

impl eframe::App for MainWindow<'_> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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

        let wallpapers_modal = Modal::new(ctx, "wallpapers_modal");
        wallpapers_modal.show(|ui| {
            wallpapers_modal.title(ui, "Wallpapers manager");
            wallpapers_modal.frame(ui, |ui| {
                egui::containers::ScrollArea::new([false, true]).max_height(600.0).show(ui, |ui| {
                    egui::Grid::new("WallpaperGrid").show(ui, |ui| {
                        let wallpapers = self.get_wallpapers().unwrap();
                        let mut column = 0;
                        for wallpaper in wallpapers {
                            let image_box = ui.add(ImageButton::new(Image::new(format!("file://{}", wallpaper.preview_file))
                                .fit_to_exact_size(Vec2::new(self.icon_size, self.icon_size))));
                            if image_box.clicked() {
                                println!("Wallpaper {} clicked.", wallpaper.id.clone());
                                self.set_screen_wallpaper(self.select_current_screen.clone().unwrap(), wallpaper.id.clone());
                                //wallpapers_modal.close();
                            }
                            column += 1;
                            if column == Self::get_column_count(ctx.input(|i: &egui::InputState| i.screen_rect()).width(), self.icon_size) {
                                column = 0;
                                ui.end_row();
                            }
                        }
                    });
                });
            });
            wallpapers_modal.buttons(ui, |ui| {
                wallpapers_modal.button(ui, "Close");
            })
        });

        let screen_modal = Modal::new(ctx, "screen_modal");
        screen_modal.show(|ui| {
            screen_modal.title(ui, "Screen selector");
            screen_modal.frame(ui, |ui| {
                for screen in self.config.screens.clone() {
                    if ui.button(screen.clone()).clicked() {
                        self.select_current_screen = Some(screen);
                        screen_modal.close();
                    }
                }
            });
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
                    let wallpapers = self.get_wallpapers().unwrap();
                    let mut column = 0;
                    for wallpaper in wallpapers {
                        //let image_box = ui.add(ImageButton::new(Image::new(format!("file://{}", wallpaper.preview_file)).fit_to_exact_size(Vec2::new(self.icon_size, self.icon_size))));
                        let mut image: Option<Image> = None;
                        if self.preview_images.contains_key(&wallpaper.id) {
                            image = self.preview_images[wallpaper.id.as_str()].clone();
                        }
                        if image.is_none() {
                            image = Some(self.default_preview_image.clone());
                        }
                        let image_box = ui.add(ImageButton::new(image.unwrap()));
                        if image_box.clicked() {
                            println!("Wallpaper {} clicked.", wallpaper.id.clone());
                            self.set_screen_wallpaper(self.select_current_screen.clone().unwrap(), wallpaper.id.clone());
                            //wallpapers_modal.close();
                        }
                        column += 1;
                        if column == Self::get_column_count(ctx.input(|i: &egui::InputState| i.screen_rect()).width(), self.icon_size) {
                            column = 0;
                            ui.end_row();
                        }
                    }
                });
            });

            return;
            match self.page {
                0 => {
                    for screen in self.config.screens.clone() {
                        if ui.button(screen.clone()).clicked() {
                            self.select_current_screen = Some(screen);
                            wallpapers_modal.open();
                        }
                    }

                },
                1 => {
                    egui::containers::ScrollArea::new([false, true]).show(ui, |ui| {
                        egui::Grid::new("WallpaperGrid").show(ui, |ui| {
                            let wallpapers = self.get_wallpapers().unwrap();
                            let mut column = 0;
                            for wallpaper in wallpapers {
                                ui.add(Image::new(format!("file://{}", wallpaper.preview_file)).fit_to_exact_size(Vec2::new(self.icon_size, self.icon_size))).context_menu(|ui| {
                                    ui.menu_button("Set for screen", |ui| {
                                        let mons = display_info::DisplayInfo::all().unwrap();
                                        for mon in mons {
                                            //self.set_screen_wallpaper(mon.name, ui, wallpaper.id.clone());
                                        }
                                    });
                                    if ui.button("Delete").clicked() {
                                        self.wallpaper = Some(wallpaper);
                                        delete_modal.open();
                                        ui.close_menu();
                                    }
                                });
                                column += 1;
                                if column == Self::get_column_count(ctx.input(|i: &egui::InputState| i.screen_rect()).width(), self.icon_size) {
                                    column = 0;
                                    ui.end_row();
                                }
                            }
                        });
                    });
                },
                _ => {
                    ui.label("Error");
                }
            };
        });
    }
}