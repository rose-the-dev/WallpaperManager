fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut help: bool = args.contains(&"-h".to_string()) | args.contains(&"--help".to_string());
    let dry_run: bool = args.contains(&"-d".to_string()) | args.contains(&"--dru-run".to_string());
    let version: bool = args.contains(&"-v".to_string()) | args.contains(&"--version".to_string());

    if args.len() == 1 {
        println!("No commands specified.");
        help = true;
    }
    if help {
        help_message();
        return;
    }
    if version {
        return;
    }

    match args[1].as_str() {
        "set" => {

        }
        "get" => {

        }
        "restart" => {

        }
        _ => panic!("Not recognised.")
    }
}

fn help_message() {
    println!("Control an active wallpaper-engine instance.");
    println!();
    println!("Usage: wallpaper-ctl [options...] subcommand <params>");
    println!();
    println!("Subcommands:");
    println!("  set <display-index> <name|id>  Set wallpaper.");
    println!("  get                            Get current wallpaper configuration.");
    println!("  restart                        Restart the wallpaper daemon.");
    println!();
    println!("Options:");
    println!("  -h, --help     print this message and exit.");
    println!("  -d, --dru-run  Pretends to run the action and write it out to console.");
    println!("  -v, --version  print the version and exit.");
}