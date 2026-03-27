use cairo::{Context, Format, ImageSurface};
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
                             reexports::calloop::{EventLoop, timer::{TimeoutAction, Timer}}
};
use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_output, wl_pointer, wl_seat, wl_shm, wl_surface},
    Connection,
    Proxy,
    QueueHandle,
    globals::GlobalList,
    protocol::wl_output::WlOutput
};
use std::borrow::Borrow;
use std::os::fd::AsRawFd;
use std::time::Duration;
use std::collections::{HashMap};

fn main() {
    let conn = Connection::connect_to_env().unwrap();
    //let ipc_fd = "";

    //let (tx, rx) = std::sync::mpsc::channel::<u8>();

    let mut sub_queue = EventLoop::<SimpleLayer>::try_new().unwrap();
    let (globals, mut event_queue) = registry_queue_init(&conn).unwrap();
    let globals = Box::new(globals);
    let qh: QueueHandle<SimpleLayer> = event_queue.handle();
    let compositor = CompositorState::bind(globals.borrow(), &qh).expect("wl_compositor is not available");
    let layer_shell = LayerShell::bind(globals.borrow(), &qh).expect("layer shell is not available");
    let shm = Shm::bind(&globals, &qh).expect("wl_shm is not available");

    let mut simple_layer = SimpleLayer {
        compositor,
        registry_state: RegistryState::new(globals.borrow()),
        seat_state: SeatState::new(globals.borrow(), &qh),
        output_state: OutputState::new(globals.borrow(), &qh),
        globals,
        shm,

        exit: false,
        //first_configure: true,
        layer_shell,
        layers: HashMap::new(),
        pointer: None,

        debug_draw: true,
    };


    let sub_queue_handle = sub_queue.handle();
    sub_queue_handle.insert_source(smithay_client_toolkit::reexports::calloop_wayland_source::WaylandSource::new(conn, event_queue), |_event, metadata, shared_data| {
        metadata.blocking_dispatch(shared_data)
    }).unwrap();

    sub_queue_handle.insert_source(Timer::from_duration(Duration::from_millis(100)), |_event, _metadata, _shared_data| {
        TimeoutAction::ToDuration(Duration::from_millis(100))
    }).unwrap();

    let result = sub_queue.run(Duration::from_millis(100), &mut simple_layer, |x| {
        if x.exit {
            println!("exiting example");
            return;
        }
    });

    match result {
        Ok(_) => { println!("Program finished without error.") },
        Err(e) => { println!("Program error: {:?}", e) },
    }

    //loop {
    //    //event_queue.dispatch_pending(&mut simple_layer).unwrap();
    //    let ret = event_queue.blocking_dispatch(&mut simple_layer).unwrap();
    //    if simple_layer.exit {
    //        println!("exiting example");
    //        break;
    //    }
    //}
}

fn get_output_size(output: &WlOutput) -> (u32, u32) {
    let output2: Option<&OutputData> = output.data();
    let (width, height) = output2.unwrap().with_output_info(|f| { f.logical_size }).unwrap();
    (width as u32, height as u32)
}

struct LayerData {
    first_configure: bool,
    configured: bool,
    pool: SlotPool,
    layer: LayerSurface,

    width: u32,
    height: u32,
}

struct SimpleLayer {
    globals: Box<GlobalList>,
    compositor: CompositorState,
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    layer_shell: LayerShell,
    shm: Shm,

    // State info
    exit: bool,
    //first_configure: bool,
    layers: HashMap<WlOutput, LayerData>,
    pointer: Option<wl_pointer::WlPointer>,

    // Other stuff
    debug_draw: bool,
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

