mod extra;

use cairo::{Context, Format, ImageSurface, SurfacePattern};
use smithay_client_toolkit::{
                             delegate_compositor, delegate_layer, delegate_output, delegate_pointer, delegate_registry, delegate_seat, delegate_shm, registry_handlers,
                             compositor::{CompositorHandler, CompositorState},
                             output::{OutputHandler, OutputData, OutputState},
                             registry::{ProvidesRegistryState, RegistryState},
                             seat::{
                                 pointer::{PointerEvent, PointerEventKind, PointerHandler},
                                 Capability, SeatHandler, SeatState,
                             },
                             shell::{
                                 wlr_layer::{Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface, LayerSurfaceConfigure},
                                 WaylandSurface,
                             },
                             shm::{slot::SlotPool, Shm, ShmHandler},
                             reexports::calloop::{EventLoop, Interest, Mode}
};
use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_output, wl_pointer, wl_seat, wl_shm, wl_surface},
    Connection,
    Proxy,
    QueueHandle,
    protocol::wl_output::WlOutput
};
use std::borrow::Borrow;
use std::time::Duration;
use std::collections::{HashMap};
use std::fs::File;
use std::io::{Read, Write};
use std::net::Shutdown;
use std::os::unix::net::UnixListener;
use smithay_client_toolkit::reexports::calloop::PostAction;

