use std::collections::HashMap;
use ctrlc::set_handler;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Config {
    silent: bool,
    audio_processing: bool,
    screens: HashMap<String, String>, // screen name, wallpaper id
}
#[derive(Serialize, Deserialize)]
struct ScreenInfo {
    screen_name: String,
    wallpaper_id: String,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let home = std::env::var("HOME").unwrap();
    let fmt1 = format!("{}/.local/share/wallpaper.conf", home);
    let mut config_file: &str = fmt1.as_str();

    let mut arg_num = 0;
    for i in 1..args.len() {
        let arg = &args[i];
        println!("{}", arg);
        let pair = arg.split('=').collect::<Vec<&str>>();
        if pair.len() != 2 {
            println!("Error with argument {num}, {arg}.", num=arg_num, arg=arg);
            panic!("Error with argument {num}, {arg}", num=arg_num, arg=arg);
        }
        if pair[0] == "--config" {
            config_file = pair[1];
        }
        else {
            println!("Argument {num} unknown; {arg}", num=arg_num, arg=arg);
            panic!("Argument {num} unknown; {arg}", num=arg_num, arg=arg);
        }
        arg_num += 1;
    }
    if !std::fs::exists(config_file).unwrap(){
        println!("Error: config file {} does not exist!", config_file);
        panic!("Config file {} does not exist!", config_file);
    }
    let config_data = std::fs::read_to_string(&config_file)
        .expect("Error reading config file");
    let config: Config = serde_json::from_str(config_data.as_str()).unwrap();
    let mut proc = std::process::Command::new("linux-wallpaperengine");
    proc.env("XDG_SESSION_TYPE", "wayland");
    if config.silent {
        proc.arg("--silent");
    }
    if !config.audio_processing {
        proc.arg("--no-audio-processing");
    }
    for screen in config.screens {
        proc.arg("--screen-root").arg(screen.0).arg("--bg").arg(screen.1);
    }
    let mut child = proc.spawn().expect("failed to execute process");
    let mut wait = true;
    set_handler(move || {
        println!("Killed process");
        child.kill().unwrap();
        wait = false;
        std::process::exit(0);
    }).expect("Error setting Ctrl-C handler");
    while wait { }
}
