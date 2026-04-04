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
};
use wayland_client::{
    protocol::{wl_output, wl_pointer, wl_seat, wl_surface},
    Connection,
    Proxy,
    QueueHandle,
    protocol::wl_output::WlOutput
};
use std::collections::HashMap;
use wayland_client::protocol::wl_shm;

pub struct LayerData {
    pub wl_output: WlOutput,
    pub configured: bool,
    pub pool: SlotPool,
    pub layer: LayerSurface,

    /// Current wallpaper
    pub wallpaper: Option<(String, ImageSurface)>,
    pub frames: i32,

    pub width: u32,
    pub height: u32,
    /// Mouse pos relative to monitor, ie. the right monitor will see the pointer on the left monitor as a negative value.
    pub mouse_pos: (f64, f64),
}

pub struct SimpleLayer {
    pub compositor: CompositorState,
    pub registry_state: RegistryState,
    pub seat_state: SeatState,
    pub output_state: OutputState,
    pub layer_shell: LayerShell,
    pub shm: Shm,

    // State info
    pub exit: bool,
    pub layers: HashMap<String, LayerData>,
    pub pointer: Option<wl_pointer::WlPointer>,

    // Other stuff
    pub debug_draw: bool,
    pub frames: i32,

    // wallpaper stuff
    /// Absolute mouse position, never negative unless something has gone horribly wrong.
    pub absolute_mouse_pos: (f64, f64),
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
             qh: &QueueHandle<Self>,
             _surface: &wl_surface::WlSurface,
             _time: u32)
    {
        let debug_draw = self.debug_draw;
        self.layers.iter_mut().for_each(|layer| {
            if _surface.eq(layer.1.layer.wl_surface()) {
                layer.1.draw(qh, debug_draw);
            }
        });
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
            pool,
            layer: surface,

            wallpaper: None,
            frames: 0,

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
                 qh: &QueueHandle<Self>,
                 _layer: &LayerSurface,
                 configure: LayerSurfaceConfigure,
                 _serial: u32)
    {
        println!("Configure received: {}x{}", configure.new_size.0, configure.new_size.1);

        let debug_draw = self.debug_draw;
        self.layers.iter_mut().for_each(|layer| {
            if _layer.eq(&layer.1.layer) {
                layer.1.width = configure.new_size.0;
                layer.1.height = configure.new_size.1;
                layer.1.draw(qh, debug_draw);
                layer.1.configured = true;
            }
        });
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
                //Axis { .. } => {}
                //Enter { .. } => {}
                //Leave { .. } => {}
                Motion { .. } => {
                    self.layers.iter_mut().for_each(|(_output, layer)| {
                        let mon_pos = get_output_pos(&layer.wl_output);
                        if &event.surface == layer.layer.wl_surface() {
                            self.absolute_mouse_pos = (event.position.0 + mon_pos.0 as f64, event.position.1 + mon_pos.1 as f64);
                            layer.mouse_pos = (event.position.0, event.position.1);
                        }
                    });
                }
                Press { button, .. } => {
                    println!("Press {:x} @ {:?}", button, event.position);
                }
                Release { button, .. } => {
                    println!("Release {:x} @ {:?}", button, event.position);
                }
                _ => {}
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

pub fn get_output_size(output: &WlOutput) -> (u32, u32) {
    let output2: Option<&OutputData> = output.data();
    let (width, height) = output2.unwrap().with_output_info(|f| { f.logical_size }).unwrap();
    (width as u32, height as u32)
}
pub fn get_output_pos(output: &WlOutput) -> (i32, i32) {
    let output2: Option<&OutputData> = output.data();
    output2.unwrap().with_output_info(|f| { f.logical_position }).unwrap()
}

pub fn get_output_name(output: &WlOutput) -> String {
    let output2: Option<&OutputData> = output.data();
    output2.unwrap().with_output_info(|f| { f.name.clone() }).unwrap()
}

pub fn get_output_proper_name(output: &WlOutput) -> String {
    let output2: Option<&OutputData> = output.data();
    output2.unwrap().with_output_info(|f| { f.model.clone() })
}