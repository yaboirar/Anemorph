//! Windows pixel-buffer integration. No GPU device, shader compiler, or web runtime.
use crate::{App, Context, StartupSelection};
use eframe::egui::{self, ViewportId};
use egui_software_backend::{BufferMutRef, ColorFieldOrder, EguiSoftwareRender};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::{
    num::NonZeroU32,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};
struct Wake {
    when: Instant,
    pass: u64,
}
struct Runner {
    // Surface must be released before its context/window.
    surface: Option<softbuffer::Surface<Arc<Window>, Arc<Window>>>,
    buffer_context: Option<softbuffer::Context<Arc<Window>>>,
    input: Option<egui_winit::State>,
    window: Option<Arc<Window>>,
    ctx: Context,
    painter: EguiSoftwareRender,
    app: Option<App>,
    viewport: egui::ViewportBuilder,
    smoke: Option<PathBuf>,
    selection: Option<StartupSelection>,
    due: Option<Instant>,
    last_present: Option<Instant>,
    events: Vec<egui::Event>,
    close_requested: bool,
    error: Option<String>,
    paint_ms: Vec<f64>,
}
impl Runner {
    fn schedule(&mut self, when: Instant) {
        let when = self
            .last_present
            .map_or(when, |last| when.max(last + Duration::from_millis(16)));
        self.due = Some(self.due.map_or(when, |old| old.min(when)));
    }
    fn paint(&mut self, el: &ActiveEventLoop) -> Result<(), String> {
        let started = Instant::now();
        let window = self.window.as_ref().unwrap();
        let size = window.inner_size();
        if size.width == 0 || size.height == 0 || window.is_minimized() == Some(true) {
            return Ok(());
        }
        let input = self.input.as_mut().unwrap();
        let mut raw = input.take_egui_input(window);
        let info = raw.viewports.entry(ViewportId::ROOT).or_default();
        egui_winit::update_viewport_info(info, &self.ctx, window, false);
        if self.close_requested {
            info.events.push(egui::ViewportEvent::Close);
        }
        raw.events.append(&mut self.events);
        let out = self
            .ctx
            .run(raw, |ctx| self.app.as_mut().unwrap().update_ui(ctx));
        input.handle_platform_output(window, out.platform_output);
        let mut actions = Vec::new();
        let mut close = self.close_requested;
        if let Some(v) = out.viewport_output.get(&ViewportId::ROOT) {
            let mut info = egui::ViewportInfo::default();
            for command in &v.commands {
                match command {
                    egui::ViewportCommand::Close => close = true,
                    egui::ViewportCommand::CancelClose => close = false,
                    _ => {}
                }
            }
            egui_winit::process_viewport_commands(
                &self.ctx,
                &mut info,
                v.commands.clone(),
                window,
                &mut actions,
            );
        }
        self.close_requested = false;
        if close {
            self.app.as_mut().unwrap().save_settings();
            el.exit();
            return Ok(());
        }
        let surface = self.surface.as_mut().unwrap();
        surface
            .resize(
                NonZeroU32::new(size.width).unwrap(),
                NonZeroU32::new(size.height).unwrap(),
            )
            .map_err(|e| e.to_string())?;
        let mut buffer = surface.buffer_mut().map_err(|e| e.to_string())?;
        let primitives = self.ctx.tessellate(out.shapes, out.pixels_per_point);
        let pixels: &mut [[u8; 4]] = bytemuck::cast_slice_mut(&mut buffer);
        self.painter.render(
            &mut BufferMutRef::new(pixels, size.width as usize, size.height as usize),
            &primitives,
            &out.textures_delta,
            out.pixels_per_point,
        );
        for action in actions {
            if let egui_winit::ActionRequested::Screenshot(user_data) = action {
                let rgb: Vec<u8> = pixels.iter().flat_map(|p| [p[2], p[1], p[0]]).collect();
                self.events.push(egui::Event::Screenshot {
                    viewport_id: ViewportId::ROOT,
                    user_data,
                    image: Arc::new(egui::ColorImage::from_rgb(
                        [size.width as usize, size.height as usize],
                        &rgb,
                    )),
                });
                self.ctx.request_repaint();
            }
        }
        buffer.present().map_err(|e| e.to_string())?;
        self.last_present = Some(Instant::now());
        if self.smoke.is_some() {
            self.paint_ms.push(started.elapsed().as_secs_f64() * 1000.);
        }
        Ok(())
    }
}
impl ApplicationHandler<Wake> for Runner {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let result = (|| -> Result<(), String> {
            let window = Arc::new(
                egui_winit::create_window(&self.ctx, el, &self.viewport)
                    .map_err(|e| e.to_string())?,
            );
            let hwnd = match window.window_handle().map_err(|e| e.to_string())?.as_raw() {
                RawWindowHandle::Win32(w) => Some(w.hwnd.get() as usize),
                _ => None,
            };
            if let Some(hwnd) = hwnd {
                use windows_sys::Win32::Graphics::Dwm::{
                    DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND, DwmSetWindowAttribute,
                };
                let preference = DWMWCP_ROUND;
                // This is the window just created by this UI thread; DWM copies the value.
                unsafe {
                    DwmSetWindowAttribute(
                        hwnd as _,
                        DWMWA_WINDOW_CORNER_PREFERENCE as u32,
                        (&preference as *const i32).cast(),
                        std::mem::size_of_val(&preference) as u32,
                    );
                }
            }
            let buffer_context =
                softbuffer::Context::new(window.clone()).map_err(|e| e.to_string())?;
            let surface = softbuffer::Surface::new(&buffer_context, window.clone())
                .map_err(|e| e.to_string())?;
            let input = egui_winit::State::new(
                self.ctx.clone(),
                ViewportId::ROOT,
                &window,
                Some(window.scale_factor() as f32),
                None,
                None,
            );
            let mut app = App::initialize_core(
                &self.ctx,
                self.smoke.clone(),
                self.selection.is_none(),
                hwnd,
            );
            if let Some(dir) = &self.smoke {
                let _ = std::fs::write(
                    dir.join("graphics.json"),
                    r#"{"adapter":"Windows pixel buffer","backend":"CPU rasterizer","type":"Cpu"}"#,
                );
            }
            if let Some(selection) = self.selection.take() {
                app.request_selection(selection, &self.ctx);
            }
            self.app = Some(app);
            self.input = Some(input);
            self.surface = Some(surface);
            self.buffer_context = Some(buffer_context);
            window.request_redraw();
            self.window = Some(window);
            Ok(())
        })();
        if let Err(e) = result {
            self.error = Some(e);
            el.exit();
        }
    }
    fn user_event(&mut self, _el: &ActiveEventLoop, event: Wake) {
        if event.pass.saturating_add(1) >= self.ctx.cumulative_pass_nr() {
            self.schedule(event.when);
        }
    }
    fn window_event(&mut self, el: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if self.window.as_ref().is_none_or(|w| w.id() != id) {
            return;
        }
        if matches!(event, WindowEvent::RedrawRequested) {
            self.due = None;
            if let Err(e) = self.paint(el) {
                self.error = Some(e);
                el.exit();
            }
            return;
        }
        if matches!(event, WindowEvent::CloseRequested) {
            self.close_requested = true;
        }
        let response = self
            .input
            .as_mut()
            .unwrap()
            .on_window_event(self.window.as_ref().unwrap(), &event);
        if response.repaint || self.close_requested {
            self.schedule(Instant::now());
        }
    }
    fn about_to_wait(&mut self, el: &ActiveEventLoop) {
        if let Some(due) = self.due {
            if due <= Instant::now() {
                self.due = None;
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
                el.set_control_flow(ControlFlow::Wait);
            } else {
                el.set_control_flow(ControlFlow::WaitUntil(due));
            }
        } else {
            el.set_control_flow(ControlFlow::Wait);
        }
    }
    fn exiting(&mut self, _el: &ActiveEventLoop) {
        if let Some(app) = &mut self.app {
            app.save_settings();
        }
        if let Some(dir) = &self.smoke {
            self.paint_ms.sort_by(f64::total_cmp);
            let n = self.paint_ms.len();
            if n > 0 {
                let _=std::fs::write(dir.join("drawing.json"),serde_json::json!({"frames":n,"mean_full_frame_ms":self.paint_ms.iter().sum::<f64>()/n as f64,"p95_full_frame_ms":self.paint_ms[(n-1)*95/100],"max_full_frame_ms":self.paint_ms[n-1]}).to_string());
            }
        }
    }
}
pub fn run(
    viewport: egui::ViewportBuilder,
    smoke: Option<PathBuf>,
    selection: Option<StartupSelection>,
) -> eframe::Result<()> {
    let event_loop = EventLoop::<Wake>::with_user_event()
        .build()
        .map_err(|e| eframe::Error::AppCreation(Box::new(e)))?;
    let proxy = event_loop.create_proxy();
    let ctx = Context::default();
    ctx.set_request_repaint_callback(move |r| {
        let _ = proxy.send_event(Wake {
            when: Instant::now() + r.delay,
            pass: r.current_cumulative_pass_nr,
        });
    });
    let mut runner = Runner {
        surface: None,
        buffer_context: None,
        input: None,
        window: None,
        ctx,
        painter: EguiSoftwareRender::new(ColorFieldOrder::Bgra),
        app: None,
        viewport,
        smoke,
        selection,
        due: None,
        last_present: None,
        events: Vec::new(),
        close_requested: false,
        error: None,
        paint_ms: Vec::new(),
    };
    event_loop
        .run_app(&mut runner)
        .map_err(|e| eframe::Error::AppCreation(Box::new(e)))?;
    if let Some(e) = runner.error {
        return Err(eframe::Error::AppCreation(Box::new(std::io::Error::other(
            e,
        ))));
    }
    Ok(())
}
