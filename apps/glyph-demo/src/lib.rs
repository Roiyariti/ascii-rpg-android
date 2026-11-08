mod scene;

use ascii_renderer::{Glyph, GlyphRenderer};
use game_core::GameState;
use glam::Vec2;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use turn_system::Action;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, MouseButton, TouchPhase, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

type AppResult<T> = Result<T, Box<dyn std::error::Error>>;

struct Gpu {
    instance: wgpu::Instance,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    renderer: GlyphRenderer,
    drawable: bool,
}

impl Gpu {
    async fn new(event_loop: &ActiveEventLoop, window: Arc<Window>) -> AppResult<Self> {
        let instance =
            wgpu::Instance::new(wgpu::InstanceDescriptor::new_with_display_handle_from_env(
                Box::new(event_loop.owned_display_handle()),
            ));
        let surface = instance.create_surface(window.clone())?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                ..Default::default()
            })
            .await?;
        let info = adapter.get_info();
        eprintln!("Adaptador: {} / {:?}", info.name, info.backend);
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("ASCII RPG GPU"),
                required_limits: wgpu::Limits::downlevel_defaults(),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
                ..Default::default()
            })
            .await?;
        let size = window.inner_size();
        let mut config = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .ok_or("Superficie no compatible")?;
        config.present_mode = wgpu::PresentMode::Fifo;
        surface.configure(&device, &config);
        let renderer = GlyphRenderer::new(&device, &queue, config.format);
        Ok(Self {
            instance,
            surface,
            device,
            queue,
            config,
            renderer,
            drawable: size.width > 0 && size.height > 0,
        })
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.drawable = width > 0 && height > 0;
        if !self.drawable {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    fn draw(&mut self, window: Arc<Window>, glyphs: &[Glyph]) -> AppResult<bool> {
        if !self.drawable {
            return Ok(false);
        }
        let (frame, suboptimal) = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => (frame, false),
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => (frame, true),
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(false);
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return Ok(false);
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                self.surface = self.instance.create_surface(window)?;
                self.surface.configure(&self.device, &self.config);
                return Ok(false);
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err("Error de validación de superficie".into());
            }
        };
        let view = frame.texture.create_view(&Default::default());
        self.renderer
            .render(&self.device, &self.queue, &view, glyphs)?;
        window.pre_present_notify();
        self.queue.present(frame);
        if suboptimal {
            self.surface.configure(&self.device, &self.config);
        }
        Ok(true)
    }
}

struct App {
    // La GPU se libera en suspended antes de abandonar la ventana nativa Android.
    gpu: Option<Gpu>,
    window: Option<Arc<Window>>,
    state: GameState,
    glyphs: Vec<Glyph>,
    start: Instant,
    next_frame: Instant,
    cursor: Vec2,
    frames: u32,
    smoke_frames: Option<u32>,
    error: Option<String>,
}

impl App {
    fn new() -> AppResult<Self> {
        let now = Instant::now();
        let smoke_frames = std::env::var("ASCII_RPG_SMOKE_FRAMES")
            .ok()
            .map(|value| value.parse())
            .transpose()?;
        Ok(Self {
            gpu: None,
            window: None,
            state: GameState::default(),
            glyphs: Vec::with_capacity(4096),
            start: now,
            next_frame: now,
            cursor: Vec2::ZERO,
            frames: 0,
            smoke_frames,
            error: None,
        })
    }

    fn fail(&mut self, event_loop: &ActiveEventLoop, error: impl ToString) {
        self.error = Some(error.to_string());
        event_loop.exit();
    }

