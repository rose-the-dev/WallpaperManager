use std::env::join_paths;
use std::ffi::CString;
use std::fmt::format;
use std::path::{Path, PathBuf};
use eframe::egui;
use eframe::egui::{include_image, Image, Pos2, Widget};
use eframe::egui::menu::menu_button;
use image::ImageReader;

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::<MyApp>::default())
        }),
    )
}

struct MyApp {
    location: String,
    //name: String,
    //age: u32,
}

impl MyApp {
    fn get_string(self, wallpaper_dir: String) -> String {
        let preview_file = "";
        let subDir = format!("{base_dir}/{wallpaper_dir}/", base_dir = self.location, wallpaper_dir = wallpaper_dir);
        std::fs::read_dir(Path::new(subDir.as_str())).unwrap();
        format!("file://{base_dir}/{wallpaper_dir}/{preview}", base_dir = self.location, wallpaper_dir = wallpaper_dir, preview = preview_file)
    }
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            location: "/home/rose/.steam/steam/steamapps/workshop/content/431960".to_owned(),
            //name: "Rosemary".to_owned(),
            //age: 42,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {

            /*ui.heading("My egui Application");
            ui.horizontal(|ui| {
                let name_label = ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.name)
                    .labelled_by(name_label.id);
            });
            ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            if ui.button("Increment").clicked() {
                self.age += 1;
            }
            ui.label(format!("Hello '{}', age {}", self.name, self.age));
            */

            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Exit").clicked() {
                        ui.close_menu();

                    }
                })
            });
            egui::Grid::new("UniqueId1").show(ui, |ui| {
                ui.image("file:///home/rose/.steam/steam/steamapps/workshop/content/431960/894376172/preview.jpg");
                ui.image(format!("file://{0}{1}", self.location, "894376172/preview.jpg"));
                ui.image("file:///home/rose/.steam/steam/steamapps/workshop/content/431960/894376172/preview.jpg");
            });
            //ui.image(egui::include_image!(
            //    "../../../crates/egui/assets/ferris.png"
            //));
        });
    }
}
