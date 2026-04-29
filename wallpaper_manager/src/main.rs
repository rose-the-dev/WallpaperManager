mod common;

use std::cmp::PartialEq;
use crate::common::*;
use wallpaper_common::{CONFIG_FILE, CONFIG_DIR, Clamp, Config, Scaling, ScreenInfo};
use std::collections::{BTreeMap, VecDeque};
use display_info::DisplayInfo;
use eframe::{egui};
use eframe::egui::{include_image, Align2, ComboBox, Image, InnerResponse, Vec2};
use wallpaper_common::wallpaper::WallpaperInfo;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let version: bool = args.contains(&"-v".to_string()) | args.contains(&"--version".to_string());
    if version {
        println!("wallpaper-engine version {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_app_id("wallpaper-manager").with_min_inner_size([400.0, 300.0]).with_inner_size([800.0, 500.0]),
        ..Default::default()
    };
    eframe::run_native(
        "WallpaperManager",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::<MainWindow>::default())
        }),
    ).unwrap();
}

struct MainWindow<'a> {
    frames: u64,
    config: wallpaper_common::Config,
    wallpaper: Option<WallpaperInfo>,
    wallpapers: BTreeMap<String, Wallpaper<'a>>,
    default_preview_image: Image<'a>,
    /// The current selected screen to set wallpaper.
    select_current_screen: Option<String>,
    animations: Vec<(AnimData, String)>,

    subpage: Subpage,
}

#[derive(PartialEq)]
enum Subpage { None, About, Delete, Import, GetWallpapers }

impl Default for MainWindow<'static> {
    fn default() -> Self {
        let binding = std::env::home_dir().unwrap();
        let config_dir = format!("{0}/{1}", binding.to_str().unwrap(), CONFIG_DIR);
        let config_file = format!("{0}/{1}", config_dir, CONFIG_FILE);
        if std::fs::exists(config_dir.clone()).unwrap() == false {
            std::fs::create_dir(config_dir).expect("Unable to create config dir");
        }
        let display_info = DisplayInfo::all().unwrap();
        let mut config: Config;
        let res = Config::from_file(config_file);
        let mut config_error = false;
        match res {
            Ok(x) => config = x,
            Err(e) => {
                config = Config::empty();
                config_error = true;
            },
        }
        let mut screens: Vec<String> = Vec::new();
        //for (name, info) in config.wallpapers.iter() {
        //    if !display_info.iter().any(|x| x.name == *name) {
        //        screens.push(name.clone());
        //    }
        //}

        for info in display_info.iter() {
            if !config.wallpapers.iter().any(|x| x.0.clone() == info.name) {
                screens.push(info.name.clone());
                //screens.push(info.name.clone());
            }
        }
        for screen in screens.iter() {
            config.wallpapers.insert(screen.clone(), Some(ScreenInfo { id: "test".to_string(), scaling: Scaling::Default }));
        }
        screens.clear();
        for (name, info) in config.wallpapers.iter() {
            if !display_info.iter().any(|x| x.name == *name) {
                screens.push(name.clone());
            }
        }

        
        //let mut wallpaper_process: Option<Child> = None;
        //if config.auto_start {
        //    wallpaper_process = Some(start_wallpaper_process(config.clone()));
        //}
        let default_preview_image = Image::new(include_image!("assets/UnknownImage.png"));

        let mut x = Self {
            frames: 0,
            config,
            wallpaper: None,
            wallpapers: BTreeMap::new(),
            default_preview_image,
            select_current_screen: None,
            animations: Vec::new(),

            subpage: Subpage::None,
        };
        x.load_all_wallpapers();
        x
    }
}

struct AnimData {
    /// The frames that the animation starts at.
    start: u64,
    /// Defines the frames for the fade in animation.
    start_frames: u64,
    /// Defines the frames between end of fade in and start of fade out.
    end: u64,
    /// Defines the frames for the fade out animation.
    end_frames: u64,
}

impl AnimData {
    /// time is the number of frames after start.
    fn with_default_frames(start: u64, time: u64) -> Self {
        Self {
            start,
            start_frames: 5,
            end: time,
            end_frames: 30,
        }
    }

    fn has_reached_end(&self, current_frames: u64) -> bool {
        self.start + self.start_frames + self.end + self.end_frames <= current_frames
    }
}


