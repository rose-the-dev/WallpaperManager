mod extra;

use cairo::{Context, Format, ImageSurface, IoError, SurfacePattern};
use smithay_client_toolkit::{
                             compositor::{CompositorHandler, CompositorState},
                             output::OutputState,
                             registry::RegistryState,
                             seat::SeatState,
                             shell::{
                                 wlr_layer::LayerShell,
                                 WaylandSurface,
                             },
                             shm::Shm,
                             reexports::calloop::{EventLoop, Interest, Mode}
};
use wayland_client::{
    globals::registry_queue_init,
    protocol::wl_shm,
    Connection,
    QueueHandle,
};
use std::borrow::Borrow;
use std::time::Duration;
use std::collections::{HashMap};
use std::fs::File;
use std::io::Write;
use std::net::Shutdown;
use std::os::unix::net::UnixListener;
use smithay_client_toolkit::reexports::calloop::PostAction;

use extra::*;

fn main() {
    //ffmpeg_next::init().unwrap();
    //let x = ffmpeg_next::decoder::new();
    //x.open().unwrap();

    let args: Vec<String> = std::env::args().collect();
    let version: bool = args.contains(&"-v".to_string()) | args.contains(&"--version".to_string());
    if version {
        println!("wallpaper-engine version {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    if std::fs::exists("/tmp/wallpaper-engine.sock").unwrap() {
        std::fs::remove_file("/tmp/wallpaper-engine.sock").unwrap();
    }
    let sock = UnixListener::bind("/tmp/wallpaper-engine.sock").expect("Failed to creat unix socket.");
    sock.set_nonblocking(true).expect("Failed to set non-blocking.");

    let conn = Connection::connect_to_env().unwrap();

    let mut sub_queue = EventLoop::<SimpleLayer>::try_new().unwrap();
    let (globals, event_queue) = registry_queue_init(&conn).unwrap();
    let qh: QueueHandle<SimpleLayer> = event_queue.handle();
    let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor is not available");
    let layer_shell = LayerShell::bind(&globals, &qh).expect("layer shell is not available");
    let shm = Shm::bind(&globals, &qh).expect("wl_shm is not available");

    let mut simple_layer = SimpleLayer {
        compositor,
        registry_state: RegistryState::new(globals.borrow()),
        seat_state: SeatState::new(globals.borrow(), &qh),
        output_state: OutputState::new(globals.borrow(), &qh),
        layer_shell,
        shm,

        exit: false,
        layers: HashMap::new(),
        pointer: None,

        debug_draw: true,
        frames: 0,

        absolute_mouse_pos: (0.0, 0.0),
    };

    let sub_queue_handle = sub_queue.handle();
    sub_queue_handle.insert_source(smithay_client_toolkit::reexports::calloop_wayland_source::WaylandSource::new(conn, event_queue), |_event, metadata, shared_data| {
        metadata.dispatch_pending(shared_data)
    }).unwrap();
    sub_queue_handle.insert_source(smithay_client_toolkit::reexports::calloop::generic::Generic::new(sock, Interest::READ, Mode::Edge), |event, metadata, shared_data| {
        if event.readable {
            let (mut stream, _addr) = metadata.accept().unwrap();
            let recv = wallpaper_common::read_socket(&mut stream);
            if recv.is_some() {
                let recv = recv.as_ref().unwrap().split(":").collect::<Vec<&str>>();
                let command = *recv.first().unwrap();
                let args = recv.last().unwrap().split(",").collect::<Vec<&str>>();

                match command {
                    "set" => {
                        println!("{}", args[0]);
                        if args[0] == "all" {
                            shared_data.layers.iter_mut().for_each(|x| {
                                set_wallpaper_internal(x.1, args[1].to_string())
                            });
                            stream.write(b"Done.").unwrap_or(0);
                        }
                        else if shared_data.layers.contains_key(args[0]) {

                            set_wallpaper_internal(shared_data.layers.get_mut(&args[0].to_string()).unwrap(), args[1].to_string());
                            stream.write(b"Done.").unwrap_or(0);
                        }
                        else {
                            stream.write(b"Output doesn't exist.").unwrap_or(0);
                        }
                    },
                    "list-outputs" => {
                        let mut out_val = String::new();
                        for (out, layer) in shared_data.layers.iter() {
                            let wp = layer.wallpaper.as_ref();
                            let mut wp2 = "None".to_string();
                            if wp.is_some() {
                                wp2 = wp.unwrap().0.clone();
                            }
                            out_val.push_str(format!("{}: {}\n", out.as_str(), get_output_proper_name(&layer.wl_output)).as_str());
                            out_val.push_str(format!("  -Pos:        {:?}\n", get_output_pos(&layer.wl_output)).as_str());
                            out_val.push_str(format!("  -Size:       {:?}\n", get_output_size(&layer.wl_output)).as_str());
                            out_val.push_str(format!("  -Current WP: {}\n", wp2).as_str());
                            out_val.push_str(format!("  -Tmp:        {}\n\n", "No value yet").as_str());
                        }
                        stream.write(out_val.as_bytes()).unwrap_or(0);
                    }
                    //"list-wallpapers" => {
                    //}
                    "option" => {
                        ;
                    }
                    "save" => { // Probably could implement all the config stuff in engine exclusively, instead of having that done in multiple areas.

                    }
                    _ => {
                        println!("Read {}, {:?}.", command, args);
                        stream.write(b"Not recognised.").unwrap_or(0);
                    }
                }
                stream.shutdown(Shutdown::Both).unwrap();
            }
        }
        Ok(PostAction::Continue)
    }).unwrap();

    let result = sub_queue.run(Duration::from_millis(100), &mut simple_layer, |x| {
        //println!("Funky");
        if x.exit {
            println!("exiting example");
            return;
        }
    });
    match result {
        Ok(_) => { println!("Program finished without error.") },
        Err(e) => { println!("Program error: {:?}\nFailed after {} frames.", e, simple_layer.frames) },
    }
}


fn set_wallpaper_internal(layer: &mut LayerData, wallpaper_id: String) {
    let long_name = format!("{}/{}", wallpaper_common::wallpaper::get_wallpaper_dir(Some(wallpaper_id.clone())), ""); // TODO: Make functional wallpaper json reader and image stuff
    let image_surface = get_image(long_name).unwrap();
    layer.wallpaper = Some((wallpaper_id, image_surface));
}


impl LayerData {
    fn debug_background(context: &Context, width: u32, height: u32) {
        context.set_source_rgba(1.0, 0.0, 1.0, 1.0);
        context.paint().unwrap();

        context.set_source_rgb(1.0, 1.0, 1.0);
        context.set_line_width(1.0);

        for x in 0..=(width / 100) {
            context.move_to(x as f64 * 100.0, 0.0);
            context.line_to(x as f64 * 100.0, height as f64);
            context.stroke().unwrap();
        }
        for x in 0..=(height / 100) {
            context.move_to(0.0, x as f64 * 100.0);
            context.line_to(width as f64, x as f64 * 100.0);
            context.stroke().unwrap();
        }
    }

    fn draw(&mut self, qh: &QueueHandle<SimpleLayer>, debug_draw: bool) {
        self.frames += 1;
        //let mut layer_num = layer_num;
        //if !self.configured {
        //    return;
        //}
        //println!("Draw");
        let (width, height) = (self.width, self.height);
        let stride = self.width as i32 * 4;

        let (buffer, canvas) = self.pool.create_buffer(width as i32, height as i32, stride, wl_shm::Format::Argb8888).expect("create buffer");
        let wallpaper = self.wallpaper.as_ref();
        if wallpaper.is_some() {
            let wallpaper = wallpaper.unwrap();
            unsafe{
                let canvas_ptr = canvas.as_mut_ptr();
                let surface = ImageSurface::create_for_data_unsafe(canvas_ptr, Format::ARgb32, width as i32, height as i32, stride).unwrap();
                let context = Context::new(&surface).expect("create surface");
                context.set_source_rgba(0.0, 0.0, 0.0, 1.0); // TODO: Buffer never gets cleared due to how shm and pools work, although that shouldn't matter for a wallpaper manager.
                context.paint().expect("paint");                              // Just a note for future Rose (this could probably be removed).
                let pattern = SurfacePattern::create(&wallpaper.1);

                context.set_source(&pattern).expect("set source");
                context.paint().expect("paint");
            }
        }
        else {
            if debug_draw {
                unsafe{
                    let canvas_ptr = canvas.as_mut_ptr();
                    let surface = ImageSurface::create_for_data_unsafe(canvas_ptr, Format::ARgb32, width as i32, height as i32, stride).unwrap();
                    let context = Context::new(&surface).expect("create surface");

                    Self::debug_background(&context, self.width, self.height);
                }
            }
        }

        let surface = self.layer.wl_surface();
        if self.configured {
            buffer.attach_to(surface).expect("buffer attach");
        }
        surface.damage_buffer(0, 0, width as i32, height as i32);
        surface.commit();
        surface.frame(qh, surface.clone());
    }
}

fn get_image(file: String) -> Result<ImageSurface, IoError> {
    let mut file = File::open(file)?;
    ImageSurface::create_from_png(&mut file)
}