fn main() {
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

    //let fd = smithay_client_toolkit::reexports::calloop::generic::Generic::new(sock, Interest::READ, Mode::Edge);

    let conn = Connection::connect_to_env().unwrap();

    let mut sub_queue = EventLoop::<SimpleLayer>::try_new().unwrap();
    let (globals, event_queue) = registry_queue_init(&conn).unwrap();
    //let globals = Box::new(globals);
    let qh: QueueHandle<SimpleLayer> = event_queue.handle();
    let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor is not available");
    let layer_shell = LayerShell::bind(&globals, &qh).expect("layer shell is not available");
    let shm = Shm::bind(&globals, &qh).expect("wl_shm is not available");

    //let mut x = 0;
    //let mut layers = HashMap::new();
    //for output in OutputState::new(globals.borrow(), &qh).outputs() {
    //    //if x == 1 {
    //    //    break;
    //    //}
    //    //let (width, height) = get_output_size(&output);
    //    let surface = compositor.create_surface(&qh);
    //    let layer = layer_shell.create_layer_surface(&qh, surface, Layer::Bottom, Some("wallpaper-engine"), Some(&output));
    //    layer.set_anchor(Anchor::all());
    //    layer.set_keyboard_interactivity(KeyboardInteractivity::OnDemand);
    //    layer.set_size(main_width, main_height);
    //    layer.set_exclusive_zone(-1);
    //    layer.commit();
    //    let pool: SlotPool = SlotPool::new((main_width * main_height * 4) as usize, &shm).expect("Failed to create pool");
    //    layers.insert(output, LayerData {
    //        first_configure: true,
    //        configured: false,
    //        pool,
    //        layer,
    //        width: main_width,
    //        height: main_height,
    //    });
    //    x += 1;
    //}

    let mut simple_layer = SimpleLayer {
        compositor,
        registry_state: RegistryState::new(globals.borrow()),
        seat_state: SeatState::new(globals.borrow(), &qh),
        output_state: OutputState::new(globals.borrow(), &qh),
        shm,

        exit: false,
        //first_configure: true,
        layer_shell,
        layers: HashMap::new(),
        pointer: None,

        debug_draw: true,
        frames: 0,
        //sock,

        absolute_mouse_pos: (0.0, 0.0),
    };


    let sub_queue_handle = sub_queue.handle();
    sub_queue_handle.insert_source(smithay_client_toolkit::reexports::calloop_wayland_source::WaylandSource::new(conn, event_queue), |_event, metadata, shared_data| {
        metadata.dispatch_pending(shared_data)
    }).unwrap();

    //sub_queue_handle.insert_source(Timer::from_duration(Duration::from_millis(10)), |_event, _metadata, shared_data| {
    //    println!("Funky");
    //    let x = shared_data.sock.accept();
    //    match x {
    //        Ok((mut stream, _)) => {
    //            let mut x = String::new();
    //            let read_bytes = stream.read_to_string(&mut x).unwrap();
    //            println!("Read {} bytes: {}", x, read_bytes);
    //        },
    //        Err(_) => {}
    //    }
    //    TimeoutAction::ToDuration(Duration::from_millis(100))
    //}).unwrap();
    sub_queue_handle.insert_source(smithay_client_toolkit::reexports::calloop::generic::Generic::new(sock, Interest::READ, Mode::Edge), |event, metadata, shared_data| {
        if event.readable {
            let (mut stream, _addr) = metadata.accept().unwrap();
            let mut reader = wallpaper_common::SocketReader::new(255);
            let recv = reader.read_socket(&mut stream);
            //let recv = wallpaper_common::read_socket(&mut stream);
            if recv.is_some() {
                let recv = recv.as_ref().unwrap().split(">").collect::<Vec<&str>>();
                let command = *recv.first().unwrap();
                let args = recv.last().unwrap().split(":").collect::<Vec<&str>>();

                match command {
                    "set" => {
                        println!("{}", args[0]);
                        if args[0] == "all" {
                            shared_data.layers.iter_mut().for_each(|x| {
                                let mut file = File::open(args[1]).expect("open file");
                                let image_surface = ImageSurface::create_from_png(&mut file).expect("Image surface creation");
                                x.1.wallpaper = Some((args[1].to_string(), image_surface));
                            });
                            shared_data.draw();
                            stream.write(b"Done.").unwrap_or(0);
                        }
                        else if shared_data.layers.contains_key(args[0]) {
                            let mut file = File::open(args[1]).expect("open file");
                            let image_surface = ImageSurface::create_from_png(&mut file).expect("Image surface creation");
                            shared_data.layers.get_mut(&args[0].to_string()).unwrap().wallpaper = Some((args[1].to_string(), image_surface));
                            shared_data.draw();
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
                            out_val.push_str(format!("  -Tmp:        {:?}\n\n", "No value yet").as_str());
                        }
                        stream.write(out_val.as_bytes()).unwrap_or(0);
                    }
                    //"list-wallpapers" => {
                    //}
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

struct LayerData {
    wl_output: WlOutput,
    first_configure: bool,
    configured: bool,
    pool: SlotPool,
    layer: LayerSurface,

    /// Current wallpaper
    wallpaper: Option<(String, ImageSurface)>,

    width: u32,
    height: u32,
    /// Mouse pos relative to monitor, ie. the right monitor will see the pointer on the left monitor as a negative value.
    mouse_pos: (f64, f64),
}

struct SimpleLayer {
    compositor: CompositorState,
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    layer_shell: LayerShell,
    shm: Shm,

    // State info
    exit: bool,
    //first_configure: bool,
    layers: HashMap<String, LayerData>,
    pointer: Option<wl_pointer::WlPointer>,

    // Other stuff
    debug_draw: bool,
    frames: i32,
    //sock: UnixListener,

    // wallpaper stuff
    /// Absolute mouse position, never negative unless something has gone horribly wrong.
    absolute_mouse_pos: (f64, f64),
}

impl SimpleLayer {
    fn debug_background(context: &Context, draw: bool, width: u32, height: u32) {
        if draw {
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
    }

    pub fn draw(&mut self) {
        self.frames += 1;
        //let mut layer_num = layer_num;
        println!("Draw");

        for (_output, layer) in self.layers.iter_mut() {
            if !layer.configured {
                return;
            }
            let (width, height) = (layer.width, layer.height);
            let stride = layer.width as i32 * 4;

            let (buffer, canvas) = layer.pool.create_buffer(width as i32, height as i32, stride, wl_shm::Format::Argb8888).expect("create buffer");
            //let (buffer, canvas) = layer.buffer.as_mut().unwrap();
            //let canvas = layer.buffer.as_mut().unwrap().canvas(&mut layer.pool).unwrap();

            let wallpaper = layer.wallpaper.as_ref();
            if wallpaper.is_some() {
                let wallpaper = wallpaper.unwrap();
                unsafe{
                    let canvas_ptr = canvas.as_mut_ptr();
                    let surface = ImageSurface::create_for_data_unsafe(canvas_ptr, Format::ARgb32, width as i32, height as i32, stride).unwrap();
                    let context = Context::new(&surface).expect("create surface");

                    //let mut file = File::open("/home/rose/Pictures/Wallpapers/PrideBackground1.png").expect("open file");
                    //let image_surface = ImageSurface::create_from_png(&mut file).expect("Image surface creation");
                    let pattern = SurfacePattern::create(&wallpaper.1);

                    context.set_source(&pattern).expect("set source");
                    context.paint().expect("paint");
                    //Self::debug_background(&context, self.debug_draw, width, height);
                }
            }


            let surface = layer.layer.wl_surface();
            buffer.attach_to(surface).expect("buffer attach");
            surface.damage_buffer(0, 0, width as i32, height as i32);
            surface.commit();
            //surface.frame(qh, surface.clone());
        }
    }
}

impl CompositorHandler for SimpleLayer {
    fn scale_factor_changed(&mut self,
                            _conn: &Connection,
                            _qh: &QueueHandle<Self>,
                            _surface: &wl_surface::WlSurface,
                            _new_factor: i32)
    {
    }

    fn transform_changed(&mut self,_conn: &Connection,
                         _qh: &QueueHandle<Self>,
                         _surface: &wl_surface::WlSurface,
                         _new_transform: wl_output::Transform)
    {
    }

    fn frame(&mut self,
             _conn: &Connection,
             _qh: &QueueHandle<Self>,
             _surface: &wl_surface::WlSurface,
             _time: u32)
    {
        println!("Frame callback received!");
        self.draw();
    }

    fn surface_enter(&mut self,
                     _conn: &Connection,
                     _qh: &QueueHandle<Self>,
                     _surface: &wl_surface::WlSurface,
                     _output: &WlOutput)
    {
    }

    fn surface_leave(&mut self,
                     _conn: &Connection,
                     _qh: &QueueHandle<Self>,
                     _surface: &wl_surface::WlSurface,
                     _output: &WlOutput)
    {
    }
}

impl OutputHandler for SimpleLayer {
    fn output_state(&mut self) -> &mut OutputState
    {
        &mut self.output_state
    }

    fn new_output(&mut self,
                  _conn: &Connection,
                  qh: &QueueHandle<Self>,
                  output: WlOutput)
    {
        println!("New output callback received.");
        let (width, height) = get_output_size(&output);

        let ref layer_shell = self.layer_shell;
        let pool: SlotPool = SlotPool::new((width * height * 4) as usize, &self.shm).expect("Failed to create pool");
        let surface = layer_shell.create_layer_surface(qh, self.compositor.create_surface(qh), Layer::Background, Some("wallpaper-engine"), Some(&output));
        surface.set_anchor(Anchor::all());
        surface.set_exclusive_zone(-1);
        surface.set_size(width, height);
        surface.set_keyboard_interactivity(KeyboardInteractivity::OnDemand);
        surface.commit();

        self.layers.insert(get_output_name(&output), LayerData {
            wl_output: output,
            configured: false,
            first_configure: true,
            pool,
            layer: surface,

            wallpaper: None,

            width,
            height,
            mouse_pos: (0.0, 0.0),
        });
    }

    fn update_output(&mut self,
                     _conn: &Connection,
                     _qh: &QueueHandle<Self>,
                     output: WlOutput)
    {
        println!("Update callback received.");
        let (width, height) = get_output_size(&output);
        let layer = self.layers.get_mut(&get_output_name(&output)).unwrap();
        layer.pool.resize((width * height * 4) as usize).expect("Failed to resize pool");
        layer.layer.set_size(width, height);
        layer.layer.commit();
        layer.width = width;
        layer.height = height;
    }

    fn output_destroyed(&mut self,
                        _conn: &Connection,
                        _qh: &QueueHandle<Self>,
                        output: WlOutput)
    {
        println!("Output destroyed.");
        self.layers.remove(&get_output_name(&output)).unwrap();

    }
}

impl LayerShellHandler for SimpleLayer {
    fn closed(&mut self,
              _conn: &Connection,
              _qh: &QueueHandle<Self>,
              _layer: &LayerSurface)
    {
        self.exit = true;
    }

    fn configure(&mut self,
                 _conn: &Connection,
                 _qh: &QueueHandle<Self>,
                 _layer: &LayerSurface,
                 configure: LayerSurfaceConfigure,
                 _serial: u32)
    {
        println!("Configure received: {}x{}", configure.new_size.0, configure.new_size.1);
        self.layers.iter_mut().for_each(|layer| {
            if _layer.eq(&layer.1.layer) {
                layer.1.width = configure.new_size.0;
                layer.1.height = configure.new_size.1;
                layer.1.configured = true;
                layer.1.first_configure = false;

                //let width = layer.1.width;
                //let height = layer.1.height;
                //let stride = layer.1.width as i32 * 4;
                //let (buffer, canvas) = layer.1.pool.create_buffer(width as i32, height as i32, stride, wl_shm::Format::Argb8888).expect("create buffer");
                //layer.1.buffer = Some(buffer);
                //let surface = layer.1.layer.wl_surface();
                //layer.1.buffer.as_mut().unwrap().attach_to(surface).expect("buffer attach");
                //surface.damage_buffer(0, 0, width as i32, height as i32);
                //surface.commit();
                //surface.frame(qh, surface.clone());
            }

        });
        self.draw();

        //if self.layers.get_mut().unwrap().first_configure {
        //    self.draw(qh);
        //    self.first_configure = false;
        //}


    }
}

impl SeatHandler for SimpleLayer {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(&mut self,
                _: &Connection,
                _: &QueueHandle<Self>,
                _: wl_seat::WlSeat)
    {

    }

    fn new_capability(&mut self,
                      _conn: &Connection,
                      qh: &QueueHandle<Self>,
                      seat: wl_seat::WlSeat,
                      capability: Capability)
    {
        if capability == Capability::Pointer && self.pointer.is_none() {
            println!("Set pointer capability");
            let pointer = self.seat_state.get_pointer(qh, &seat).expect("Failed to create pointer");
            self.pointer = Some(pointer);
        }
    }

    fn remove_capability(&mut self,
                         _conn: &Connection,
                         _: &QueueHandle<Self>,
                         _: wl_seat::WlSeat,
                         capability: Capability)
    {
        if capability == Capability::Pointer && self.pointer.is_some() {
            println!("Unset pointer capability");
            self.pointer.take().unwrap().release();
        }
    }

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl PointerHandler for SimpleLayer {
    fn pointer_frame(&mut self,
                     _conn: &Connection,
                     _qh: &QueueHandle<Self>,
                     _pointer: &wl_pointer::WlPointer,
                     events: &[PointerEvent])
    {
        use PointerEventKind::*;
        for event in events {
            if !self.layers.iter().any(|layer| {layer.1.layer.wl_surface() == &event.surface}) {
                continue;
            }
            //if &event.surface != self.layer.wl_surface() {
            //    continue;
            //}
            match event.kind {
                Axis { .. } => {}
                Enter { .. } => {}
                Leave { .. } => {}
                Motion { .. } => {
                    self.layers.iter_mut().for_each(|(_output, layer)| {
                        let mon_pos = get_output_pos(&layer.wl_output);
                        if &event.surface == layer.layer.wl_surface() {
                            self.absolute_mouse_pos = (event.position.0 + mon_pos.0 as f64, event.position.1 + mon_pos.1 as f64);
                            layer.mouse_pos = (event.position.0, event.position.1);
                            //println!("Absolute pos: {:?}", self.absolute_mouse_pos);
                            //println!("{:?} Local pos: {:?}", output, layer.mouse_pos);
                        }
                    });

                    //self.layers.iter_mut().for_each(|(output, layer)| {
                    //    let mon_pos = get_output_pos(&layer.wl_output);
                    //    layer.mouse_pos = (event.position.0 - mon_pos.0 as f64, event.position.1 - mon_pos.1 as f64);
                    //    println!("{:?} Local pos: {:?}", output, layer.mouse_pos);
                    //});
                }
                Press { button, .. } => {
                    println!("Press {:x} @ {:?}", button, event.position);
                }
                Release { button, .. } => {
                    println!("Release {:x} @ {:?}", button, event.position);
                }
            }
        }
    }
}

impl ShmHandler for SimpleLayer {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl ProvidesRegistryState for SimpleLayer {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState, SeatState];
}

delegate_compositor!(SimpleLayer);
delegate_output!(SimpleLayer);
delegate_shm!(SimpleLayer);

delegate_seat!(SimpleLayer);
delegate_pointer!(SimpleLayer);

delegate_layer!(SimpleLayer);

delegate_registry!(SimpleLayer);

fn get_output_size(output: &WlOutput) -> (u32, u32) {
    let output2: Option<&OutputData> = output.data();
    let (width, height) = output2.unwrap().with_output_info(|f| { f.logical_size }).unwrap();
    (width as u32, height as u32)
}
fn get_output_pos(output: &WlOutput) -> (i32, i32) {
    let output2: Option<&OutputData> = output.data();
    output2.unwrap().with_output_info(|f| { f.logical_position }).unwrap()
}

fn get_output_name(output: &WlOutput) -> String {
    let output2: Option<&OutputData> = output.data();
    output2.unwrap().with_output_info(|f| { f.name.clone() }).unwrap()
}

fn get_output_proper_name(output: &WlOutput) -> String {
    let output2: Option<&OutputData> = output.data();
    output2.unwrap().with_output_info(|f| { f.model.clone() })
}