use wallpaper_common::Ipc;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut help: bool = args.contains(&"-h".to_string()) | args.contains(&"--help".to_string());
    let _dry_run: bool = args.contains(&"-d".to_string()) | args.contains(&"--dry-run".to_string());
    let version: bool = args.contains(&"-v".to_string()) | args.contains(&"--version".to_string());
    let _save: bool = args.contains(&"-s".to_string()) | args.contains(&"--save".to_string());

    if args.len() == 1 {
        println!("No commands specified.\n");
        help = true;
    }
    if help {
        help_message();
        return;
    }
    if version {
        println!("Version info");
        println!("  -wallpaper-ctl version {}", env!("CARGO_PKG_VERSION"));
        let x = std::process::Command::new("wallpaper-engine").arg("--version").output();
        let y = std::process::Command::new("wallpaper-manager").arg("--version").output();
        if x.is_ok() {
            println!("  -{:?}", x.unwrap().stdout);
        }
        else {
            println!("  -wallpaper-engine not installed or could not be found.");
        }
        if y.is_ok() {
            println!("  -{:?}", y.unwrap().stdout);
        }
        else {
            println!("  -wallpaper-manager not installed or could not be found.");
        }
        return;
    }

    let ipc = Ipc::connect();
    match ipc {
        Ok(mut ipc) => {
            match args[1].to_lowercase().as_str() {
                "set" => {
                    ensure_parameters(&args, 2);
                    let res = ipc.send_change_wallpaper(args.get(2).unwrap().to_string(), args.get(3).unwrap().to_string()).unwrap();
                    println!("{}", res);
                }
                "list-outputs" => {
                    ensure_parameters(&args, 0);
                    let res = ipc.send_list_outputs().unwrap();
                    println!("{}", res);
                }
                "list-wallpapers" => {
                    ensure_parameters(&args, 0);
                    let res = ipc.send_list_wallpapers().unwrap();
                    println!("{}", res);
                }
                "option" => {
                    ensure_parameters(&args, 2);
                    let res = ipc.send_option(args.get(2).unwrap().to_string(), args.get(3).unwrap().to_string()).unwrap();
                    println!("{}", res);
                }
                "restart" => {
                    ensure_parameters(&args, 0);
                    // TODO: Consider other method of "restarting" service, like a non systemd dependent version if needed.
                    std::process::Command::new("systemctl").arg("--user").arg("restart").arg("wallpaper-engine.service").output().expect("Failed to restart wallpaper.");
                }
                "current-config" => {
                    // TODO: Requires config stuff to be finished, implement print config stuff.
                    println!("Nothing yet.");
                }
                _ => help_message()
            }
        }
        Err(_) => println!("No active wallpaper-engine instance found.")
    }

}

fn ensure_parameters(args: &Vec<String>, size: i32) {
    if (args.len()) != (size + 2) as usize {
        println!("Command requires {} parameters.", size);
        panic!("Not enough parameters.");
    }
}

fn help_message() {
    println!("Control an active wallpaper-engine instance.");
    println!();
    println!("Usage: wallpaper-ctl [options...] subcommand <params>");
    println!();
    println!("Subcommands:");
    println!("  set <display-index> <name|id>  Set wallpaper.");
    println!("    - display-index can be 'all' to set for all monitors.");
    println!("  list-outputs                   Get outputs.");
    println!("  list-wallpapers                Get wallpapers.");
    println!("  restart                        Restart the wallpaper daemon.");
    println!("  current-config                 Print out current configuration used within wallpaper-engine, not the config file.");
    println!();
    println!("Options:");
    println!("  -h, --help     print this message and exit.");
    println!("  -d, --dru-run  Pretends to run the action and write it out to console.");
    println!("  -v, --version  print the version and exit.");
    println!("  -s, --save     Saves the current wallpaper configuration.");
}

fn save_to_config() {

}