#![cfg(any(target_os = "android", target_os = "ios"))]

mod app;

#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;
#[cfg(target_os = "ios")]
use winit::platform::ios::EventLoopBuilderExtIOS;

use std::num::NonZeroU32;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopBuilder};
use winit::window::{Window, WindowAttributes, WindowId};

use egui::ViewportId;
use egui_wgpu::winit::Painter;
use egui_winit::State;

const INITIAL_WIDTH: u32 = 1920;
const INITIAL_HEIGHT: u32 = 1080;

/// A custom event type for the winit app.
#[derive(Debug)]
enum UserEvent {
    RequestRedraw,
}

/// Enable egui to request redraws via a custom Winit event...
#[derive(Clone)]
struct RepaintSignal(
    std::sync::Arc<std::sync::Mutex<winit::event_loop::EventLoopProxy<UserEvent>>>,
);

struct AppState {
    ctx: egui::Context,
    state: Option<State>,
    painter: Option<Painter>,
    window: Option<Arc<Window>>,
    my_app: crate::app::AliasApp,
    repaint_signal: RepaintSignal,
}

impl ApplicationHandler<UserEvent> for AppState {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = WindowAttributes::default()
                .with_decorations(true)
                .with_resizable(true)
                .with_transparent(false)
                .with_title("egui winit + wgpu example")
                .with_inner_size(winit::dpi::PhysicalSize {
                    width: INITIAL_WIDTH,
                    height: INITIAL_HEIGHT,
                });
            let window = event_loop.create_window(window_attributes).unwrap();
            let window = Arc::new(window);

            let mut painter = pollster::block_on(Painter::new(
                self.ctx.clone(),
                egui_wgpu::WgpuConfiguration::default(),
                1,
                None,
                false,
                true,
            ));

            pollster::block_on(painter.set_window(ViewportId::ROOT, Some(window.clone()))).unwrap();

            let pixels_per_point = window.scale_factor() as f32;
            let max_texture_side = painter.max_texture_side();

            let state = State::new(
                self.ctx.clone(),
                ViewportId::ROOT,
                event_loop,
                Some(pixels_per_point),
                window.theme(),
                max_texture_side,
            );

            self.painter = Some(painter);
            self.state = Some(state);
            self.window = Some(window);
        } else if let Some(window) = self.window.as_ref() {
            if let Some(painter) = self.painter.as_mut() {
                pollster::block_on(painter.set_window(ViewportId::ROOT, Some(window.clone())))
                    .unwrap();
                window.request_redraw();
            }
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(painter) = self.painter.as_mut() {
            pollster::block_on(painter.set_window(ViewportId::ROOT, None)).unwrap();
        }
        self.window = None;
        self.state = None;
        self.painter = None;
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if self.window.as_ref().map_or(false, |w| w.id() == window_id) {
            if let (Some(window), Some(state)) = (self.window.as_mut(), self.state.as_mut()) {
                let response = state.on_window_event(window, &event);

                match event {
                    WindowEvent::Resized(size) => {
                        if let Some(painter) = self.painter.as_mut() {
                            if let (Some(w_nz), Some(h_nz)) =
                                (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
                            {
                                painter.on_window_resized(ViewportId::ROOT, w_nz, h_nz);
                            }
                        }
                    }
                    WindowEvent::CloseRequested => {
                        event_loop.exit();
                    }
                    WindowEvent::RedrawRequested => {
                        let raw_input = state.take_egui_input(window);
                        let full_output = self.ctx.run(raw_input, |ctx| {
                            self.my_app.ui(ctx);
                        });
                        state.handle_platform_output(window, full_output.platform_output);

                        if let Some(painter) = self.painter.as_mut() {
                            let pixels_per_point = window.scale_factor() as f32;
                            let prim = self.ctx.tessellate(full_output.shapes, pixels_per_point);
                            painter.paint_and_update_textures(
                                ViewportId::ROOT,
                                pixels_per_point,
                                egui::Rgba::default().to_array(),
                                &prim,
                                &full_output.textures_delta,
                                Vec::new(),
                            );
                        }

                        let repaint_delay = full_output
                            .viewport_output
                            .get(&ViewportId::ROOT)
                            .map(|vo| vo.repaint_delay)
                            .unwrap_or_default();

                        if repaint_delay.is_zero() {
                            window.request_redraw();
                        }
                    }
                    _ => {}
                }

                if response.repaint {
                    window.request_redraw();
                }
            }
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::RequestRedraw => {
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {}

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: winit::event::StartCause) {}
    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {}
    fn memory_warning(&mut self, _event_loop: &ActiveEventLoop) {}
    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        _event: winit::event::DeviceEvent,
    ) {
    }
}

fn _main(event_loop: EventLoop<UserEvent>) {
    let ctx = egui::Context::default();
    let repaint_signal = RepaintSignal(std::sync::Arc::new(std::sync::Mutex::new(
        event_loop.create_proxy(),
    )));
    let repaint_proxy = repaint_signal.0.clone();
    ctx.set_request_repaint_callback(move |_info| {
        repaint_proxy
            .lock()
            .unwrap()
            .send_event(UserEvent::RequestRedraw)
            .ok();
    });

    add_fonts_to_ctx(ctx);

    let mut app_state = AppState {
        ctx,
        state: None,
        painter: None,
        window: None,
        my_app: crate::app::AliasApp::default(),
        repaint_signal,
    };

    event_loop.run_app(&mut app_state).unwrap();
}

#[cfg(any(target_os = "ios", target_os = "android"))]
fn stop_unwind<F: FnOnce() -> T, T>(f: F) -> T {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(t) => t,
        Err(err) => {
            eprintln!("attempt to unwind out of `rust` with err: {:?}", err);
            std::process::abort()
        }
    }
}

#[cfg(target_os = "ios")]
fn _start_app() {
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event()
        .with_main_thread_check(true) // Ensure this runs on the main thread for iOS
        .build()
        .unwrap();
    stop_unwind(|| _main(event_loop));
}

#[unsafe(no_mangle)]
#[inline(never)]
#[cfg(target_os = "ios")]
pub extern "C" fn start_app() {
    _start_app();
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Warn)
        .parse_default_env()
        .init();

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event()
        .build()
        .unwrap();
    _main(event_loop);
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    use winit::platform::android::EventLoopBuilderExtAndroid;

    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Warn),
    );

    let event_loop = EventLoop::<UserEvent>::with_user_event()
        .with_android_app(app)
        .build()
        .unwrap();
    stop_unwind(|| _main(event_loop));
}
