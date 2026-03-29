mod common;

use std::cmp::PartialEq;
use crate::common::*;
use wallpaper_common::{CONFIG_FILE, CONFIG_DIR, WALLPAPER_DIR};
use std::collections::{BTreeMap, HashMap};
use std::ops::RangeInclusive;
//use std::process::{Child, Command};
use display_info::DisplayInfo;
use eframe::{egui};
use eframe::egui::{include_image, Align2, ComboBox, Image, InnerResponse, Vec2};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let version: bool = args.contains(&"-v".to_string()) | args.contains(&"--version".to_string());
    if version {
        println!("wallpaper-engine version {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    env_logger::init();
    //Command::new("pkill").arg("-f").arg("linux-wallpaperengine").output().expect("Failed to kill wallpaper process.");
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
    config: Config,
    wallpaper: Option<WallpaperInfo>,
    //wallpapers: HashMap<String, Wallpaper<'a>>,
    wallpapers: BTreeMap<String, Wallpaper<'a>>,
    default_preview_image: Image<'a>,
    //wallpaper_process: Option<Child>,
    /// The current selected screen to set wallpaper.
    select_current_screen: Option<String>,

    subpage: Subpage,
}

#[derive(PartialEq)]
enum Subpage { None, About, Delete, Import, GetWallpapers }

impl Default for MainWindow<'static> {
    fn default() -> Self {
        let binding = std::env::home_dir().unwrap();
        let config_dir = format!("{0}/{1}", binding.to_str().unwrap(), CONFIG_DIR);
        let config_file = format!("{0}/{1}", config_dir, CONFIG_FILE);
        if std::fs::exists(config_dir.clone()).is_ok_and(|x| x == false) {
            std::fs::create_dir(config_dir).expect("Unable to create config dir");
        }
        if std::fs::exists(config_file.clone()).is_ok_and(|x| x == false) {
            let conf = Config::default();
            write_config(config_file.clone(), conf);
        }

        let config = read_config(config_file.clone());
        //let mut wallpaper_process: Option<Child> = None;
        //if config.auto_start {
        //    wallpaper_process = Some(start_wallpaper_process(config.clone()));
        //}
        let default_preview_image = Image::new(include_image!("assets/UnknownImage.png"));

        let mut x = Self {
            config,
            wallpaper: None,
            wallpapers: BTreeMap::new(),
            default_preview_image,
            //wallpaper_process,
            select_current_screen: Some(DisplayInfo::all().unwrap()[0].name.clone()),

            subpage: Subpage::None,
        };
        x.load_all_wallpapers();
        x
    }
}


impl MainWindow<'static> {
    fn load_next_image(&mut self) -> bool {
        let wps = get_wallpapers().unwrap(); // TODO: FIX THIS, THIS THROWS ERRORS WHEN ADDING WALLPAPERS
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

    fn load_all_wallpapers(&mut self) -> bool {
        let mut used = false;
        let wps = get_wallpapers().unwrap();
        for wp in wps {
            if !self.wallpapers.contains_key(&wp.id) {
                self.wallpapers.insert(wp.id.clone(), Wallpaper { wallpaper_info: wp.clone(), image: None });
                used = true;
            }
        }
        used
    }

    fn set_screen_wallpaper(&mut self, screen: String, wallpaper_id: String) {
        //kill_wallpaper(self.wallpaper_process.as_mut());

        if self.config.wallpapers.contains_key(&screen) {
            self.config.wallpapers.get_mut(&screen).unwrap().id = wallpaper_id;
        }
        else {
            self.config.wallpapers.insert(screen, ScreenInfo { id: wallpaper_id, scaling: Scaling::Default });
        }

        let config_file = format!("{0}/{1}/{2}", std::env::home_dir().unwrap().to_str().unwrap(), CONFIG_DIR, CONFIG_FILE);
        write_config(config_file, self.config.clone());
        restart_wallpaper_service(ServiceType::Service).expect("Unable to restart service.");

        //let wp_proc = Some(start_wallpaper_process(self.config.clone()));
        //self.wallpaper_process = wp_proc;
    }

    fn delete_wallpaper(&self, wallpaper: WallpaperInfo) {
        if self.config.debugging {
            println!("Deleting wallpaper: {}", wallpaper.full_path.to_str().unwrap());
        }
        std::fs::remove_dir_all(wallpaper.full_path).unwrap()
    }

    fn floating_bg() -> egui::Frame {
        egui::Frame::default()
            .fill(egui::Color32::from_rgb(30, 30, 45))
            .stroke(egui::Stroke::new(1.0, egui::Color32::LIGHT_GRAY))
            .corner_radius(8.0)
            .inner_margin(egui::Margin::same(12))
    }

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
}