impl MainWindow<'static> {
    fn load_next_image(&mut self) -> bool {
        let wps = wallpaper_common::wallpaper::get_wallpapers().unwrap(); // TODO: FIX THIS, THIS THROWS ERRORS WHEN ADDING WALLPAPERS
        for wp in wps {
            if !self.wallpapers.contains_key(&wp.id) {
                self.wallpapers.insert(wp.id.clone(), Wallpaper { wallpaper_info: wp.clone(), image: Some(Image::new(format!("file://{}/{}", wp.full_path.to_str().unwrap(), wp.data.preview.clone()))) });
                return true;
            } else {
                let x = self.wallpapers.get_mut(&wp.id).unwrap();
                if x.image.is_none() {
                    x.image = Some(Image::new(format!("file://{}/{}", wp.full_path.to_str().unwrap(), wp.data.preview.clone())));
                    return true;
                }
            }
        }
        false
    }

    fn load_all_wallpapers(&mut self) -> bool {
        let mut used = false;
        let wps = wallpaper_common::wallpaper::get_wallpapers().unwrap();
        for wp in wps {
            if !self.wallpapers.contains_key(&wp.id) {
                self.wallpapers.insert(wp.id.clone(), Wallpaper { wallpaper_info: wp.clone(), image: None });
                used = true;
            }
        }
        used
    }

    fn set_screen_wallpaper(&mut self, screen: String, wallpaper_id: String) -> Result<(), std::io::Error> {
        let ipc = wallpaper_common::Ipc::connect();
        match ipc {
            Ok(mut ipc) => {
                ipc.send_change_wallpaper(screen, wallpaper_id)?;
                Ok(())
            }
            Err(e) => {Err(e)}
        }
    }

    fn delete_wallpaper(&self, wallpaper: wallpaper_common::wallpaper::WallpaperInfo) {
        //if self.config.debugging {
            println!("Deleting wallpaper: {}", wallpaper.full_path.to_str().unwrap());
        //}
        std::fs::remove_dir_all(wallpaper.full_path).unwrap()
    }

    fn floating_bg() -> egui::Frame {
        egui::Frame::default()
            .fill(egui::Color32::from_rgb(30, 30, 45))
            .stroke(egui::Stroke::new(1.0, egui::Color32::LIGHT_GRAY))
            .corner_radius(8.0)
            .inner_margin(egui::Margin::same(12))
    }
    fn floating_bg_alpha(alpha: u8) -> egui::Frame {
        egui::Frame::default()
            .fill(egui::Color32::from_rgba_unmultiplied(30, 30, 45, alpha))
            .stroke(egui::Stroke::new(1.0, egui::Color32::LIGHT_GRAY))
            .corner_radius(8.0)
            .inner_margin(egui::Margin::same(12))
    }

    /// Ironic name because this is for detecting clicks outside the floating menus.
    fn floating_clicked(pointer: egui::PointerState, area: egui::Response) -> bool {
        if pointer.primary_clicked() {
            if let Some(pos) = pointer.interact_pos() {
                if !area.rect.contains(pos) {
                    return true;
                }
            }
        }
        return false;
    }


    fn check_banners_open(&mut self) -> bool {
        for (anim, text) in self.animations.iter_mut() {
            anim.has_reached_end(self.frames);
        }


        self.animations.len() > 0
    }

    fn push_banner(&mut self, text: String, anim: AnimData) {
        self.animations.push((anim, text));
    }
}

