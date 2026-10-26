#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod engine;
mod naming;
mod perf;
mod ui;
mod window_drag;
use eframe::egui::{self, Color32, Context, Modifiers};
use engine::{Change, Row, Settings};
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_ACCENT: Color32 = Color32::from_rgb(0x35, 0x96, 0x97);
use std::{
    path::{Path, PathBuf},
    sync::{Arc, mpsc},
    time::{Duration, Instant},
};

#[derive(Clone, Copy, PartialEq)]
enum Page {
    Home,
    Settings,
    Configuration,
}
impl Page {
    fn index(self) -> usize {
        match self {
            Self::Home => 0,
            Self::Settings => 1,
            Self::Configuration => 2,
        }
    }
}
enum Work {
    Scanned(Result<Vec<PathBuf>, String>),
    Changed(Result<usize, String>, bool),
}
struct Smoke {
    dir: PathBuf,
    stage: usize,
    last: Instant,
    snapshots: usize,
    idle_start: u64,
    native_restore_verified: bool,
}
struct App {
    settings: Settings,
    data: PathBuf,
    folder: String,
    series: String,
    series_suggested: bool,
    format_draft: String,
    format_cache: Option<(String, Result<naming::Format, String>)>,
    about_open: bool,
    native_window: Option<usize>,
    files: Arc<Vec<PathBuf>>,
    preview: Option<mpsc::Receiver<Result<Vec<Row>, String>>>,
    preview_again: bool,
    context: Context,
    undo_available: bool,
    undo_check: Instant,
    rows: Vec<Row>,
    counts: std::cell::Cell<Option<(usize, usize, usize)>>,
    root: Option<PathBuf>,
    status: String,
    error: bool,
    busy: bool,
    receiver: Option<mpsc::Receiver<Work>>,
    page: Page,
    transition: Instant,
    tab_from: f32,
    tab_target: f32,
    tab_x: f32,
    nav_rects: [egui::Rect; 3],
    plan_due: Option<Instant>,
    folder_due: Option<Instant>,
    dirty: bool,
    anchor: Option<usize>,
    check_anchor: Option<(usize, bool)>,
    table_focus: bool,
    draft_color: Color32,
    picker_hue: f32,
    hex: String,
    color_message: String,
    effective_zoom: f32,
    logo: egui::TextureHandle,
    smoke: Option<Smoke>,
    frames: u64,
    ui_micros: u128,
    max_visible_rows: usize,
}
fn parse_hex(s: &str) -> Option<Color32> {
    let s = s.trim().trim_start_matches('#');
    let s = if s.len() == 3 {
        s.chars().flat_map(|c| [c, c]).collect::<String>()
    } else {
        s.to_owned()
    };
    if s.len() != 6 {
        return None;
    }
    let n = u32::from_str_radix(&s, 16).ok()?;
    Some(Color32::from_rgb((n >> 16) as u8, (n >> 8) as u8, n as u8))
}
fn hex(c: Color32) -> String {
    format!("#{:02X}{:02X}{:02X}", c.r(), c.g(), c.b())
}
fn blend(a: Color32, b: Color32, t: f32) -> Color32 {
    Color32::from_rgb(
        (a.r() as f32 * t + b.r() as f32 * (1. - t)) as u8,
        (a.g() as f32 * t + b.g() as f32 * (1. - t)) as u8,
        (a.b() as f32 * t + b.b() as f32 * (1. - t)) as u8,
    )
}
fn on_color(c: Color32) -> Color32 {
    let linear = |n: u8| {
        let n = n as f32 / 255.;
        if n <= 0.04045 {
            n / 12.92
        } else {
            ((n + 0.055) / 1.055).powf(2.4)
        }
    };
    if 0.2126 * linear(c.r()) + 0.7152 * linear(c.g()) + 0.0722 * linear(c.b()) > 0.179 {
        Color32::from_gray(16)
    } else {
        Color32::WHITE
    }
}
impl App {
    fn new(cc: &eframe::CreationContext<'_>, smoke: Option<PathBuf>) -> Self {
        round_window_corners(cc);
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};
        let native_window = cc.window_handle().ok().and_then(|h| match h.as_raw() {
            RawWindowHandle::Win32(w) => Some(w.hwnd.get() as usize),
            _ => None,
        });
        let mut fonts = egui::FontDefinitions::default();
        // Use the user's installed font; no proprietary font data is bundled.
        let font_path = std::env::var_os("WINDIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(r"C:\Windows"))
            .join("Fonts/arialbd.ttf");
        let mut brand_fonts = fonts.families[&egui::FontFamily::Proportional].clone();
        if let Ok(bytes) = std::fs::read(font_path) {
            fonts.font_data.insert(
                "arial-bold".into(),
                egui::FontData::from_owned(bytes).into(),
            );
            brand_fonts.insert(0, "arial-bold".into());
        }
        fonts
            .families
            .insert(egui::FontFamily::Name("brand".into()), brand_fonts);
        cc.egui_ctx.set_fonts(fonts);
        let data = smoke
            .as_ref()
            .map(|p| p.join("settings"))
            .unwrap_or_else(engine::data_dir);
        let settings = Settings::load(&data.join("settings.json"));
        let color = settings
            .accent_hex
            .as_deref()
            .and_then(parse_hex)
            .unwrap_or(DEFAULT_ACCENT);
        let logo = image::load_from_memory(include_bytes!("../assets/icon.png"))
            .expect("Embedded logo")
            .to_rgba8();
        let logo = cc.egui_ctx.load_texture(
            "anemorph-brand",
            egui::ColorImage::from_rgba_unmultiplied(
                [logo.width() as usize, logo.height() as usize],
                logo.as_raw(),
            ),
            egui::TextureOptions::LINEAR,
        );
        let startup = if settings.remember_folder {
            settings.last_folder.clone()
        } else {
            None
        };
        let format_draft = settings.number_format.clone();
        let mut app = Self {
            settings,
            data: data.clone(),
            folder: String::new(),
            series: String::new(),
            series_suggested: false,
            format_draft,
            format_cache: None,
            about_open: false,
            native_window,
            files: Arc::new(vec![]),
            preview: None,
            preview_again: false,
            context: cc.egui_ctx.clone(),
            undo_available: data.join("native-last-batch.json").is_file()
                || data.join("native-last-batch.staged").is_file(),
            undo_check: Instant::now(),
            rows: vec![],
            counts: std::cell::Cell::new(None),
            root: None,
            status: "Choose a folder to begin.".into(),
            error: false,
            busy: false,
            receiver: None,
            page: Page::Home,
            transition: Instant::now() - Duration::from_secs(1),
            tab_from: 0.,
            tab_target: 0.,
            tab_x: 0.,
            nav_rects: [egui::Rect::NOTHING; 3],
            plan_due: None,
            folder_due: None,
            dirty: false,
            anchor: None,
            check_anchor: None,
            table_focus: false,
            draft_color: color,
            picker_hue: egui::ecolor::Hsva::from(color).h,
            hex: hex(color),
            color_message: String::new(),
            effective_zoom: 1.,
            logo,
            smoke: smoke.map(|dir| Smoke {
                dir,
                stage: 0,
                last: Instant::now(),
                snapshots: 0,
                idle_start: 0,
                native_restore_verified: false,
            }),
            frames: 0,
            ui_micros: 0,
            max_visible_rows: 0,
        };
        app.theme(&cc.egui_ctx);
        if let Some(p) = startup.filter(|p| p.is_dir()) {
            app.open(p, true, &cc.egui_ctx)
        }
        app
    }
    fn save_settings(&mut self) {
        if !self.settings.remember_folder {
            self.settings.last_folder = None
        }
        if let Err(e) = self.settings.save(&self.data.join("settings.json")) {
            self.status = format!("Could not save settings: {e}");
            self.error = true
        }
    }
    fn open(&mut self, root: PathBuf, infer: bool, ctx: &Context) {
        self.folder_due = None;
        if self.busy {
            return;
        }
        if !root.is_dir() {
            self.status = "Choose an existing folder first.".into();
            self.error = true;
            return;
        }
        self.folder = root.to_string_lossy().into();
        if self.settings.auto_name && (infer || self.series.is_empty()) {
            self.series = engine::guess_name(&root);
            self.series_suggested = true;
        }
        self.root = Some(root.clone());
        if self.settings.remember_folder {
            self.settings.last_folder = Some(root.clone());
            self.save_settings()
        }
        let settings = self.settings.clone();
        let (tx, rx) = mpsc::channel();
        let context = ctx.clone();
        self.receiver = Some(rx);
        self.busy = true;
        self.status = "Reading video files…".into();
        self.error = false;
        std::thread::spawn(move || {
            let _ = tx.send(Work::Scanned(engine::scan(&root, &settings)));
            context.request_repaint();
        });
    }
    fn load_typed_folder(&mut self, ctx: &Context) {
        self.folder_due = None;
        let path = self.folder.trim().trim_matches('"');
        if path.is_empty() {
            self.error = false;
            self.status.clear();
            return;
        }
        self.open(PathBuf::from(path), true, ctx);
    }
    fn folder_edited(&mut self, ctx: &Context) {
        self.counts.set(None);
        self.root = None;
        self.rows = Vec::new();
        self.files = Arc::new(Vec::new());
        self.preview_again = self.preview.is_some();
        self.status.clear();
        self.error = false;
        self.dirty = true;
        self.plan_due = None;
        if self.folder.trim().trim_matches('"').is_empty()
            && self.series_suggested
            && self.settings.clear_series_on_empty
        {
            self.series.clear();
            self.series_suggested = false;
        }
        self.folder_due = Some(Instant::now() + Duration::from_millis(500));
        ctx.request_repaint_after(Duration::from_millis(500));
    }
    fn row_counts(&self) -> (usize, usize, usize) {
        if let Some(counts) = self.counts.get() {
            return counts;
        }
        let counts = self
            .rows
            .iter()
            .fold((0, 0, 0), |(ready, included, selected), r| {
                (
                    ready + usize::from(r.ready),
                    included + usize::from(r.ready && r.included),
                    selected + usize::from(r.highlighted),
                )
            });
        self.counts.set(Some(counts));
        counts
    }
    fn any_file_selected(&self) -> bool {
        self.rows.iter().any(|r| r.included || r.highlighted)
    }
    fn toggle_all_files(&mut self) {
        self.counts.set(None);
        let select = !self.any_file_selected();
        for row in &mut self.rows {
            row.included = select && row.ready;
            row.highlighted = select && row.ready;
        }
        self.anchor = None;
        self.check_anchor = None;
        self.table_focus = true;
    }
    fn rebuild(&mut self) {
        self.plan_due = None;
        let Some(root) = &self.root else { return };
        self.dirty = true;
        if self.preview.is_some() {
            self.preview_again = true;
            return;
        }
        if self.files.len() >= 1000 {
            let (tx, rx) = mpsc::channel();
            let root = root.clone();
            let name = self.series.clone();
            let settings = self.settings.clone();
            let files = Arc::clone(&self.files);
            let context = self.context.clone();
            self.preview = Some(rx);
            std::thread::spawn(move || {
                let _ = tx.send(engine::plan(&root, &name, &files, &settings));
                context.request_repaint();
            });
            return;
        }
        let result = engine::plan(root, &self.series, &self.files, &self.settings);
        self.install_preview(result);
    }
    fn install_preview(&mut self, result: Result<Vec<Row>, String>) {
        self.counts.set(None);
        let Some(root) = &self.root else { return };
        match result {
            Ok(rows) => {
                self.rows = rows;
                self.dirty = false;
                self.anchor = None;
                self.check_anchor = None;
                let ready = self.rows.iter().filter(|r| r.ready).count();
                let skipped = self.rows.iter().filter(|r| r.status == "Skipped").count();
                let assumed = self.rows.iter().filter(|r| r.assumed).count();
                let conflicts = self
                    .rows
                    .iter()
                    .filter(|r| r.status == "Name conflict")
                    .count();
                self.status = format!("{ready} ready · {skipped} skipped");
                if assumed > 0 {
                    self.status += &format!(
                        " · {assumed} use S{:02} (no season found)",
                        self.settings.default_season
                    )
                }
                if conflicts > 0 {
                    self.status += &format!(" · {conflicts} name conflicts")
                }
                self.error = conflicts > 0
            }
            Err(e) => {
                // A missing/invalid name invalidates targets, not the source list.
                let highlighted: std::collections::HashSet<_> = self
                    .rows
                    .iter()
                    .filter(|row| row.highlighted)
                    .map(|row| row.source.clone())
                    .collect();
                self.rows = self
                    .files
                    .iter()
                    .map(|source| Row {
                        source: source.clone(),
                        target: None,
                        original: source
                            .strip_prefix(root)
                            .unwrap_or(source)
                            .to_string_lossy()
                            .into_owned(),
                        output: String::new(),
                        status: "Needs input".into(),
                        ready: false,
                        included: false,
                        highlighted: highlighted.contains(source),
                        assumed: false,
                    })
                    .collect();
                self.dirty = true;
                self.status = e;
                self.error = true
            }
        }
    }
    fn refresh_series(&mut self) {
        if self.series.trim().is_empty()
            && let Some(root) = &self.root
        {
            self.series = engine::guess_name(root);
            self.series_suggested = true;
        }
        self.rebuild();
    }
    fn start_window_drag(&self, ctx: &Context) {
        if let Some(handle) = self.native_window {
            window_drag::prepare(handle, 44. * ctx.pixels_per_point());
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
    }
    fn poll(&mut self, ctx: &Context) {
        // No per-frame filesystem calls. Refresh external history at most every
        // two seconds while the app is already awake, without an idle timer.
        if self.undo_check.elapsed() >= Duration::from_secs(2) {
            self.undo_available = self.data.join("native-last-batch.json").is_file()
                || self.data.join("native-last-batch.staged").is_file();
            self.undo_check = Instant::now();
        }
        if let Some(result) = self.preview.as_ref().and_then(|r| match r.try_recv() {
            Ok(result) => Some(result),
            Err(mpsc::TryRecvError::Disconnected) => {
                Some(Err("Preview worker stopped. Refresh to retry.".into()))
            }
            Err(mpsc::TryRecvError::Empty) => None,
        }) {
            self.preview = None;
            if std::mem::take(&mut self.preview_again) {
                self.rebuild();
            } else if !self.busy && self.plan_due.is_none() && self.folder_due.is_none() {
                self.install_preview(result);
            }
        }

        if let Some(result) = self.receiver.as_ref().and_then(|r| match r.try_recv() {
            Ok(result) => Some(result),
            Err(mpsc::TryRecvError::Disconnected) => Some(Work::Scanned(Err(
                "Worker stopped. Refresh to retry.".into(),
            ))),
            Err(mpsc::TryRecvError::Empty) => None,
        }) {
            self.receiver = None;
            self.busy = false;
            match result {
                Work::Scanned(Ok(files)) => {
                    self.files = Arc::new(files);
                    self.preview_again = self.preview.is_some();
                    self.rebuild()
                }
                Work::Scanned(Err(e)) => {
                    self.status = e;
                    self.error = true
                }
                Work::Changed(Ok(n), undo) => {
                    self.undo_check = Instant::now() - Duration::from_secs(3);
                    self.undo_available = !undo;
                    let message = if undo {
                        format!("Restored {n} original filenames.")
                    } else {
                        format!("Renamed {n} files. Undo is available.")
                    };
                    if let Some(root) = self.root.clone() {
                        self.open(root, false, ctx)
                    }
                    self.status = message
                }
                Work::Changed(Err(e), _) => {
                    self.undo_available = self.data.join("native-last-batch.json").is_file()
                        || self.data.join("native-last-batch.staged").is_file();
                    self.status = e;
                    self.error = true
                }
            }
        }
    }
    fn execute(&mut self, undo: bool, ctx: &Context) {
        if self.busy || (!undo && self.dirty) {
            return;
        }
        let changes: Vec<_> = self
            .rows
            .iter()
            .filter(|r| r.ready && r.included)
            .filter_map(|r| {
                r.target.as_ref().map(|target| Change {
                    source: r.source.clone(),
                    target: target.clone(),
                })
            })
            .collect();
        if !undo && changes.is_empty() {
            return;
        }
        let history = self.data.join("native-last-batch.json");
        let (tx, rx) = mpsc::channel();
        let context = ctx.clone();
        self.receiver = Some(rx);
        self.busy = true;
        self.status = if undo {
            "Restoring original names…"
        } else {
            "Renaming files…"
        }
        .into();
        std::thread::spawn(move || {
            let result = if undo {
                engine::undo(&history)
            } else {
                engine::rename_batch(&changes, &history)
            };
            let _ = tx.send(Work::Changed(result, undo));
            context.request_repaint();
        });
    }
    fn navigate(&mut self, page: Page, ctx: &Context) {
        if self.page == page {
            return;
        }
        self.tab_from = self.tab_x;
        self.tab_target = page.index() as f32;
        self.page = page;
        self.transition = Instant::now();
        ctx.request_repaint();
    }
    fn progress(&self) -> f32 {
        if self.settings.animations {
            (self.transition.elapsed().as_secs_f32() / 0.2).min(1.)
        } else {
            1.
        }
    }
    fn mark(&mut self, value: Option<bool>) {
        self.counts.set(None);
        let mark = value.unwrap_or_else(|| {
            self.rows
                .iter()
                .any(|r| r.highlighted && r.ready && !r.included)
        });
        for r in self.rows.iter_mut().filter(|r| r.highlighted && r.ready) {
            r.included = mark
        }
    }
    fn select(&mut self, index: usize, check: bool, mods: Modifiers) {
        self.counts.set(None);
        self.table_focus = true;
        if check {
            let mark = !self.rows[index].included;
            if mods.shift
                && let Some((anchor, value)) = self.check_anchor
            {
                let lo = anchor.min(index);
                let hi = anchor.max(index);
                for (i, r) in self.rows.iter_mut().enumerate() {
                    r.highlighted = i >= lo && i <= hi;
                    if r.highlighted && r.ready {
                        r.included = value
                    }
                }
                return;
            }
            if self.rows[index].highlighted
                && self.rows.iter().filter(|r| r.highlighted).count() > 1
            {
                self.mark(Some(mark))
            } else {
                for (i, r) in self.rows.iter_mut().enumerate() {
                    r.highlighted = i == index
                }
                if self.rows[index].ready {
                    self.rows[index].included = mark
                }
            }
            self.check_anchor = Some((index, mark));
            self.anchor = Some(index);
        } else if mods.shift {
            let anchor = self.anchor.unwrap_or(index);
            for (i, r) in self.rows.iter_mut().enumerate() {
                r.highlighted = i >= anchor.min(index) && i <= anchor.max(index)
            }
        } else if mods.ctrl || mods.command {
            self.rows[index].highlighted = !self.rows[index].highlighted;
            self.anchor = Some(index)
        } else {
            for (i, r) in self.rows.iter_mut().enumerate() {
                r.highlighted = i == index
            }
            self.anchor = Some(index)
        }
    }
}

/// Let the Windows compositor clip the native window, including its client area.
/// Applied once at creation: no per-frame masks, transparency, or resize work.
fn round_window_corners(cc: &eframe::CreationContext<'_>) {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use windows_sys::Win32::Graphics::Dwm::{
        DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND, DwmSetWindowAttribute,
    };
    if let Ok(handle) = cc.window_handle()
        && let RawWindowHandle::Win32(window) = handle.as_raw()
    {
        let preference = DWMWCP_ROUND;
        // Safety: the creation context owns this live HWND; DWM copies the
        // correctly sized preference synchronously and retains no pointer.
        unsafe {
            DwmSetWindowAttribute(
                window.hwnd.get() as _,
                DWMWA_WINDOW_CORNER_PREFERENCE as u32,
                (&preference as *const i32).cast(),
                std::mem::size_of_val(&preference) as u32,
            );
        }
    }
}
fn main() -> eframe::Result {
    let args: Vec<_> = std::env::args_os().collect();
    if args.get(1).is_some_and(|s| s == "--benchmark") {
        let dir = args.get(2).expect("Benchmark output directory");
        perf::run(Path::new(dir)).expect("Isolated benchmark failed");
        return Ok(());
    }
    let smoke = if args.get(1).is_some_and(|s| s == "--smoke-test") {
        args.get(2).map(PathBuf::from)
    } else {
        None
    };
    if let Some(p) = &smoke {
        let _ = std::fs::create_dir_all(p);
    }
    let data = smoke
        .as_ref()
        .map(|p| p.join("settings"))
        .unwrap_or_else(engine::data_dir);
    let settings = Settings::load(&data.join("settings.json"));
    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon.png"))
        .expect("Embedded icon");
    let mut graphics = eframe::egui_wgpu::WgpuConfiguration {
        desired_maximum_frame_latency: Some(1),
        ..Default::default()
    };
    // This utility has a small, bounded scene. Avoid game-oriented allocation
    // hints and extra queued frames; retain vsync and the existing adapter.
    if let eframe::egui_wgpu::WgpuSetup::CreateNew(setup) = &mut graphics.wgpu_setup {
        let descriptor = setup.device_descriptor.clone();
        setup.device_descriptor = Arc::new(move |adapter| {
            let mut device = descriptor(adapter);
            device.memory_hints = wgpu::MemoryHints::MemoryUsage;
            device
        });
    }
    let options = eframe::NativeOptions {
        wgpu_options: graphics,
        viewport: egui::ViewportBuilder::default()
            .with_title("Anemorph")
            .with_inner_size([settings.window_width, settings.window_height])
            .with_min_inner_size([480., 360.])
            .with_decorations(false)
            .with_resizable(true)
            .with_icon(icon),
        ..Default::default()
    };
    eframe::run_native(
        "Anemorph",
        options,
        Box::new(move |cc| Ok(Box::new(App::new(cc, smoke)))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn app() -> App {
        let cc = eframe::CreationContext::_new_kittest(Context::default());
        App::new(
            &cc,
            Some(std::env::temp_dir().join(format!("anemorph-ui-{}", std::process::id()))),
        )
    }
    #[test]
    fn large_preview_runs_off_thread_and_discards_superseded_results() {
        let mut a = app();
        let root = std::env::temp_dir().join("anemorph-background-preview");
        a.root = Some(root.clone());
        a.files = Arc::new(
            (1..=2000)
                .map(|i| root.join(format!("Raw S01E{:03}.mkv", i % 400 + 1)))
                .collect(),
        );
        a.series = "Before".into();
        a.rebuild();
        assert!(a.preview.is_some() && a.dirty);
        a.series = "After".into();
        a.rebuild();
        let deadline = Instant::now() + Duration::from_secs(10);
        let ctx = a.context.clone();
        while a.preview.is_some() && Instant::now() < deadline {
            a.poll(&ctx);
            std::thread::sleep(Duration::from_millis(2));
        }
        assert!(!a.dirty && a.preview.is_none());
        assert!(a.rows.iter().all(|r| r.output.starts_with("After ")));
        a.series = "Discard".into();
        a.rebuild();
        a.folder.clear();
        a.folder_edited(&ctx);
        while a.preview.is_some() && Instant::now() < deadline {
            a.poll(&ctx);
            std::thread::sleep(Duration::from_millis(2));
        }
        assert!(a.rows.is_empty() && a.root.is_none());
    }
    #[test]
    fn selection_ranges_and_groups() {
        let mut a = app();
        let root = std::env::temp_dir().join("anemorph-selection");
        let files: Vec<_> = (1..=8)
            .map(|n| root.join(format!("Old S02E{n:02}.mkv")))
            .collect();
        a.rows = engine::plan(&root, "Show", &files, &a.settings).unwrap();
        a.select(1, true, Modifiers::NONE);
        a.select(
            5,
            true,
            Modifiers {
                shift: true,
                ..Modifiers::NONE
            },
        );
        assert!(a.rows[1..=5].iter().all(|r| !r.included));
        assert!(a.rows[0].included && a.rows[6].included);
        a.mark(Some(true));
        assert!(a.rows.iter().all(|r| r.included));
        a.select(2, false, Modifiers::NONE);
        a.select(
            4,
            false,
            Modifiers {
                ctrl: true,
                ..Modifiers::NONE
            },
        );
        assert_eq!(a.rows.iter().filter(|r| r.highlighted).count(), 2);
        a.mark(Some(false));
        assert!(!a.rows[2].included && !a.rows[4].included && a.rows[3].included);
    }
    #[test]
    fn refresh_preserves_cached_files() {
        let mut a = app();
        let root = std::env::temp_dir().join("anemorph-cached");
        a.root = Some(root.clone());
        a.files = Arc::new(vec![root.join("Old - 13.mkv")]);
        a.series = "Kuroko no Basket".into();
        a.dirty = true;
        a.rebuild();
        assert_eq!(a.rows[0].output, "Kuroko no Basket S01E13.mkv");
        assert!(!a.dirty);
        a.series.clear();
        a.rebuild();
        assert_eq!(a.rows.len(), 1);
        assert_eq!(a.rows[0].original, "Old - 13.mkv");
        assert!(a.dirty && !a.rows[0].ready && a.rows[0].target.is_none());
        assert_eq!(a.root.as_ref(), Some(&root));
        a.refresh_series();
        assert!(!a.series.is_empty() && !a.dirty);
        assert_eq!(a.rows.len(), 1);
        a.series = "My custom title".into();
        a.refresh_series();
        assert_eq!(a.rows[0].output, "My custom title S01E13.mkv");
    }
    #[test]
    fn hex_and_foreground() {
        assert_eq!(parse_hex("#FF0000"), Some(Color32::RED));
        assert_eq!(parse_hex("abc"), Some(Color32::from_rgb(170, 187, 204)));
        assert!(parse_hex("invalid").is_none());
        assert_eq!(on_color(Color32::BLACK), Color32::WHITE);
        assert_eq!(on_color(Color32::WHITE), Color32::from_gray(16));
    }
    #[test]
    fn any_checked_file_changes_the_global_selection_action() {
        let mut a = app();
        let root = std::env::temp_dir().join("anemorph-select-toggle");
        a.rows = engine::plan(
            &root,
            "Show",
            &[
                root.join("E01.mkv"),
                root.join("E02.mkv"),
                root.join("Movie.mkv"),
            ],
            &a.settings,
        )
        .unwrap();
        assert!(a.any_file_selected());
        a.toggle_all_files();
        assert!(!a.any_file_selected());
        a.rows[0].included = true;
        assert!(a.any_file_selected());
        a.toggle_all_files();
        assert!(!a.any_file_selected());
        a.rows[1].highlighted = true;
        assert!(a.any_file_selected());
        a.toggle_all_files();
        assert!(!a.any_file_selected());
        a.toggle_all_files();
        assert!(a.rows[0].included && a.rows[1].included);
        assert!(!a.rows[2].included && !a.rows[2].highlighted);
    }
    #[test]
    fn clearing_suggested_name_obeys_saved_preference() {
        let mut a = app();
        let ctx = Context::default();
        assert!(a.settings.clear_series_on_empty);
        a.series = "Suggested".into();
        a.series_suggested = true;
        a.folder.clear();
        a.folder_edited(&ctx);
        assert!(a.series.is_empty());
        a.settings.clear_series_on_empty = false;
        a.series = "Suggested".into();
        a.series_suggested = true;
        a.folder_edited(&ctx);
        assert_eq!(a.series, "Suggested");
        let restored: Settings =
            serde_json::from_str(&serde_json::to_string(&a.settings).unwrap()).unwrap();
        assert!(!restored.clear_series_on_empty);
        a.settings.clear_series_on_empty = true;
        a.series = "My own name".into();
        a.series_suggested = false;
        a.folder_edited(&ctx);
        assert_eq!(a.series, "My own name");
    }
    #[test]
    fn typed_folder_loads_without_enter() {
        let mut a = app();
        let ctx = Context::default();
        a.folder = "  \"\"  ".into();
        a.load_typed_folder(&ctx);
        assert!(!a.busy && !a.error);
        let root = std::env::temp_dir().join(format!("anemorph-typed-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("Show S01E01.mkv"), b"fixture").unwrap();
        a.folder = format!("  \"{}\"  ", root.display());
        a.load_typed_folder(&ctx);
        let deadline = Instant::now() + Duration::from_secs(5);
        while a.busy && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(5));
            a.poll(&ctx);
        }
        assert!(!a.busy && !a.error);
        assert_eq!(a.root.as_ref(), Some(&root));
        assert_eq!(a.rows.len(), 1);
        std::fs::remove_file(root.join("Show S01E01.mkv")).unwrap();
        std::fs::remove_dir(root).unwrap();
    }
    #[test]
    fn transition_stops_requesting_frames() {
        let mut a = app();
        let ctx = Context::default();
        a.navigate(Page::Settings, &ctx);
        assert!(a.progress() < 1.);
        a.transition = Instant::now() - Duration::from_millis(250);
        assert_eq!(a.progress(), 1.);
        a.settings.animations = false;
        a.navigate(Page::Home, &ctx);
        assert_eq!(a.progress(), 1.);
    }
}
