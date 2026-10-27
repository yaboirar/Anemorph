#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod containers;
mod elevation;
mod engine;
mod i18n;
mod integration;
mod naming;
mod perf;
mod selection;
mod ui;
mod uninstall;
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
enum StartupSelection {
    Request(PathBuf),
    Files(Vec<PathBuf>),
}
enum Work {
    Selected(Result<Vec<PathBuf>, String>),
    Scanned(Result<Vec<PathBuf>, String>),
    Changed(Result<Vec<Change>, String>, bool),
}
struct Smoke {
    dir: PathBuf,
    stage: usize,
    last: Instant,
    snapshots: usize,
    idle_start: u64,
    idle_frames: Option<u64>,
    native_restore_verified: bool,
}
struct EpisodeEntry {
    path: PathBuf,
    numbering: Option<(u32, u32)>,
}
struct EpisodeMarkDialog {
    entries: Vec<EpisodeEntry>,
    unknown_count: usize,
    season: u32,
    first_episode: u32,
    format: naming::Format,
    error: Option<String>,
}
struct App {
    settings: Settings,
    data: PathBuf,
    folder: String,
    series: String,
    series_suggested: bool,
    format_draft: String,
    movie_format_draft: String,
    movie_format_cache: Option<(String, Result<naming::MovieFormat, String>)>,
    format_cache: Option<(String, Result<naming::Format, String>)>,
    about_open: bool,
    uninstaller: Option<PathBuf>,
    uninstall_error: String,
    integration: integration::Integration,
    episode_dialog: Option<EpisodeMarkDialog>,
    native_window: Option<usize>,
    files: Arc<Vec<PathBuf>>,
    selected_files: Option<Arc<Vec<PathBuf>>>,
    media_markers: Arc<engine::MediaMarkers>,
    keep_selection: bool,
    completed: std::collections::HashSet<PathBuf>,
    container_anchor: Option<(usize, bool)>,
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
    rescan_pending: bool,
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
    column_drag: Option<(usize, f32)>,
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
    #[cfg(test)]
    fn new(cc: &eframe::CreationContext<'_>, smoke: Option<PathBuf>) -> Self {
        Self::initialize(cc, smoke, true)
    }
    fn initialize(
        cc: &eframe::CreationContext<'_>,
        smoke: Option<PathBuf>,
        restore_folder: bool,
    ) -> Self {
        round_window_corners(cc);
        if let Some(dir) = &smoke
            && let Some(render) = &cc.wgpu_render_state
        {
            let info = render.adapter.get_info();
            let _ = std::fs::write(dir.join("graphics.json"), serde_json::json!({"adapter":info.name,"type":format!("{:?}",info.device_type),"backend":format!("{:?}",info.backend)}).to_string());
        }
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};
        let native_window = cc.window_handle().ok().and_then(|h| match h.as_raw() {
            RawWindowHandle::Win32(w) => Some(w.hwnd.get() as usize),
            _ => None,
        });
        let data = smoke
            .as_ref()
            .map(|p| p.join("settings"))
            .unwrap_or_else(engine::data_dir);
        let settings = Settings::load(&data.join("settings.json"));
        i18n::configure(&cc.egui_ctx, &settings.language);
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
        let movie_format_draft = settings.movie_format.clone();
        let mut app = Self {
            settings,
            data: data.clone(),
            folder: String::new(),
            series: String::new(),
            series_suggested: false,
            movie_format_draft,
            movie_format_cache: None,
            format_draft,
            format_cache: None,
            about_open: false,
            uninstaller: if smoke.is_none() {
                uninstall::installed()
            } else {
                None
            },
            uninstall_error: String::new(),
            integration: integration::Integration::default(),
            episode_dialog: None,
            native_window,
            files: Arc::new(vec![]),
            selected_files: None,
            media_markers: Arc::new(engine::load_media_markers(&data.join("media-markers.json"))),
            keep_selection: false,
            completed: std::collections::HashSet::new(),
            container_anchor: None,
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
            rescan_pending: false,
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
            column_drag: None,
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
                idle_frames: None,
                native_restore_verified: false,
            }),
            frames: 0,
            ui_micros: 0,
            max_visible_rows: 0,
        };
        app.theme(&cc.egui_ctx);
        if let Some(p) = startup.filter(|p| restore_folder && p.is_dir()) {
            app.open(p, true, &cc.egui_ctx)
        }
        app
    }
    fn repair(&mut self, ctx: &Context) {
        if self.busy || self.preview.is_some() || self.integration.pending {
            return;
        }
        self.settings = Settings::load(&self.data.join("settings.json"));
        i18n::configure(ctx, &self.settings.language);
        self.theme(ctx);
        self.format_draft = self.settings.number_format.clone();
        self.movie_format_draft = self.settings.movie_format.clone();
        self.format_cache = None;
        self.movie_format_cache = None;
        self.draft_color = self
            .settings
            .accent_hex
            .as_deref()
            .and_then(parse_hex)
            .unwrap_or(DEFAULT_ACCENT);
        self.hex = hex(self.draft_color);
        self.picker_hue = egui::ecolor::Hsva::from(self.draft_color).h;
        self.color_message.clear();
        self.uninstall_error.clear();
        self.error = false;
        self.anchor = None;
        self.check_anchor = None;
        self.column_drag = None;
        self.container_anchor = None;
        self.episode_dialog = None;
        if let Some(root) = self.root.clone() {
            self.open(root, false, ctx);
        }
        if let Some(root) = self.uninstaller.as_ref().and_then(|p| p.parent()) {
            self.integration.repair(root.to_path_buf());
        }
        ctx.request_repaint();
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
            if !infer && self.root.as_ref() == Some(&root) {
                self.rescan_pending = true;
            }
            return;
        }
        self.rescan_pending = false;
        if !root.is_dir() {
            self.status = "Choose an existing folder first.".into();
            self.error = true;
            return;
        }
        self.folder = root.to_string_lossy().into();
        if infer || self.root.as_ref() != Some(&root) {
            self.selected_files = None;
        }
        if self.root.as_ref() != Some(&root) {
            self.completed.clear();
        }
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
        let selected = self.selected_files.clone();
        let (tx, rx) = mpsc::channel();
        let context = ctx.clone();
        self.receiver = Some(rx);
        self.busy = true;
        self.status = "Reading video files…".into();
        self.error = false;
        std::thread::spawn(move || {
            let result = if let Some(selected) = selected {
                Ok(selected
                    .iter()
                    .filter(|p| {
                        p.is_file()
                            && p.extension().and_then(|e| e.to_str()).is_some_and(|ext| {
                                settings.container_enabled(&ext.to_ascii_lowercase())
                            })
                    })
                    .cloned()
                    .collect())
            } else {
                engine::scan(&root, &settings)
            };
            let _ = tx.send(Work::Scanned(result));
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
    fn request_selection(&mut self, selection: StartupSelection, ctx: &Context) {
        let (tx, rx) = mpsc::channel();
        let context = ctx.clone();
        self.receiver = Some(rx);
        self.busy = true;
        self.status = "Loading selected videos...".into();
        std::thread::spawn(move || {
            let result = match selection {
                StartupSelection::Request(path) => {
                    selection::read_request(&path, &selection::directory())
                }
                StartupSelection::Files(files) => selection::validate(files),
            };
            let _ = tx.send(Work::Selected(result));
            context.request_repaint();
        });
    }
    fn open_selection(&mut self, files: Vec<PathBuf>, ctx: &Context) {
        let Some(root) = selection::common_folder(&files) else {
            self.status = "Select video files from the same drive.".into();
            self.error = true;
            return;
        };
        if self.settings.auto_name {
            self.series = engine::guess_name(&root);
            self.series_suggested = true;
        }
        self.settings.filter_extensions.clear();
        self.root = Some(root.clone());
        self.selected_files = Some(Arc::new(files));
        self.open(root, false, ctx);
    }
    fn folder_edited(&mut self, ctx: &Context) {
        self.selected_files = None;
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
            row.included = select;
            row.highlighted = select;
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
            let markers = Arc::clone(&self.media_markers);
            self.preview = Some(rx);
            std::thread::spawn(move || {
                let _ = tx.send(engine::plan_with_markers(
                    &root, &name, &files, &settings, &markers,
                ));
                context.request_repaint();
            });
            return;
        }
        let result = engine::plan_with_markers(
            root,
            &self.series,
            &self.files,
            &self.settings,
            &self.media_markers,
        );
        self.install_preview(result);
    }
    fn install_preview(&mut self, result: Result<Vec<Row>, String>) {
        self.counts.set(None);
        let Some(root) = &self.root else { return };
        match result {
            Ok(mut rows) => {
                for row in &mut rows {
                    if row.status == "No change"
                        && self
                            .completed
                            .contains(&engine::media_marker_key(&row.source))
                    {
                        row.status = "Done".into();
                    }
                }
                if std::mem::take(&mut self.keep_selection) {
                    let previous: std::collections::HashMap<_, _> = self
                        .rows
                        .iter()
                        .map(|r| (&r.source, (r.included, r.highlighted, r.ready)))
                        .collect();
                    for row in &mut rows {
                        if let Some((included, highlighted, was_ready)) = previous.get(&row.source)
                        {
                            row.highlighted = *highlighted;
                            if *was_ready {
                                row.included = *included && row.ready;
                            }
                        }
                    }
                }
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
                        kind: engine::marked_media_kind(source, &self.media_markers),
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
    fn marker_paths(&self, index: usize) -> Option<Vec<PathBuf>> {
        if self.busy {
            return None;
        }
        let row = self.rows.get(index)?;
        Some(if row.highlighted {
            self.rows
                .iter()
                .filter(|r| r.highlighted)
                .map(|r| r.source.clone())
                .collect()
        } else {
            vec![row.source.clone()]
        })
    }
    fn store_media_markers(&mut self, markers: engine::MediaMarkers) -> Result<(), String> {
        if markers != *self.media_markers {
            engine::save_media_markers(&self.data.join("media-markers.json"), &markers)?;
        }
        self.media_markers = Arc::new(markers);
        self.keep_selection = true;
        self.rebuild();
        Ok(())
    }
    fn set_media_marker(&mut self, index: usize, kind: Option<engine::MediaKind>) {
        if kind == Some(engine::MediaKind::Episode) {
            self.request_episode_marker(index);
            return;
        }
        let Some(paths) = self.marker_paths(index) else {
            return;
        };
        let mut markers = (*self.media_markers).clone();
        for path in paths {
            if let Some(kind) = kind {
                let key = engine::media_marker_key(&path);
                let mut marker = markers.get(&key).copied().unwrap_or_else(|| kind.into());
                marker.kind = kind;
                markers.insert(key, marker);
            } else {
                markers.remove(&engine::media_marker_key(&path));
            }
        }
        if let Err(error) = self.store_media_markers(markers) {
            self.status = format!("Could not save media marker: {error}");
            self.error = true;
        }
    }
    fn request_episode_marker(&mut self, index: usize) {
        let Some(paths) = self.marker_paths(index) else {
            return;
        };
        let format = match naming::Format::parse(&self.settings.number_format) {
            Ok(format) => format,
            Err(error) => {
                self.status = error;
                self.error = true;
                return;
            }
        };
        let entries: Vec<_> = paths
            .into_iter()
            .map(|path| {
                let numbering = self
                    .media_markers
                    .get(&engine::media_marker_key(&path))
                    .and_then(|m| m.season.zip(m.episode))
                    .or_else(|| {
                        engine::recognized_episode(&path, &format, self.settings.default_season)
                    });
                EpisodeEntry { path, numbering }
            })
            .collect();
        let unknown_count = entries.iter().filter(|e| e.numbering.is_none()).count();
        if unknown_count == 0 {
            let mut markers = (*self.media_markers).clone();
            for entry in entries {
                let (season, episode) = entry.numbering.unwrap();
                markers.insert(
                    engine::media_marker_key(&entry.path),
                    engine::MediaOverride::episode(season, episode),
                );
            }
            if let Err(error) = self.store_media_markers(markers) {
                self.status = format!("Could not save episode marker: {error}");
                self.error = true;
            }
        } else {
            self.episode_dialog = Some(EpisodeMarkDialog {
                entries,
                unknown_count,
                season: self.settings.default_season,
                first_episode: self.settings.first_episode,
                format,
                error: None,
            });
        }
    }
    fn apply_episode_marker(&mut self) {
        let Some(mut dialog) = self.episode_dialog.take() else {
            return;
        };
        let valid_count = u32::try_from(dialog.unknown_count).ok();
        if valid_count
            .and_then(|count| dialog.first_episode.checked_add(count.saturating_sub(1)))
            .is_none()
        {
            dialog.error = Some("Episode number exceeds the supported range.".into());
            self.episode_dialog = Some(dialog);
            return;
        }
        let mut markers = (*self.media_markers).clone();
        let mut assigned = 0_u32;
        for entry in &dialog.entries {
            let (season, episode) = if let Some(numbers) = entry.numbering {
                numbers
            } else {
                // The complete range was checked before mutating the draft map.
                let episode = dialog.first_episode + assigned;
                assigned += 1;
                (dialog.season, episode)
            };
            markers.insert(
                engine::media_marker_key(&entry.path),
                engine::MediaOverride::episode(season, episode),
            );
        }
        if let Err(error) = self.store_media_markers(markers) {
            dialog.error = Some(error);
            self.episode_dialog = Some(dialog);
        }
    }
    fn refresh_series(&mut self) {
        if let Some(root) = &self.root {
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
            if self.rescan_pending && matches!(&result, Work::Scanned(_)) {
                self.rescan_pending = false;
                if let Some(root) = self.root.clone() {
                    self.open(root, false, ctx);
                    return;
                }
            }
            match result {
                Work::Selected(Ok(files)) => self.open_selection(files, ctx),
                Work::Selected(Err(error)) => {
                    self.status = error;
                    self.error = true;
                }
                Work::Scanned(Ok(files)) => {
                    self.files = Arc::new(files);
                    self.preview_again = self.preview.is_some();
                    self.rebuild()
                }
                Work::Scanned(Err(e)) => {
                    self.status = e;
                    self.error = true
                }
                Work::Changed(Ok(changes), undo) => {
                    let n = changes.len();
                    if let Some(files) = &mut self.selected_files {
                        let moved: std::collections::HashMap<_, _> = changes
                            .iter()
                            .map(|c| (engine::media_marker_key(&c.source), c.target.clone()))
                            .collect();
                        for path in Arc::make_mut(files) {
                            if let Some(target) = moved.get(&engine::media_marker_key(path)) {
                                *path = target.clone();
                            }
                        }
                    }
                    self.completed.clear();
                    if !undo {
                        self.completed.extend(
                            changes
                                .iter()
                                .map(|change| engine::media_marker_key(&change.target)),
                        );
                    }
                    let mut markers = (*self.media_markers).clone();
                    engine::move_media_markers(&mut markers, &changes);
                    let marker_save = if markers != *self.media_markers {
                        engine::save_media_markers(&self.data.join("media-markers.json"), &markers)
                    } else {
                        Ok(())
                    };
                    self.media_markers = Arc::new(markers);
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
                    self.status = message;
                    if let Err(error) = marker_save {
                        self.status += &format!(" Could not save media markers: {error}");
                        self.error = true;
                    }
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
                engine::undo_with_changes(&history)
            } else {
                engine::rename_batch(&changes, &history).map(|_| changes)
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
        let highlighted = self.rows.iter().any(|r| r.highlighted);
        let mark = value.unwrap_or_else(|| {
            self.rows
                .iter()
                .any(|r| (r.highlighted || !highlighted) && !r.included)
        });
        for r in self
            .rows
            .iter_mut()
            .filter(|r| r.highlighted || !highlighted)
        {
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
                    if r.highlighted {
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
                self.rows[index].included = mark;
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
    let selection = if args.get(1).is_some_and(|s| s == "--selection") {
        Some(StartupSelection::Request(
            args.get(2).map(PathBuf::from).unwrap_or_default(),
        ))
    } else if args.get(1).is_some_and(|s| s == "--files") {
        Some(StartupSelection::Files(
            args.iter().skip(2).map(PathBuf::from).collect(),
        ))
    } else {
        None
    };
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
        setup.power_preference = wgpu::PowerPreference::LowPower;
        let software = args.iter().any(|a| a == "--software-rendering");
        setup.native_adapter_selector = Some(Arc::new(move |adapters, surface| {
            adapters
                .iter()
                .filter(|a| surface.is_none_or(|s| a.is_surface_supported(s)))
                .filter(|a| !software || a.get_info().device_type == wgpu::DeviceType::Cpu)
                .min_by_key(|a| match a.get_info().device_type {
                    wgpu::DeviceType::IntegratedGpu => 0,
                    wgpu::DeviceType::DiscreteGpu => 1,
                    wgpu::DeviceType::VirtualGpu => 2,
                    wgpu::DeviceType::Cpu => 3,
                    _ => 4,
                })
                .cloned()
                .ok_or_else(|| {
                    "No compatible graphics adapter. Update your display driver or try another PC."
                        .into()
                })
        }));
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
        Box::new(move |cc| {
            let mut app = App::initialize(cc, smoke, selection.is_none());
            if let Some(selection) = selection {
                app.request_selection(selection, &cc.egui_ctx);
            }
            Ok(Box::new(app))
        }),
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
    fn unchanged_files_can_be_included_but_are_never_rename_operations() {
        let mut a = app();
        let root = std::env::temp_dir().join("anemorph-unchanged-selection");
        a.root = Some(root.clone());
        a.series = "Show".into();
        a.files = Arc::new(vec![
            root.join("Show S01E09.mkv"),
            root.join("Show S01E19.mp4"),
        ]);
        a.rebuild();
        assert!(a.rows.iter().all(|r| r.status == "No change" && !r.ready));
        a.mark(Some(true));
        assert!(a.rows.iter().all(|r| r.included));
        assert_eq!(a.row_counts().1, 0);
        a.mark(Some(false));
        a.select(0, true, Modifiers::NONE);
        assert!(a.rows[0].included);
        a.select(1, true, Modifiers::SHIFT);
        assert!(a.rows.iter().all(|r| r.included));
        a.toggle_all_files();
        assert!(a.rows.iter().all(|r| !r.included));
        assert!(a.rows.iter().all(|r| !r.ready));
    }
    #[test]
    fn repair_reloads_saved_preferences_without_deleting_user_data() {
        let mut a = app();
        let root = std::env::temp_dir().join(format!("anemorph-recovery-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        a.data = root.clone();
        let saved = Settings {
            animations: false,
            ..Default::default()
        };
        saved.save(&root.join("settings.json")).unwrap();
        std::fs::write(root.join("native-last-batch.json"), b"history fixture").unwrap();
        a.format_draft = "unsaved draft".into();
        a.error = true;
        a.repair(&a.context.clone());
        assert!(!a.settings.animations && !a.error);
        assert_eq!(a.format_draft, saved.number_format);
        assert_eq!(
            std::fs::read(root.join("native-last-batch.json")).unwrap(),
            b"history fixture"
        );
        std::fs::remove_dir_all(root).unwrap();
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
    fn marking_movie_preserves_excluded_rows_and_updates_preview() {
        let mut a = app();
        let root = std::env::temp_dir().join(format!("anemorph-marker-ui-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        a.data = root.join("data");
        a.media_markers = Arc::new(engine::MediaMarkers::new());
        a.root = Some(root.clone());
        a.series = "Show".into();
        a.files = Arc::new(vec![
            root.join("Mugen Train (2020) [1080p].mp4"),
            root.join("Original S01E19.mp4"),
        ]);
        a.rebuild();
        assert!(!a.rows[0].ready);
        a.rows[1].included = false;
        a.set_media_marker(0, Some(engine::MediaKind::Movie));
        assert_eq!(a.rows[0].kind, engine::MediaKind::Movie);
        assert_eq!(a.rows[0].output, "Mugen Train (2020) Movie.mp4");
        assert!(!a.rows[1].included);
        a.rebuild();
        assert_eq!(a.rows[0].kind, engine::MediaKind::Movie);
        a.set_media_marker(0, None);
        assert!(!a.rows[0].ready && a.rows[0].target.is_none());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn marking_numbered_movie_as_episode_keeps_existing_numbers() {
        let mut a = app();
        let root =
            std::env::temp_dir().join(format!("anemorph-numbered-episode-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        a.data = root.join("data");
        a.root = Some(root.clone());
        a.series = "Show".into();
        a.settings.auto_detect = false;
        a.media_markers = Arc::new(engine::MediaMarkers::new());
        a.files = Arc::new(vec![root.join("Show Movie S03E23.mp4")]);
        a.rebuild();
        assert_eq!(a.rows[0].kind, engine::MediaKind::Movie);
        a.set_media_marker(0, Some(engine::MediaKind::Episode));
        assert!(a.episode_dialog.is_none());
        assert_eq!(a.rows[0].kind, engine::MediaKind::Episode);
        assert_eq!(a.rows[0].output, "Show S03E23.mp4");
        a.rebuild();
        assert_eq!(a.rows[0].output, "Show S03E23.mp4");
        a.set_media_marker(0, Some(engine::MediaKind::Movie));
        a.set_media_marker(0, Some(engine::MediaKind::Episode));
        assert!(a.episode_dialog.is_none());
        assert_eq!(a.rows[0].output, "Show S03E23.mp4");
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn batch_episode_marking_numbers_only_unknown_files_and_cancel_changes_nothing() {
        let mut a = app();
        let root =
            std::env::temp_dir().join(format!("anemorph-batch-episode-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        a.data = root.join("data");
        a.root = Some(root.clone());
        a.series = "Show".into();
        a.settings.auto_detect = false;
        a.media_markers = Arc::new(engine::MediaMarkers::new());
        a.files = Arc::new(vec![
            root.join("Clip alpha.mp4"),
            root.join("Show S02E19.mp4"),
            root.join("Clip beta.mp4"),
        ]);
        a.rebuild();
        for row in &mut a.rows {
            row.highlighted = true;
        }
        a.set_media_marker(0, Some(engine::MediaKind::Episode));
        assert_eq!(a.episode_dialog.as_ref().unwrap().unknown_count, 2);
        assert!(a.media_markers.is_empty());
        a.episode_dialog = None;
        assert!(a.media_markers.is_empty() && !a.rows[0].ready);
        a.set_media_marker(0, Some(engine::MediaKind::Episode));
        let dialog = a.episode_dialog.as_mut().unwrap();
        dialog.season = 2;
        dialog.first_episode = 7;
        a.apply_episode_marker();
        assert!(a.episode_dialog.is_none());
        assert_eq!(a.rows[0].output, "Show S02E07.mp4");
        assert_eq!(a.rows[1].output, "Show S02E19.mp4");
        assert_eq!(a.rows[2].output, "Show S02E08.mp4");
        assert_eq!(a.rows[1].status, "No change");
        a.rebuild();
        assert_eq!(a.rows[2].output, "Show S02E08.mp4");
        std::fs::remove_dir_all(root).unwrap();
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
    fn rapid_filter_changes_rescan_with_the_latest_choices() {
        let mut a = app();
        let root =
            std::env::temp_dir().join(format!("anemorph-filter-rescan-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let mkv = root.join("Show S01E01.mkv");
        let mp4 = root.join("Show S01E02.mp4");
        for path in [&mkv, &mp4] {
            std::fs::write(path, b"video").unwrap();
        }
        a.root = Some(root.clone());
        a.series = "Show".into();
        a.busy = true;
        let (tx, rx) = mpsc::channel();
        a.receiver = Some(rx);
        a.settings.toggle_container("mkv");
        let ctx = a.context.clone();
        a.open(root.clone(), false, &ctx);
        a.settings.toggle_container("mp4");
        a.open(root.clone(), false, &ctx);
        assert!(a.rescan_pending);
        tx.send(Work::Scanned(Ok(vec![mkv.clone()]))).unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        while a.busy && Instant::now() < deadline {
            a.poll(&ctx);
            std::thread::sleep(Duration::from_millis(2));
        }
        assert!(!a.busy && !a.rescan_pending);
        assert_eq!(a.files.as_ref(), &vec![mkv, mp4]);
        std::fs::remove_dir_all(root).unwrap();
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
        assert_eq!(a.series, engine::guess_name(&root));
        assert!(a.series_suggested);
        assert_eq!(a.rows[0].output, format!("{} S01E13.mkv", a.series));
    }
    #[test]
    fn completed_rename_stays_done_on_refresh_and_clears_on_undo() {
        let mut a = app();
        let root = std::env::temp_dir().join(format!("anemorph-done-ui-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        a.data = root.join("data");
        a.settings.remember_folder = false;
        a.root = Some(root.clone());
        a.series = "Show".into();
        let source = root.join("Old S01E19.mkv");
        let unchanged = root.join("Show S01E20.mkv");
        std::fs::write(&source, b"video").unwrap();
        std::fs::write(&unchanged, b"video").unwrap();
        a.files = Arc::new(vec![source.clone(), unchanged]);
        a.rebuild();
        let ctx = a.context.clone();
        a.execute(false, &ctx);
        let wait = |a: &mut App| {
            let deadline = Instant::now() + Duration::from_secs(10);
            while (a.busy || a.preview.is_some()) && Instant::now() < deadline {
                a.poll(&ctx);
                std::thread::sleep(Duration::from_millis(2));
            }
            assert!(!a.busy && !a.dirty);
        };
        wait(&mut a);
        assert_eq!(a.rows[0].status, "Done");
        assert!(!a.rows[0].ready && !a.rows[0].included);
        assert_eq!(a.rows[1].status, "No change");
        a.open(root.clone(), false, &ctx);
        wait(&mut a);
        assert_eq!(a.rows[0].status, "Done");
        a.execute(true, &ctx);
        wait(&mut a);
        assert!(a.completed.is_empty());
        assert!(source.is_file());
        assert!(a.rows.iter().all(|r| r.status != "Done"));
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn explorer_selection_stays_scoped_through_refresh_rename_and_undo() {
        let mut a = app();
        let root = std::env::temp_dir().join(format!("anemorph-scoped-ui-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        a.data = root.join("data");
        a.settings.remember_folder = false;
        let selected = root.join("Old S01E19.mkv");
        let untouched = root.join("Old S01E20.mp4");
        std::fs::write(&selected, b"selected video").unwrap();
        std::fs::write(&untouched, b"untouched video").unwrap();
        let ctx = a.context.clone();
        a.open_selection(vec![selected.clone()], &ctx);
        let wait = |a: &mut App| {
            let deadline = Instant::now() + Duration::from_secs(10);
            while (a.busy || a.preview.is_some()) && Instant::now() < deadline {
                a.poll(&ctx);
                std::thread::sleep(Duration::from_millis(2));
            }
            assert!(!a.busy);
        };
        wait(&mut a);
        assert_eq!(a.rows.len(), 1);
        a.series = "Show".into();
        a.rebuild();
        a.execute(false, &ctx);
        wait(&mut a);
        assert_eq!(a.rows.len(), 1);
        assert_eq!(a.rows[0].source, root.join("Show S01E19.mkv"));
        assert_eq!(a.rows[0].status, "Done");
        assert!(untouched.is_file());
        a.open(root.clone(), false, &ctx);
        wait(&mut a);
        assert_eq!(a.rows.len(), 1);
        a.execute(true, &ctx);
        wait(&mut a);
        assert_eq!(a.rows[0].source, selected);
        assert_eq!(a.rows.len(), 1);
        assert!(untouched.is_file());
        a.open(root.clone(), true, &ctx);
        wait(&mut a);
        assert!(a.selected_files.is_none());
        assert_eq!(a.rows.len(), 2);
        std::fs::remove_dir_all(root).unwrap();
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
        assert!(a.rows[2].included && a.rows[2].highlighted);
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