impl eframe::App for MainWindow<'static> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let content = ctx.content_rect();
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("Get wallpapers", |ui| {
                    if ui.button("WallpaperHub").clicked() {
                        // TODO: Implement download from hub, also implement hub site.

                        ui.close();
                    }
                    if ui.button("Import from wallpaper engine").clicked() {
                        // TODO: Implement import from wallpaper engine.
                        ui.close();
                    }
                });
                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        //about_modal.open();
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

        let x = egui::SidePanel::right("side_panel").resizable(false).show(ctx, |ui| {
            let mut image = self.default_preview_image.clone().fit_to_exact_size(Vec2::new(250.0, 250.0));
            if self.wallpaper.is_some() {
                image = Image::new(format!("file://{}", self.wallpaper.as_ref().unwrap().preview_file)).fit_to_exact_size(Vec2::new(250.0, 250.0));
            }
            ui.add(image);
            if self.wallpaper.is_some() {
                ui.label(self.wallpaper.as_ref().unwrap().id.clone());
            }

            let mut fps_clicked = self.config.fps.is_some();
            //let mut update = ui.checkbox(&mut self.config.auto_start, "Auto start").changed();
            let mut update = ui.checkbox(&mut self.config.silent, "Silent").changed();
            update = update | ui.checkbox(&mut self.config.no_audio_processing, "No audio processing").changed();
            let fps_changed = ui.checkbox(&mut fps_clicked, "FPS").changed();
            update = update | fps_changed;
            if fps_changed {
                if fps_clicked {
                    self.config.fps = Some(30);
                }
                else {
                    self.config.fps = None;
                }
            }
            if fps_clicked {
                update = update | ui.add(egui::Slider::new(self.config.fps.as_mut().unwrap(), RangeInclusive::new(1, 100))).changed();
            }
            let text = self.config.clamp.clone();
            update = update | ComboBox::from_label("Clamp").selected_text(format!("{:?}", text)).show_ui(ui, |ui| {
                let mut up = ui.selectable_value(&mut self.config.clamp, Clamp::Clamp, "Clamp").changed();
                up = up | ui.selectable_value(&mut self.config.clamp, Clamp::Border, "Border").changed();
                up = up | ui.selectable_value(&mut self.config.clamp, Clamp::Repeat, "Repeat").changed();
                up
            }).inner.unwrap_or(false);

            let text = self.config.wallpapers[self.select_current_screen.clone().unwrap().as_str()].scaling.clone();
            update = update | ComboBox::from_label("Scaling").selected_text(format!("{:?}", text)).show_ui(ui, |ui| {
                let mut x = self.config.wallpapers.get_mut(self.select_current_screen.clone().unwrap().as_str());
                let mut up = ui.selectable_value(&mut x.as_mut().unwrap().scaling, Scaling::Default, "Default").changed();
                up = up | ui.selectable_value(&mut x.as_mut().unwrap().scaling, Scaling::Fit, "Fit").changed();
                up = up | ui.selectable_value(&mut x.as_mut().unwrap().scaling, Scaling::Fill, "Fill").changed();
                up = up | ui.selectable_value(&mut x.as_mut().unwrap().scaling, Scaling::Stretch, "Stretch").changed();
                up
            }).inner.unwrap_or(false);

            update = update | ui.checkbox(&mut self.config.no_fullscreen_pause, "No fullscreen pause").changed();

            if update {
                //kill_wallpaper(self.wallpaper_process.as_mut());
                write_config(format!("{0}/{1}/{2}", std::env::home_dir().unwrap().to_str().unwrap(), CONFIG_DIR, CONFIG_FILE), self.config.clone());
                restart_wallpaper_service(ServiceType::Service).expect("Unable to restart service.");

                //let wp_proc = Some(start_wallpaper_process(self.config.clone()));
                //self.wallpaper_process = wp_proc;
            }
            ui.add_space(30.0);
            ui.separator();

            ui.checkbox(&mut true, "Show Steam Wallpaper engine items");
        });



        let area: Option<InnerResponse<()>> = match self.subpage {
            Subpage::None => {None},
            Subpage::About => {
                let area = egui::Area::new("about_panel".into())
                    .movable(false)
                    .anchor(egui::Align2::CENTER_TOP, [0.0, 0.0])
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
                    .anchor(egui::Align2::CENTER_CENTER, [-x.response.rect.width() / 2.0, 0.0])
                    .pivot(egui::Align2::CENTER_CENTER)
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
                                self.set_screen_wallpaper(self.select_current_screen.clone().unwrap(), id.clone());
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