impl eframe::App for MainWindow<'static> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.frames += 1;
        let _content = ctx.content_rect();
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("Get wallpapers", |ui| {
                    if ui.button("WallpaperHub").clicked() {
                        self.subpage = Subpage::GetWallpapers;
                        ui.close();
                    }
                    if ui.button("Import from wallpaper engine").clicked() {
                        // TODO: Implement import from wallpaper engine.
                        ui.close();
                    }
                });
                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        self.subpage = Subpage::About;
                        ui.close();
                    }
                });
            });
            egui::Grid::new("top_panel_grid").show(ui, |ui| {
                let displays = DisplayInfo::all().expect("Couldn't get display info");
                for screen in displays {
                    if ui.button(&screen.name).clicked() {
                        self.select_current_screen = Some(screen.name);
                    }
                }
                if self.select_current_screen.is_some() {
                    ui.label(format!("Selected screen: {0}", self.select_current_screen.clone().unwrap()));
                }
            });
        });

        egui::Area::new("save_config".into())
            .movable(false)
            .anchor(Align2::RIGHT_BOTTOM, [-40.0, -40.0])
            .show(ctx, |ui| {
            if ui.add(egui::Button::new("Save config").corner_radius(5).min_size([100.0, 35.0].into())).clicked() {
                let config = format!("{}/{}/{}", std::env::home_dir().unwrap().to_str().unwrap(), CONFIG_DIR, CONFIG_FILE);
                self.config.save(config).unwrap();
            };
        });

        let mut side_width: f32 = 0.0;
        if self.select_current_screen.is_some() {
            side_width = egui::SidePanel::right("side_panel").resizable(false).show(ctx, |ui| {
                let mut image = self.default_preview_image.clone().fit_to_exact_size(Vec2::new(250.0, 250.0));
                if self.wallpaper.is_some() {
                    image = Image::new(format!("file://{}/{}", self.wallpaper.as_ref().unwrap().full_path.to_str().unwrap(), self.wallpaper.as_ref().unwrap().data.preview)).fit_to_exact_size(Vec2::new(250.0, 250.0));
                }
                ui.add(image);
                if self.wallpaper.is_some() {
                    ui.label(self.wallpaper.as_ref().unwrap().id.clone());
                }
                let ipc = wallpaper_common::Ipc::connect();
                let ipc_connected = ipc.is_ok();
                let mut fps_clicked = self.config.fps.is_some();
                let update = ui.checkbox(&mut self.config.silent, "Silent").changed();
                if update && ipc_connected {
                    let mut ipc = ipc.unwrap();
                    ipc.send_option("silent".to_string(), format!("{}", self.config.silent)).unwrap();
                    return;
                }

                let update = ui.checkbox(&mut self.config.no_audio_processing, "No audio processing").changed();
                if update && ipc_connected {
                    let mut ipc = ipc.unwrap();
                    ipc.send_option("audio_processing".to_string(), format!("{}", self.config.no_audio_processing)).unwrap();
                    return;
                }

                let fps_changed = ui.checkbox(&mut fps_clicked, "FPS").changed();
                if fps_changed {
                    if fps_clicked {
                        self.config.fps = Some(30);
                    }
                    else {
                        self.config.fps = None;
                    }
                    if ipc_connected {
                        let mut ipc = ipc.unwrap();
                        ipc.send_option("fps-enabled".to_string(), format!("{}", self.config.fps.is_some())).unwrap();
                        return;
                    }
                }
                if fps_clicked {
                    let update = ui.add(egui::Slider::new(self.config.fps.as_mut().unwrap(), 5..=100)).changed();
                    if update && ipc_connected {
                        let mut ipc = ipc.unwrap();
                        ipc.send_option("fps".to_string(), format!("{:?}", self.config.fps)).unwrap();
                        return;
                    }
                }
                let text = self.config.clamp.clone();
                let update = ComboBox::from_label("Clamp").selected_text(format!("{:?}", text)).show_ui(ui, |ui| {
                    let mut up = ui.selectable_value(&mut self.config.clamp, Clamp::Clamp, "Clamp").changed();
                    up = up | ui.selectable_value(&mut self.config.clamp, Clamp::Border, "Border").changed();
                    up = up | ui.selectable_value(&mut self.config.clamp, Clamp::Repeat, "Repeat").changed();
                    up
                }).inner.unwrap_or(false);
                if update && ipc_connected {
                    let mut ipc = ipc.unwrap();
                    ipc.send_option("clamp".to_string(), format!("{:?}", self.config.clamp)).unwrap();
                    return;
                }

                let text = self.config.wallpapers[self.select_current_screen.clone().unwrap().as_str()].as_ref().unwrap().scaling.clone();
                let update = ComboBox::from_label("Scaling").selected_text(format!("{:?}", text)).show_ui(ui, |ui| {
                    let mut x = self.config.wallpapers.get_mut(self.select_current_screen.clone().unwrap().as_str());
                    let val = x.as_mut().unwrap().as_mut().unwrap();
                    let mut up = ui.selectable_value(&mut val.scaling, Scaling::Default, "Default").changed();
                    up = up | ui.selectable_value(&mut val.scaling, Scaling::Fit, "Fit").changed();
                    up = up | ui.selectable_value(&mut val.scaling, Scaling::Fill, "Fill").changed();
                    up = up | ui.selectable_value(&mut val.scaling, Scaling::Stretch, "Stretch").changed();
                    up
                }).inner.unwrap_or(false);
                if update && ipc_connected {
                    let mut ipc = ipc.unwrap();
                    ipc.send_option("scaling".to_string(), format!("{},{:?}", self.select_current_screen.clone().unwrap(), self.config.wallpapers[self.select_current_screen.clone().unwrap().as_str()].as_ref().unwrap().scaling.clone())).unwrap();
                    return;
                }

                let update = ui.checkbox(&mut self.config.no_fullscreen_pause, "No fullscreen pause").changed();
                if update && ipc_connected {
                    let mut ipc = ipc.unwrap();
                    ipc.send_option("fullscreen_pause".to_string(), format!("{}", self.config.no_fullscreen_pause)).unwrap();
                    return;
                }
            }).response.rect.width();
        }
        else {
            egui::Area::new("screen_select_panel".into())
                .movable(false)
                .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {});
        }


        let mut c: i32 = 0;
        for (anim, text) in self.animations.iter() {
            egui::Area::new("warning_area".into())
                .movable(false)
                .anchor(Align2::CENTER_BOTTOM, [0.0, -((50.0 * c as f32) + 50.0)])
                .show(ctx, |ui| {
                    let bg = Self::floating_bg_alpha(255);
                    bg.show(ui, |ui| {
                        ui.add(egui::Label::new(text));
                    });
                });
            c += 1;
        }


        let area: Option<InnerResponse<()>> = match self.subpage {
            Subpage::None => {None},
            Subpage::About => {
                let area = egui::Area::new("about_panel".into())
                    .movable(false)
                    .anchor(Align2::CENTER_TOP, [0.0, 0.0])
                    .show(ctx, |ui| {
                        let bg = Self::floating_bg();
                        bg.show(ui, |ui| {
                            if ui.button("Close").clicked() {
                                self.subpage = Subpage::None;
                            }
                        });
                    });
                Some(area)
            },
            Subpage::Delete => {
                let area = egui::Area::new("delete_panel".into())
                    .movable(false)
                    .anchor(Align2::CENTER_CENTER, [-side_width / 2.0, 0.0])
                    .pivot(Align2::CENTER_CENTER)
                    .show(ctx, |ui| {
                        let bg = Self::floating_bg();
                        bg.show(ui, |ui| {
                            ui.label("Are you sure you want to delete this wallpaper?");
                            ui.horizontal_centered(|ui| {
                                if ui.button("Delete").clicked() {
                                    self.subpage = Subpage::None;
                                    self.delete_wallpaper(self.wallpaper.clone().unwrap());
                                    self.wallpapers.remove(&self.wallpaper.clone().unwrap().id);
                                }
                                if ui.button("Cancel").clicked() {
                                    self.subpage = Subpage::None;
                                }
                            });
                        });
                    });
                Some(area)
            },
            Subpage::Import => {
                let area = egui::Area::new("import_panel".into())
                    .movable(false)
                    .anchor(egui::Align2::CENTER_TOP, [100.0, 0.0])
                    .show(ctx, |ui| {
                        let bg = Self::floating_bg();
                        bg.show(ui, |ui| {
                            ui.label("Import from steam Wallpaper Engine.");
                            if ui.button("Close").clicked() {
                                self.subpage = Subpage::None;
                            }
                        });
                    });
                Some(area)
            },
            Subpage::GetWallpapers => {
                let area = egui::Area::new("get_panel".into())
                    .movable(false)
                    .anchor(egui::Align2::CENTER_TOP, [100.0, 150.0])
                    .show(ctx, |ui| {
                        let bg = Self::floating_bg();
                        bg.show(ui, |ui| {
                            // TODO: Implement download from hub, also implement hub site.
                            if ui.button("Close").clicked() {
                                self.subpage = Subpage::None;
                            }
                        });
                    });
                Some(area)
            }
        };

        if area.is_some() && Self::floating_clicked(ctx.input(|i| i.pointer.clone()), area.unwrap().response) {
            self.subpage = Subpage::None;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_enabled_ui(self.subpage == Subpage::None,  |ui| {
                egui::containers::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        self.load_next_image();
                        let wallpapers: BTreeMap<String, Wallpaper> = self.wallpapers.clone(); // TODO: Add sorting functionality, [None, by-id, by-name]
                        for (id, wallpaper) in wallpapers {
                            let mut image = wallpaper.image;
                            if image.is_none() {
                                image = Some(self.default_preview_image.clone());
                            }
                            let image_box = ui.add(egui::Button::image(image.unwrap().fit_to_exact_size(Vec2::new(self.config.icon_size, self.config.icon_size))));
                            if image_box.clicked() && self.select_current_screen.is_some() {
                                if self.config.debugging {
                                    println!("Wallpaper {} clicked.", id.clone());
                                }
                                self.wallpaper = Some(wallpaper.wallpaper_info.clone());
                                let x = self.set_screen_wallpaper(self.select_current_screen.clone().unwrap(), id.clone());
                                if x.is_err() {
                                    self.push_banner("Error".to_string(), AnimData::with_default_frames(self.frames, 40))
                                }
                            }
                            image_box.context_menu(|ui| {
                                if ui.button("Delete").clicked() {
                                    self.wallpaper = Some(wallpaper.wallpaper_info.clone());
                                    self.subpage = Subpage::Delete;
                                    //self.delete_wallpaper(wallpaper.wallpaper_info.clone());
                                    //self.wallpapers.remove(&id);

                                    ui.close();
                                }
                            });
                        }
                    })
                });
            });
        });
    }
}