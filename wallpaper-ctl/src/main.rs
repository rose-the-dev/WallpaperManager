use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use wallpaper_common::{SocketReader};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut help: bool = args.contains(&"-h".to_string()) | args.contains(&"--help".to_string());
    let dry_run: bool = args.contains(&"-d".to_string()) | args.contains(&"--dry-run".to_string());
    let version: bool = args.contains(&"-v".to_string()) | args.contains(&"--version".to_string());
    let save: bool = args.contains(&"-s".to_string()) | args.contains(&"--save".to_string());

    if args.len() == 1 {
        println!("No commands specified.\n");
        help = true;
    }
    if help {
        help_message();
        return;
    }
    if version {
        return;
    }

    let mut sock = UnixStream::connect("/tmp/wallpaper-engine.sock").expect("Failed to connect to wallpaper.sock");
    let mut sock_reader = SocketReader::new(1000);
    match args[1].to_lowercase().as_str() {
        "set" => {
            ensure_parameters(&args, 2);
            let v = format!("set>{}:{}", args[2], args[3]);
            sock.write(v.as_bytes()).unwrap();
            println!("{}", sock_reader.read_socket(&mut sock).unwrap());
            //println!("{}", wallpaper_common::read_socket(&mut sock).unwrap());
        }
        "list-outputs" => {
            ensure_parameters(&args, 0);
            sock.write(b"list-outputs>").unwrap();
            println!("{}", sock_reader.read_socket(&mut sock).unwrap());
            //println!("{}", wallpaper_common::read_socket(&mut sock).unwrap());
        }
        "list-wallpapers" => {
            ensure_parameters(&args, 0);
            sock.write(b"list-wallpapers>").unwrap();
            println!("{}", sock_reader.read_socket(&mut sock).unwrap());
            //println!("{}", wallpaper_common::read_socket(&mut sock).unwrap());
        }
        "restart" => {
            ensure_parameters(&args, 0);
            // TODO: Consider other method of "restarting" service, like a non systemd dependent version if needed.
            std::process::Command::new("systemctl").arg("--user").arg("restart").arg("wallpaper-engine.service").output().expect("Failed to restart wallpaper.");
        }
        _ => help_message()
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
    println!("  list-outputs                   Get outputs.");
    println!("  list-wallpapers                Get wallpapers.");
    println!("  restart                        Restart the wallpaper daemon.");
    println!();
    println!("Options:");
    println!("  -h, --help     print this message and exit.");
    println!("  -d, --dru-run  Pretends to run the action and write it out to console.");
    println!("  -v, --version  print the version and exit.");
    println!("  -s, --save     Saves the current wallpaper configuration.");
}

fn save_to_config() {

}