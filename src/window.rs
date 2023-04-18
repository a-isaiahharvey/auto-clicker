use std::{
    sync::{
        mpsc::{self, Sender},
        Arc, Mutex,
    },
    thread::{self, sleep},
    time::Duration,
};

use egui::{FontDefinitions, Style};
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};

use rdev::{simulate, EventType};
use wgpu::Dx12Compiler;
use winit::{
    dpi::{LogicalSize, Size},
    event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder, WindowButtons},
};

use crate::gui::{self, ClickInterval, ClickOptions, ClickPosition, ClickType, MouseButton};

/// A custom event type for the winit app.
enum Event {
    RequestRedraw,
}

/// This is the repaint signal type that egui needs for requesting a repaint from another thread.
/// It sends the custom `RequestRedraw` event to the winit event loop.
struct ExampleRepaintSignal(std::sync::Mutex<winit::event_loop::EventLoopProxy<Event>>);

impl epi::backend::RepaintSignal for ExampleRepaintSignal {
    fn request_repaint(&self) {
        self.0.lock().unwrap().send_event(Event::RequestRedraw).ok();
    }
}

struct State {
    app_gui: gui::MainApp,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    window: Window,
    egui_rpass: RenderPass,
    platform: Platform,
}

impl State {
    // Creating some of the wgpu types requires async code
    async fn new(
        window: Window,
        is_running: Arc<Mutex<bool>>,
        tx_click_interval: Sender<ClickInterval>,
        tx_click_options: Sender<ClickOptions>,
        tx_click_position: Sender<ClickPosition>,
    ) -> State {
        let size = window.inner_size();

        let app_gui = gui::MainApp::new(
            is_running,
            tx_click_interval,
            tx_click_options,
            tx_click_position,
        );

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Dx12Compiler::default(),
        });

        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();
        let surface_caps = surface.get_capabilities(&adapter);

        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.describe().srgb)
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        // We use the egui_winit_platform crate as the platform.
        let platform = Platform::new(PlatformDescriptor {
            physical_width: size.width,
            physical_height: size.height,
            scale_factor: window.scale_factor(),
            font_definitions: FontDefinitions::default(),
            style: Style::default(),
        });

        // We use the egui_wgpu_backend crate as the render backend.
        let egui_rpass = RenderPass::new(&device, surface_format, 1);

        if let Some(theme) = window.theme() {
            use egui::Visuals;
            platform.context().set_visuals(match theme {
                winit::window::Theme::Light => Visuals::light(),
                winit::window::Theme::Dark => Visuals::dark(),
            });
        }

        Self {
            app_gui,
            surface,
            device,
            queue,
            config,
            window,
            egui_rpass,
            platform,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    fn update(&mut self) {}

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.platform.begin_frame();

        self.app_gui.update(&self.platform.context());

        // End the UI frame. We could now handle the output and draw the UI with the backend.
        let full_output = self.platform.end_frame(Some(&self.window));
        let paint_jobs = self.platform.context().tessellate(full_output.shapes);

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            // Upload all resources for the GPU.
            let screen_descriptor = ScreenDescriptor {
                physical_width: self.config.width,
                physical_height: self.config.height,
                scale_factor: self.window().scale_factor() as f32,
            };
            let tdelta: egui::TexturesDelta = full_output.textures_delta;
            self.egui_rpass
                .add_textures(&self.device, &self.queue, &tdelta)
                .expect("add texture ok");
            self.egui_rpass.update_buffers(
                &self.device,
                &self.queue,
                &paint_jobs,
                &screen_descriptor,
            );

            // Record all render passes.
            self.egui_rpass
                .execute(
                    &mut encoder,
                    &view,
                    &paint_jobs,
                    &screen_descriptor,
                    Some(wgpu::Color::BLACK),
                )
                .unwrap();
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_enabled_buttons(WindowButtons::all().difference(WindowButtons::MAXIMIZE))
        .with_resizable(false)
        .with_inner_size(Size::Logical(LogicalSize {
            width: 437.0,
            height: 265.0,
        }))
        .with_title("Auto Clicker")
        .build(&event_loop)
        .unwrap();

    let (tx_click_interval, rx_click_interval) = mpsc::channel::<ClickInterval>();
    let (tx_click_options, rx_click_options) = mpsc::channel::<ClickOptions>();
    let (tx_click_position, rx_click_position) = mpsc::channel::<ClickPosition>();

    let is_running = Arc::new(Mutex::new(false));
    let is_running_autoclick_thread = is_running.clone();
    let is_running_state_thread = is_running.clone();
    thread::spawn(move || {
        let mut is_running = false;
        let mut delay = Duration::from_secs(0);
        let mut mouse_button = rdev::Button::Left;
        let mut click_position = ClickPosition::default();
        let mut click_type = ClickType::default();

        loop {
            if let Ok(value) = is_running_autoclick_thread.lock() {
                is_running = *value;
            }

            if let Ok(click_interval) = rx_click_interval.try_recv() {
                delay = convert_time_to_duration(
                    click_interval.hours,
                    click_interval.minutes,
                    click_interval.seconds,
                    click_interval.milliseconds,
                );
            }

            if let Ok(click_options) = rx_click_options.try_recv() {
                mouse_button = match click_options.mouse_button {
                    MouseButton::Left => rdev::Button::Left,
                    MouseButton::Middle => rdev::Button::Middle,
                    MouseButton::Right => rdev::Button::Right,
                };

                click_type = click_options.click_type;
            }

            if let Ok(position) = rx_click_position.try_recv() {
                click_position = position;
            }

            if is_running {
                if let ClickPosition::Custom { x, y } = click_position {
                    send(&EventType::MouseMove {
                        x: x as f64,
                        y: y as f64,
                    });
                }

                let click_times = match click_type {
                    ClickType::Single => 1,
                    ClickType::Double => 2,
                };

                for _ in 0..click_times {
                    send(&EventType::ButtonPress(mouse_button));
                    send(&EventType::ButtonRelease(mouse_button));
                }
                sleep(delay);
            }
            sleep(Duration::from_millis(5));
        }
    });

    let mut state = State::new(
        window,
        is_running,
        tx_click_interval,
        tx_click_options,
        tx_click_position,
    )
    .await;

    event_loop.run(move |event, _, control_flow| {
        use winit::event::Event;

        control_flow.set_wait();
        state.platform.handle_event(&event);

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::F6),
                            ..
                        },
                    ..
                } => {}
                WindowEvent::ThemeChanged(theme) => {
                    use egui::Visuals;
                    state.platform.context().set_visuals(match theme {
                        winit::window::Theme::Light => Visuals::light(),
                        winit::window::Theme::Dark => Visuals::dark(),
                    });
                    state.window().request_redraw();
                }
                WindowEvent::CursorMoved { .. } => {
                    state.window().request_redraw();
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    if input.state == ElementState::Released {
                        match input.virtual_keycode {
                            Some(VirtualKeyCode::F6) => {
                                *is_running_state_thread.lock().unwrap() = true;
                            }
                            Some(VirtualKeyCode::F7) => {
                                *is_running_state_thread.lock().unwrap() = false;
                            }
                            Some(VirtualKeyCode::F8) => {
                                if let Ok(is_running) = &mut is_running_state_thread.lock() {
                                    **is_running = !**is_running;
                                }
                            }
                            _ => {}
                        };
                    }
                }
                _ => {}
            },
            Event::RedrawRequested(window_id) if window_id == state.window().id() => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{e:?}"),
                }
            }

            _ => {}
        }
    });
}

fn send(event_type: &EventType) {
    let delay = Duration::from_millis(20);
    match simulate(event_type) {
        Ok(()) => (),
        Err(_) => {
            eprintln!("We could not send {event_type:?}");
        }
    }
    // Let ths OS catchup (at least MacOS)
    thread::sleep(delay);
}

fn convert_time_to_duration(
    hours: usize,
    minutes: usize,
    seconds: usize,
    milliseconds: usize,
) -> Duration {
    let total_milliseconds =
        milliseconds + (seconds * 1000) + (minutes * 60 * 1000) + (hours * 60 * 60 * 1000);
    let seconds = total_milliseconds / 1000;
    let nanos = (total_milliseconds % 1000) * 1_000_000;
    Duration::new(seconds as u64, nanos as u32)
}
