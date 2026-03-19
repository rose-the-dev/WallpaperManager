mod common;

use crate::common::*;
use std::process::Command;
//use ctrlc::set_handler;

fn main() {
    env_logger::init();
    Command::new("pkill").arg("-f").arg("linux-wallpaperengine").output().expect("Failed to kill wallpaper process.");
    let binding = std::env::home_dir().unwrap();
    //let config_dir = format!("{0}/{1}", binding.to_str().unwrap(), Global::CONFIG_DIR);
    let config_file = format!("{0}/{1}/{2}", binding.to_str().unwrap(), CONFIG_DIR, CONFIG_FILE);
    let mut config_file = config_file.as_str();


    let args: Vec<String> = std::env::args().collect();
    let help: bool = args.contains(&"-h".to_string()) | args.contains(&"--help".to_string());
    let debug: bool = args.contains(&"-d".to_string()) | args.contains(&"--debug".to_string());

    let mut arg_num = 0;
    for i in 1..args.len() {
        let arg = &args[i];
        let pair = arg.split('=').collect::<Vec<&str>>();

        if pair.len() == 2 {
            if pair[0] == "--config" {
                config_file = pair[1];
            }
            else {
                println!("Argument {num} unknown; {arg}", num=arg_num, arg=arg);
                panic!("Argument {num} unknown; {arg}", num=arg_num, arg=arg);
            }
        }

        arg_num += 1;
    }
    if !std::fs::exists(config_file).unwrap() {
        println!("Error: config file {} does not exist!", config_file);
        panic!("Config file {} does not exist!", config_file);
    }
    let config_data = std::fs::read_to_string(&config_file).expect("Error reading config file");
    let config: Config = serde_json::from_str(config_data.as_str()).unwrap();

    //let mut child = start_wallpaper_process(config);

    //let mut child = proc.spawn().expect("failed to execute process");
    //let mut wait = true;
    //set_handler(move || {
    //    println!("Killed process");
    //    child.kill().unwrap();
    //    wait = false;
    //    std::process::exit(0);
    //}).expect("Error setting Ctrl-C handler");
    //while wait { }

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
    if debug {
        println!("{:?}", proc.get_args());
    }
    let out = proc.output().expect("Failed to start wallpaper process.");
    if debug {
        println!("{:?}", out.stdout);
    }
    println!("{:?}", out.stderr);
}