    pub fn draw(&mut self, qh: &QueueHandle<Self>, layer_num: i32) {
        let mut layer_num = layer_num;
        //println!("drawing layer");

        for (output, layer) in self.layers.iter_mut() {
            if layer_num <= 0 {
                if !layer.configured {
                    return;
                }
                let width = layer.width;
                let height = layer.height;
                let stride = layer.width as i32 * 4;

                let (buffer, canvas) = layer.pool.create_buffer(width as i32, height as i32, stride, wl_shm::Format::Argb8888).expect("create buffer");
                unsafe{
                    let canvas_ptr = canvas.as_mut_ptr();
                    let surface = ImageSurface::create_for_data_unsafe(canvas_ptr, Format::ARgb32, width as i32, height as i32, stride).unwrap();
                    let context = Context::new(&surface).expect("create surface");

                    Self::debug_background(&context, self.debug_draw, width, height);
                }
                let surface = layer.layer.wl_surface();
                buffer.attach_to(surface).expect("buffer attach");
                surface.damage_buffer(0, 0, width as i32, height as i32);
                surface.commit();
                surface.frame(qh, surface.clone());
            }
            else {
                layer_num -= 1;
            }
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
        // Not needed for this example.
    }

    fn transform_changed(&mut self,_conn: &Connection,
                         _qh: &QueueHandle<Self>,
                         _surface: &wl_surface::WlSurface,
                         _new_transform: wl_output::Transform)
    {
        // Not needed for this example.
    }

    fn frame(&mut self,
             _conn: &Connection,
             qh: &QueueHandle<Self>,
             _surface: &wl_surface::WlSurface,
             _time: u32)
    {
        //println!("Frame callback received!");
        self.draw(qh,-1);
    }

    fn surface_enter(&mut self,
                     _conn: &Connection,
                     _qh: &QueueHandle<Self>,
                     _surface: &wl_surface::WlSurface,
                     _output: &WlOutput)
    {
        // Not needed for this example.
    }

    fn surface_leave(&mut self,
                     _conn: &Connection,
                     _qh: &QueueHandle<Self>,
                     _surface: &wl_surface::WlSurface,
                     _output: &WlOutput)
    {
        // Not needed for this example.
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

        self.layers.insert(output, LayerData {
            configured: false,
            first_configure: true,
            pool,
            layer: surface,
            width,
            height,
        });
    }

    fn update_output(&mut self,
                     _conn: &Connection,
                     _qh: &QueueHandle<Self>,
                     output: WlOutput)
    {
        println!("Update callback received.");
        let (width, height) = get_output_size(&output);
        let layer = self.layers.get_mut(&output).unwrap();
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
        self.layers.remove(&output).unwrap();

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
                 qh: &QueueHandle<Self>,
                 _layer: &LayerSurface,
                 configure: LayerSurfaceConfigure,
                 _serial: u32)
    {
        println!("Configure received: {}x{}", configure.new_size.0, configure.new_size.1);
        let mut x: i32 = 0;
        self.layers.iter_mut().for_each(|layer| {
            if _layer.eq(&layer.1.layer) {
                layer.1.width = configure.new_size.0;
                layer.1.height = configure.new_size.1;
                layer.1.configured = true;
                layer.1.first_configure = false;
            }
            else {
                x += 1;
            }
        });
        self.draw(qh, x);

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
            // Ignore events for other surfaces
            if !self.layers.iter().any(|layer| {layer.1.layer.wl_surface() == &event.surface}) {
                continue;
            }

            //if &event.surface != self.layer.wl_surface() {
            //    continue;
            //}
            match event.kind {
                Enter { .. } => {
                    println!("Pointer entered @{:?}", event.position);
                    //self.debug_draw = true;
                }
                Leave { .. } => {
                    println!("Pointer left");
                    //self.debug_draw = false;
                }
                Motion { .. } => {}
                Press { button, .. } => {
                    println!("Press {:x} @ {:?}", button, event.position);
                    //self.shift = self.shift.xor(Some(0));
                }
                Release { button, .. } => {
                    println!("Release {:x} @ {:?}", button, event.position);
                }
                Axis { horizontal, vertical, .. } => {
                    println!("Scroll H:{horizontal:?}, V:{vertical:?}");
                }
            }
        }
    }
}

impl ShmHandler for SimpleLayer {
    fn shm_state(&mut self) -> &mut Shm {
        println!("Shm state: {:?}", self.shm);
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