    fn touch(&mut self, point: Vec2) {
        if let Some(window) = &self.window {
            let size = window.inner_size();
            if size.width == 0 || size.height == 0 {
                return;
            }
            let canvas = point / Vec2::new(size.width as f32, size.height as f32)
                * Vec2::from_array(scene::CANVAS);
            if let Some(action) = scene::action_at(canvas) {
                self.state.apply(action);
            }
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gpu.is_some() {
            return;
        }
        let window = match event_loop.create_window(
            Window::default_attributes()
                .with_title("ASCII RPG 2.5D | El umbral de ceniza")
                .with_inner_size(winit::dpi::LogicalSize::new(960.0, 600.0)),
        ) {
            Ok(window) => Arc::new(window),
            Err(error) => {
                self.fail(event_loop, error);
                return;
            }
        };
        match pollster::block_on(Gpu::new(event_loop, window.clone())) {
            Ok(gpu) => {
                self.gpu = Some(gpu);
                self.window = Some(window.clone());
                self.next_frame = Instant::now();
                window.request_redraw();
            }
            Err(error) => self.fail(event_loop, error),
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.gpu = None;
        self.window = None;
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let Some(window) = self.window.clone().filter(|w| w.id() == id) else {
            return;
        };
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(gpu) = &mut self.gpu {
                    gpu.resize(size.width, size.height);
                }
            }
            WindowEvent::KeyboardInput { event, .. }
                if event.state == ElementState::Pressed && !event.repeat =>
            {
                let action = match event.physical_key {
                    PhysicalKey::Code(KeyCode::Escape) => {
                        event_loop.exit();
                        None
                    }
                    PhysicalKey::Code(KeyCode::KeyW | KeyCode::ArrowUp) => {
                        Some(Action::Move { dx: 0, dy: -1 })
                    }
                    PhysicalKey::Code(KeyCode::KeyS | KeyCode::ArrowDown) => {
                        Some(Action::Move { dx: 0, dy: 1 })
                    }
                    PhysicalKey::Code(KeyCode::KeyA | KeyCode::ArrowLeft) => {
                        Some(Action::Move { dx: -1, dy: 0 })
                    }
                    PhysicalKey::Code(KeyCode::KeyD | KeyCode::ArrowRight) => {
                        Some(Action::Move { dx: 1, dy: 0 })
                    }
                    PhysicalKey::Code(KeyCode::Space) => Some(Action::Wait),
                    PhysicalKey::Code(KeyCode::KeyF) => Some(Action::Attack),
                    PhysicalKey::Code(KeyCode::KeyQ) => Some(Action::SwitchWeapon),
                    PhysicalKey::Code(KeyCode::KeyE) => Some(Action::Equip),
                    _ => None,
                };
                if let Some(action) = action {
                    self.state.apply(action);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = Vec2::new(position.x as f32, position.y as f32)
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => self.touch(self.cursor),
            WindowEvent::Touch(touch) if touch.phase == TouchPhase::Started => {
                self.touch(Vec2::new(touch.location.x as f32, touch.location.y as f32))
            }
            WindowEvent::RedrawRequested => {
                scene::build(
                    &mut self.glyphs,
                    &self.state,
                    self.start.elapsed().as_secs_f32(),
                );
                if let Some(gpu) = &mut self.gpu {
                    match gpu.draw(window, &self.glyphs) {
                        Ok(true) => {
                            self.frames += 1;
                            if self.smoke_frames.is_some_and(|limit| self.frames >= limit) {
                                eprintln!(
                                    "Demo: {} frames presentados; {} glifos/frame",
                                    self.frames,
                                    self.glyphs.len()
                                );
                                event_loop.exit();
                            }
                        }
                        Ok(false) => {}
                        Err(error) => {
                            self.fail(event_loop, error);
                            return;
                        }
                    }
                }
                self.next_frame = Instant::now() + Duration::from_micros(16_667);
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            if Instant::now() >= self.next_frame {
                window.request_redraw();
            }
            event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_frame));
        } else {
            event_loop.set_control_flow(ControlFlow::Wait);
        }
    }
}

fn run_loop(event_loop: EventLoop<()>) -> AppResult<()> {
    let mut app = App::new()?;
    event_loop.run_app(&mut app)?;
    match app.error {
        Some(error) => Err(error.into()),
        None => Ok(()),
    }
}

pub fn run() -> AppResult<()> {
    run_loop(EventLoop::new()?)
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub fn android_main(app: winit::platform::android::activity::AndroidApp) {
    use winit::platform::android::EventLoopBuilderExtAndroid;
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag("AsciiRpg"),
    );
    let result = EventLoop::builder()
        .with_android_app(app)
        .build()
        .map_err(|e| -> Box<dyn std::error::Error> { e.into() })
        .and_then(run_loop);
    if let Err(error) = result {
        log::error!("No se pudo ejecutar ASCII RPG: {error}");
    }
}
