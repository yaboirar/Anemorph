use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::LazyLock,
    sync::atomic::{AtomicU64, Ordering},
};

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
#[serde(default, rename_all = "PascalCase")]
pub struct Settings {
    pub language: String,
    pub dark_mode: bool,
    pub compact: bool,
    pub auto_detect: bool,
    pub manual_numbering: bool,
    pub auto_name: bool,
    pub clear_series_on_empty: bool,
    pub number_format: String,
    pub movie_format: String,
    pub subfolders: bool,
    pub filter_extensions: Vec<String>,
    pub home_filter_options: Vec<String>,
    #[serde(skip_serializing)]
    pub filter_extension: Option<String>,
    pub remember_folder: bool,
    pub animations: bool,
    pub default_season: u32,
    pub first_episode: u32,
    pub zoom_percent: u32,
    pub accent_hex: Option<String>,
    pub last_folder: Option<PathBuf>,
    pub table_name_ratio: f32,
    pub table_status_width: f32,
    pub window_width: f32,
    pub window_height: f32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            language: "en-US".into(),
            dark_mode: true,
            compact: false,
            auto_detect: true,
            manual_numbering: false,
            auto_name: true,
            clear_series_on_empty: true,
            number_format: crate::naming::DEFAULT_FORMAT.into(),
            movie_format: crate::naming::DEFAULT_MOVIE_FORMAT.into(),
            subfolders: true,
            filter_extensions: Vec::new(),
            home_filter_options: ["mkv", "mp4", "mov", "avi", "webm", "m4v"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            filter_extension: None,
            remember_folder: false,
            animations: true,
            default_season: 1,
            first_episode: 1,
            zoom_percent: 100,
            accent_hex: None,
            last_folder: None,
            table_name_ratio: 0.5,
            table_status_width: 116.,
            window_width: 1080.,
            window_height: 740.,
        }
    }
}
impl Settings {
    pub fn container_enabled(&self, extension: &str) -> bool {
        VIDEO_CONTAINERS
            .iter()
            .any(|(ext, _)| extension.eq_ignore_ascii_case(ext))
            && (self.filter_extensions.is_empty() || self.container_selected(extension))
    }
    pub fn container_selected(&self, extension: &str) -> bool {
        self.filter_extensions
            .iter()
            .any(|selected| extension.eq_ignore_ascii_case(selected))
    }
    pub fn container_visible(&self, extension: &str) -> bool {
        self.home_filter_options
            .iter()
            .any(|visible| visible.eq_ignore_ascii_case(extension))
    }
    pub fn toggle_container_visibility(&mut self, extension: &str) {
        if self.container_visible(extension) {
            self.home_filter_options.retain(|value| value != extension);
            self.filter_extensions.retain(|value| value != extension);
        } else if VIDEO_CONTAINERS
            .iter()
            .any(|(value, _)| *value == extension)
        {
            self.home_filter_options.push(extension.to_owned());
        }
    }
    pub fn set_container_selected(&mut self, extension: &str, selected: bool) {
        let extension = extension.to_ascii_lowercase();
        if !VIDEO_CONTAINERS.iter().any(|(ext, _)| *ext == extension) {
            return;
        }
        if selected && !self.container_visible(&extension) {
            return;
        }
        self.filter_extensions.retain(|ext| ext != &extension);
        if selected {
            self.filter_extensions.push(extension);
        }
        self.filter_extensions.sort_unstable();
    }
    pub fn toggle_container(&mut self, extension: &str) {
        self.set_container_selected(extension, !self.container_selected(extension));
    }
    pub fn toggle_all_containers(&mut self) {
        self.filter_extensions = if self.filter_extensions.len() == self.home_filter_options.len() {
            Vec::new()
        } else {
            self.home_filter_options.clone()
        };
        self.filter_extensions.sort_unstable();
    }
    pub fn container_mask(&self) -> u32 {
        VIDEO_CONTAINERS
            .iter()
            .enumerate()
            .fold(0, |mask, (i, (extension, _))| {
                mask | (u32::from(self.container_selected(extension)) << i)
            })
    }

    pub fn load(path: &Path) -> Self {
        let mut s: Self = read_bounded(path, 64 * 1024)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        if s.filter_extensions.is_empty()
            && let Some(legacy) = s.filter_extension.take()
        {
            s.filter_extensions.push(legacy);
        }
        s.filter_extension = None;
        if !crate::i18n::LANGUAGES
            .iter()
            .any(|(code, _)| *code == s.language)
        {
            s.language = "en-US".into();
        }
        for extension in &mut s.filter_extensions {
            extension.make_ascii_lowercase();
        }
        s.filter_extensions
            .retain(|e| VIDEO_CONTAINERS.iter().any(|(ext, _)| e == ext));
        s.filter_extensions.sort_unstable();
        s.filter_extensions.dedup();
        for extension in &mut s.home_filter_options {
            extension.make_ascii_lowercase();
        }
        s.home_filter_options
            .retain(|e| VIDEO_CONTAINERS.iter().any(|(ext, _)| e == ext));
        let mut visible = HashSet::new();
        s.home_filter_options
            .retain(|extension| visible.insert(extension.clone()));
        s.filter_extensions
            .retain(|e| s.home_filter_options.contains(e));
        s.zoom_percent = s.zoom_percent.clamp(75, 150);
        s.number_format = crate::naming::migrate_legacy(&s.number_format);
        s.default_season = s.default_season.min(999);
        s.first_episode = s.first_episode.min(9999);
        s.table_name_ratio = s.table_name_ratio.clamp(0.05, 0.95);
        s.table_status_width = s.table_status_width.clamp(90., 10000.);
        s.window_width = s.window_width.clamp(480., 4000.);
        s.window_height = s.window_height.clamp(360., 2400.);
        s
    }
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let bytes = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        atomic_write(path, &bytes)
    }
}
pub fn data_dir() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("EpisodeRenamer")
}
// Bounded reads also enforce the limit if a file grows while being read.
fn read_bounded(path: &Path, limit: u64) -> Result<Vec<u8>, String> {
    let file = fs::File::open(path).map_err(|e| e.to_string())?;
    if file.metadata().map_err(|e| e.to_string())?.len() > limit {
        return Err("Data file exceeds the supported size.".into());
    }
    let mut bytes = Vec::new();
    file.take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > limit {
        return Err("Data file exceeds the supported size.".into());
    }
    Ok(bytes)
}
pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let parent = path.parent().ok_or("Invalid data path")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let temp = parent.join(format!(
        ".anemorph-{}-{}.tmp",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    // Never truncate or follow an existing temporary file.
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .map_err(|e| e.to_string())?;
    let result = (|| {
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temp, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result.map_err(|e: std::io::Error| e.to_string())
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum MediaKind {
    Episode,
    Movie,
}
pub fn media_kind(path: &Path) -> MediaKind {
    static MOVIE: LazyLock<Regex> =
        LazyLock::new(|| re(r"(?i)(?:^|[^a-z0-9])movies?(?:$|[^a-z0-9])"));
    if MOVIE.is_match(&path.file_stem().unwrap_or_default().to_string_lossy()) {
        MediaKind::Movie
    } else {
        MediaKind::Episode
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MediaOverride {
    pub kind: MediaKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub season: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub episode: Option<u32>,
}
impl From<MediaKind> for MediaOverride {
    fn from(kind: MediaKind) -> Self {
        Self {
            kind,
            season: None,
            episode: None,
        }
    }
}
impl MediaOverride {
    pub fn episode(season: u32, episode: u32) -> Self {
        Self {
            kind: MediaKind::Episode,
            season: Some(season),
            episode: Some(episode),
        }
    }
    pub fn numbering(self) -> Option<(u32, u32)> {
        if self.kind == MediaKind::Episode {
            self.season.zip(self.episode)
        } else {
            None
        }
    }
}
pub type MediaMarkers = HashMap<PathBuf, MediaOverride>;
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct MediaMarker {
    path: PathBuf,
    #[serde(flatten)]
    media: MediaOverride,
}
pub fn media_marker_key(path: &Path) -> PathBuf {
    PathBuf::from(key(path))
}
pub fn marked_media_kind(path: &Path, markers: &MediaMarkers) -> MediaKind {
    if markers.is_empty() {
        return media_kind(path);
    }
    markers
        .get(&media_marker_key(path))
        .map(|mark| mark.kind)
        .unwrap_or_else(|| media_kind(path))
}
pub fn load_media_markers(path: &Path) -> MediaMarkers {
    read_bounded(path, 16 * 1024 * 1024)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Vec<MediaMarker>>(&bytes).ok())
        .unwrap_or_default()
        .into_iter()
        .map(|m| (media_marker_key(&m.path), m.media))
        .collect()
}
pub fn save_media_markers(path: &Path, markers: &MediaMarkers) -> Result<(), String> {
    let records: Vec<_> = markers
        .iter()
        .map(|(path, media)| MediaMarker {
            path: path.clone(),
            media: *media,
        })
        .collect();
    let bytes = serde_json::to_vec(&records).map_err(|e| e.to_string())?;
    if bytes.len() > 16 * 1024 * 1024 {
        return Err("Too many media markers to save.".into());
    }
    atomic_write(path, &bytes)
}
pub fn move_media_markers(markers: &mut MediaMarkers, changes: &[Change]) {
    let moved: Vec<_> = changes
        .iter()
        .filter_map(|c| {
            markers
                .remove(&media_marker_key(&c.source))
                .map(|kind| (media_marker_key(&c.target), kind))
        })
        .collect();
    markers.extend(moved);
}
#[cfg(test)]
use crate::containers::OTHER_VIDEO_EXTENSIONS;
pub use crate::containers::VIDEO_CONTAINERS;
fn movie_name(path: &Path) -> String {
    static TAGS: LazyLock<Regex> = LazyLock::new(|| re(r"\[[^\]]*\]"));
    static META: LazyLock<Regex> = LazyLock::new(|| {
        re(
            r"(?i)\((?:movie|movies|(?:BD|BluRay|WEB[- .]?DL|WEBRip|DVDRip|HDRip|Dual[ .]Audio|Multi[ .]Audio|x26[45]|h[ .]?26[45]|HEVC|AV1|AAC|FLAC|[0-9]{3,4}p)[^)]*)\)",
        )
    });
    static TECH: LazyLock<Regex> = LazyLock::new(|| {
        re(
            r"(?i)\b(?:[0-9]{3,4}p|x26[45]|h[ .]?26[45]|hevc|aac|flac|bluray|webrip|web[- .]?dl|bdrip)\b",
        )
    });
    static MARKER: LazyLock<Regex> =
        LazyLock::new(|| re(r"(?i)(?:^|[^a-z0-9])movies?(?:$|[^a-z0-9])"));
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let title = TAGS.replace_all(&stem, " ");
    let title = META.replace_all(&title, " ");
    let title = title.replace('_', " ");
    let title = TECH.replace_all(&title, " ");
    let title = MARKER.replace_all(&title, " ");
    title
        .replace('_', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim_matches(|c: char| c.is_whitespace() || "-._".contains(c))
        .to_owned()
}
#[derive(Clone, Debug)]
pub struct Row {
    pub kind: MediaKind,
    pub source: PathBuf,
    pub target: Option<PathBuf>,
    pub original: String,
    pub output: String,
    pub status: std::borrow::Cow<'static, str>,
    pub ready: bool,
    pub included: bool,
    pub highlighted: bool,
    pub assumed: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Change {
    pub source: PathBuf,
    pub target: PathBuf,
}
fn re(s: &str) -> Regex {
    Regex::new(s).expect("Built-in episode pattern")
}
static EXPLICIT: LazyLock<Regex> =
    LazyLock::new(|| re(r"(?i)(?:^|[^a-z0-9])S(?P<s>\d{1,3})[ ._-]*E(?P<e>\d{1,3})(?:\D|$)"));
static SEPARATED: LazyLock<Regex> = LazyLock::new(|| {
    re(r"(?i)(?:^|[^a-z0-9])S(?:eason)?[ ._-]*(?P<s>\d{1,3})[ ._]*-[ ._]*(?P<e>\d{1,3})(?:\D|$)")
});
static CROSS: LazyLock<Regex> =
    LazyLock::new(|| re(r"(?i)(?:^|\D)(?P<s>\d{1,2})x(?P<e>\d{1,3})(?:\D|$)"));
static FOLDER: LazyLock<Regex> =
    LazyLock::new(|| re(r"(?i)(?:^|[ ._\-\[])S(?:eason)?[ ._-]*(?P<s>\d{1,3})(?:$|[ ._\-\]])"));
static ORDINAL: LazyLock<Regex> =
    LazyLock::new(|| re(r"(?i)^(?P<s>\d{1,2})(?:st|nd|rd|th)\s+season$"));
static EPISODE: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        re(r"(?i)(?:^|[^a-z0-9])(?:EP(?:ISODE)?|E)[ ._-]*(?P<e>\d{1,3})(?:\D|$)"),
        re(r"^\s*(?P<e>\d{1,3})(?:[ ._-]+|$)"),
        re(r"\s-\s*(?P<e>\d{1,3})(?:\s|\[|\(|$)"),
        re(r"\s(?P<e>\d{1,3})(?:\s*(?:\[[^\]]*\]|\([^)]*\)))*\s*$"),
    ]
});
static DIGITS: LazyLock<Regex> = LazyLock::new(|| re(r"\d+"));
fn explicit(name: &str) -> Option<(u32, u32)> {
    EXPLICIT
        .captures(name)
        .or_else(|| SEPARATED.captures(name))
        .or_else(|| CROSS.captures(name))
        .and_then(|c| Some((c["s"].parse().ok()?, c["e"].parse().ok()?)))
}
pub fn recognized_episode(
    path: &Path,
    format: &crate::naming::Format,
    default_season: u32,
) -> Option<(u32, u32)> {
    let name = path.file_stem().unwrap_or_default().to_string_lossy();
    let known = explicit(&name)
        .map(|(s, e)| (Some(s), e))
        .or_else(|| format.detect(&name));
    let episode = known.map(|(_, e)| e).or_else(|| {
        EPISODE
            .iter()
            .find_map(|pattern| pattern.captures(&name).and_then(|c| c["e"].parse().ok()))
    })?;
    let season = known
        .and_then(|(s, _)| s)
        .or_else(|| path.parent().and_then(folder_season))
        .unwrap_or(default_season);
    Some((season, episode))
}
fn folder_season(path: &Path) -> Option<u32> {
    path.ancestors()
        .filter_map(|p| p.file_name())
        .filter_map(|n| n.to_str())
        .find_map(|n| {
            FOLDER
                .captures(n)
                .or_else(|| ORDINAL.captures(n))
                .and_then(|c| c["s"].parse().ok())
        })
}
fn key(p: &Path) -> String {
    p.to_string_lossy().to_lowercase()
}
pub fn scan(root: &Path, settings: &Settings) -> Result<Vec<PathBuf>, String> {
    if !root.is_dir() {
        return Err("Choose an existing folder first.".into());
    }
    let depth = if settings.subfolders { usize::MAX } else { 1 };
    let mut files = Vec::new();
    for item in walkdir::WalkDir::new(root)
        .follow_links(false)
        .max_depth(depth)
    {
        let entry = item.map_err(|e| e.to_string())?;
        if !entry.file_type().is_file() {
            continue;
        }
        let ext = entry
            .path()
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();
        if settings.container_enabled(&ext) {
            files.push(entry.into_path())
        }
    }
    files.sort_by_cached_key(|p| {
        DIGITS
            .replace_all(&key(p), |c: &regex::Captures| format!("{:0>12}", &c[0]))
            .into_owned()
    });
    Ok(files)
}
pub fn guess_name(path: &Path) -> String {
    let mut p = path;
    let simple = re(r"(?i)^(?:S(?:eason)?[ ._-]*\d{1,3}|\d{1,2}|brakes)$");
    if p.file_name()
        .is_some_and(|n| simple.is_match(&without_parentheses(&n.to_string_lossy())))
    {
        p = p.parent().unwrap_or(p)
    }
    re(r"(?i)[ ._-]+S(?:eason)?[ ._-]*\d{1,3}$")
        .replace(
            &without_parentheses(&p.file_name().unwrap_or_default().to_string_lossy()),
            "",
        )
        .trim()
        .into()
}
fn without_parentheses(name: &str) -> String {
    let mut depth = 0_u32;
    let mut cleaned = String::with_capacity(name.len());
    for c in name.chars() {
        match c {
            '(' => {
                depth += 1;
                if depth == 1 {
                    cleaned.push(' ');
                }
            }
            ')' => {
                depth = depth.saturating_sub(1);
            }
            _ if depth == 0 => cleaned.push(c),
            _ => {}
        }
    }
    cleaned.split_whitespace().collect::<Vec<_>>().join(" ")
}
pub fn plan(
    root: &Path,
    name: &str,
    files: &[PathBuf],
    settings: &Settings,
) -> Result<Vec<Row>, String> {
    plan_with_markers(root, name, files, settings, &MediaMarkers::new())
}
pub fn plan_with_markers(
    root: &Path,
    name: &str,
    files: &[PathBuf],
    settings: &Settings,
    markers: &MediaMarkers,
) -> Result<Vec<Row>, String> {
    let name = name.trim();
    let name_error = if name.is_empty() {
        Some("Enter the series name to preview filenames.")
    } else if name.chars().any(|c| c < ' ' || "<>:\"/\\|?*".contains(c)) || name.ends_with('.') {
        Some("The series name contains a character Windows cannot use in a filename.")
    } else {
        None
    };
    if let Some(error) = name_error
        && !files
            .iter()
            .any(|p| marked_media_kind(p, markers) == MediaKind::Movie)
    {
        return Err(error.into());
    }
    let number_format = crate::naming::Format::parse(&settings.number_format)?;
    let movie_format = crate::naming::MovieFormat::parse(&settings.movie_format)?;
    let mut siblings: HashMap<PathBuf, BTreeSet<u32>> = HashMap::new();
    // Parse each filename once and reuse its result for sibling inference and output.
    let detected: Vec<_> = files
        .iter()
        .map(|p| {
            if marked_media_kind(p, markers) == MediaKind::Movie {
                return (None, None, false);
            }
            let manual = if markers.is_empty() {
                None
            } else {
                markers.get(&media_marker_key(p))
            };
            if let Some((season, episode)) = manual.and_then(|m| m.numbering()) {
                return (Some(season), Some(episode), true);
            }
            let n = p.file_stem().unwrap_or_default().to_string_lossy();
            let known = explicit(&n)
                .map(|(s, e)| (Some(s), e))
                .or_else(|| number_format.detect(&n));
            let season = known.and_then(|(s, _)| s);
            let episode = known.map(|(_, e)| e).or_else(|| {
                if !settings.auto_detect && !manual.is_some_and(|m| m.kind == MediaKind::Episode) {
                    return None;
                }
                EPISODE
                    .iter()
                    .find_map(|r| r.captures(&n).and_then(|c| c["e"].parse().ok()))
            });
            (season, episode, known.is_some())
        })
        .collect();
    for (p, (season, _, _)) in files.iter().zip(&detected) {
        if let Some(s) = *season {
            siblings
                .entry(p.parent().unwrap().into())
                .or_default()
                .insert(s);
        }
    }
    let mut rows = Vec::with_capacity(files.len());
    let mut seasons = HashMap::new();
    let mut episode_index = 0_u32;
    for (index, p) in files.iter().enumerate() {
        let mut row = Row {
            kind: marked_media_kind(p, markers),
            source: p.clone(),
            target: None,
            original: p.strip_prefix(root).unwrap_or(p).to_string_lossy().into(),
            output: "Episode number not found".into(),
            status: "Skipped".into(),
            ready: false,
            included: false,
            highlighted: false,
            assumed: false,
        };
        let mut title_only = false;
        if row.kind == MediaKind::Movie {
            let title = movie_name(p);
            if title.is_empty() {
                row.output = "Movie title not found".into();
                rows.push(row);
                continue;
            }
            row.output = format!(
                "{}.{}",
                movie_format.render(&title),
                p.extension().unwrap_or_default().to_string_lossy()
            );
        } else {
            if let Some(error) = name_error {
                row.output = error.into();
                row.status = "Needs input".into();
                rows.push(row);
                continue;
            }
            let (known_season, found_episode, explicit_number) = detected[index];
            let (season, episode) = if let Some(ep) = found_episode {
                if (ep == 480 || ep == 720) && !explicit_number {
                    rows.push(row);
                    continue;
                }
                let mut s = known_season.or_else(|| {
                    *seasons
                        .entry(p.parent().unwrap())
                        .or_insert_with(|| folder_season(p.parent().unwrap()))
                });
                if s.is_none()
                    && let Some(known) = siblings.get(p.parent().unwrap())
                {
                    if known.len() == 1 {
                        s = known.first().copied()
                    } else {
                        row.output = "Season is ambiguous".into();
                        rows.push(row);
                        continue;
                    }
                }
                row.assumed = s.is_none();
                (s.unwrap_or(settings.default_season), ep)
            } else if settings.manual_numbering {
                let episode = settings
                    .first_episode
                    .checked_add(episode_index)
                    .ok_or("Episode number exceeds the supported range")?;
                episode_index = episode_index
                    .checked_add(1)
                    .ok_or("Too many episode files")?;
                (settings.default_season, episode)
            } else {
                rows.push(row);
                continue;
            };
            let suffix = number_format.render(season, episode);
            let ending = format!(" {suffix}");
            title_only = p
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .strip_suffix(&ending)
                .is_some_and(|old| old != name);
            row.output = format!(
                "{name} {suffix}.{}",
                p.extension().unwrap_or_default().to_string_lossy()
            );
        }
        if row.output.encode_utf16().count() > 255 {
            return Err(
                "The resulting filename is too long. Shorten the series name or format.".into(),
            );
        }
        let target = p.with_file_name(&row.output);
        row.ready = key(p) != key(&target);
        row.included = row.ready;
        row.status = if !row.ready {
            "No change".into()
        } else if row.kind == MediaKind::Movie {
            "Movie".into()
        } else if title_only {
            "Title changed".into()
        } else if row.assumed {
            format!("S{:02} assumed", settings.default_season).into()
        } else {
            "Ready".into()
        };
        row.target = Some(target);
        rows.push(row);
    }
    drop(detected);
    drop(siblings);
    drop(seasons);
    let mut counts = HashMap::with_capacity(rows.len());
    for r in rows.iter().filter(|r| r.ready) {
        *counts.entry(key(r.target.as_ref().unwrap())).or_insert(0) += 1
    }
    // One directory listing replaces a metadata system call for every target.
    // This is a preview snapshot; MoveFileW still enforces no-overwrite at commit.
    let mut directories: HashMap<&Path, Option<HashSet<String>>> = HashMap::new();
    for r in rows.iter_mut().filter(|r| r.ready) {
        let target = r.target.as_ref().unwrap();
        let existing = directories
            .entry(target.parent().unwrap())
            .or_insert_with(|| {
                let entries = match fs::read_dir(target.parent().unwrap()) {
                    Ok(entries) => entries,
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                        return Some(HashSet::new());
                    }
                    Err(_) => return None,
                };
                entries
                    .map(|item| item.map(|e| e.file_name().to_string_lossy().to_lowercase()))
                    .collect::<Result<HashSet<_>, _>>()
                    .ok()
            });
        let exists = existing.as_ref().map_or_else(
            || target.exists(),
            |names| {
                names.contains(
                    &target
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_lowercase(),
                )
            },
        );
        if exists || counts[&key(target)] > 1 {
            r.ready = false;
            r.included = false;
            r.status = "Name conflict".into();
        }
    }
    Ok(rows)
}
#[cfg(windows)]
fn move_no_replace(source: &Path, target: &Path) -> std::io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    let a: Vec<u16> = source.as_os_str().encode_wide().chain(Some(0)).collect();
    let b: Vec<u16> = target.as_os_str().encode_wide().chain(Some(0)).collect();
    // Both UTF-16 buffers remain alive for the synchronous Windows call. MoveFileW never overwrites.
    if unsafe { windows_sys::Win32::Storage::FileSystem::MoveFileW(a.as_ptr(), b.as_ptr()) } == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}
#[cfg(not(windows))]
fn move_no_replace(source: &Path, target: &Path) -> std::io::Result<()> {
    if target.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "Target exists",
        ));
    }
    fs::rename(source, target)
}
pub fn apply(changes: &[Change]) -> Result<(), String> {
    apply_checked(changes, false)
}
fn apply_checked(changes: &[Change], identities_verified: bool) -> Result<(), String> {
    if changes.is_empty() {
        return Err("Select at least one file to rename.".into());
    }
    let mut targets = HashSet::with_capacity(changes.len());
    let mut directories: HashMap<&Path, Option<HashSet<String>>> = HashMap::new();
    for c in changes {
        if c.target.file_name().is_none_or(|n| {
            n.to_string_lossy()
                .chars()
                .any(|c| c < ' ' || "<>:\"/\\|?*".contains(c))
        }) {
            return Err("Invalid destination filename.".into());
        }
        if !c.source.is_absolute()
            || !c.target.is_absolute()
            || c.source.parent() != c.target.parent()
            || c.source.extension() != c.target.extension()
        {
            return Err("Invalid rename record.".into());
        }
        if !identities_verified
            && !fs::symlink_metadata(&c.source).is_ok_and(|m| m.file_type().is_file())
        {
            return Err(format!("File no longer exists: {}", c.source.display()));
        }
        let names = directories
            .entry(c.target.parent().unwrap())
            .or_insert_with(|| {
                fs::read_dir(c.target.parent().unwrap())
                    .ok()?
                    .map(|e| e.map(|e| e.file_name().to_string_lossy().to_lowercase()))
                    .collect::<Result<HashSet<_>, _>>()
                    .ok()
            });
        let occupied = names.as_ref().map_or_else(
            || c.target.exists(),
            |names| {
                names.contains(
                    &c.target
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_lowercase(),
                )
            },
        );
        if occupied || !targets.insert(key(&c.target)) {
            return Err(format!(
                "Destination already exists: {}",
                c.target.display()
            ));
        }
    }
    let mut done: Vec<&Change> = Vec::with_capacity(changes.len());
    for c in changes {
        if let Err(e) = move_no_replace(&c.source, &c.target) {
            let mut failures = Vec::new();
            for previous in done.iter().rev() {
                if let Err(e) = move_no_replace(&previous.target, &previous.source) {
                    failures.push(e.to_string())
                }
            }
            return Err(if failures.is_empty() {
                format!("{e}. Completed changes were rolled back.")
            } else {
                format!(
                    "{e}. Some files could not be restored: {}",
                    failures.join("; ")
                )
            });
        }
        done.push(c)
    }
    Ok(())
}
pub fn reverse(changes: &[Change]) -> Vec<Change> {
    changes
        .iter()
        .rev()
        .map(|c| Change {
            source: c.target.clone(),
            target: c.source.clone(),
        })
        .collect()
}
// A shared, OS-enforced lock prevents two app instances from replacing the same
// undo record during a batch. Windows releases this handle lock even on a crash.
fn journal_lock(history: &Path) -> Result<fs::File, String> {
    if let Some(parent) = history.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut options = fs::OpenOptions::new();
    options.read(true).write(true).create(true).truncate(false);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.share_mode(0);
    }
    options
        .open(history.with_extension("lock"))
        .map_err(|e| format!("Undo history is busy or unavailable: {e}"))
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Identity {
    volume: u32,
    index: u64,
    size: u64,
    modified: u64,
}
fn identity(path: &Path) -> Result<Identity, String> {
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::Storage::FileSystem::{
            BY_HANDLE_FILE_INFORMATION, GetFileInformationByHandle,
        };
        use windows_sys::Win32::Storage::FileSystem::{
            FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_REPARSE_POINT, FILE_FLAG_OPEN_REPARSE_POINT,
            FILE_READ_ATTRIBUTES,
        };
        let file = fs::OpenOptions::new()
            .read(true)
            .access_mode(FILE_READ_ATTRIBUTES)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(path)
            .map_err(|e| e.to_string())?;
        let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
        // The file owns the live handle, and info is a correctly sized output buffer.
        if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut info) } == 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }
        if info.dwFileAttributes & (FILE_ATTRIBUTE_REPARSE_POINT | FILE_ATTRIBUTE_DIRECTORY) != 0 {
            return Err(format!("Not a regular file: {}", path.display()));
        }
        Ok(Identity {
            volume: info.dwVolumeSerialNumber,
            index: ((info.nFileIndexHigh as u64) << 32) | info.nFileIndexLow as u64,
            size: ((info.nFileSizeHigh as u64) << 32) | info.nFileSizeLow as u64,
            modified: ((info.ftLastWriteTime.dwHighDateTime as u64) << 32)
                | info.ftLastWriteTime.dwLowDateTime as u64,
        })
    }
    #[cfg(not(windows))]
    {
        let m = fs::metadata(path).map_err(|e| e.to_string())?;
        Ok(Identity {
            volume: 0,
            index: 0,
            size: m.len(),
            modified: m
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map_or(0, |t| t.as_nanos() as u64),
        })
    }
}
#[derive(Serialize, Deserialize)]
struct Journal {
    changes: Vec<Change>,
    identities: Vec<Identity>,
}
#[derive(Deserialize)]
#[serde(untagged)]
enum SavedJournal {
    Current(Journal),
    Legacy(Vec<Change>),
}
fn read_journal(path: &Path) -> Result<SavedJournal, String> {
    serde_json::from_slice(&read_bounded(path, 64 * 1024 * 1024)?).map_err(|e| e.to_string())
}
pub fn rename_batch(changes: &[Change], history: &Path) -> Result<usize, String> {
    let _lock = journal_lock(history)?;
    let pending = history.with_extension("staged");
    if pending.exists() {
        return Err(
            "An interrupted batch needs recovery. Use Undo before renaming another batch.".into(),
        );
    }
    let journal = Journal {
        changes: changes.to_vec(),
        identities: changes
            .iter()
            .map(|c| identity(&c.source))
            .collect::<Result<_, _>>()?,
    };
    let bytes = serde_json::to_vec(&journal).map_err(|e| e.to_string())?;
    if bytes.len() > 64 * 1024 * 1024 {
        return Err("The undo record is too large. Rename a smaller batch.".into());
    }
    atomic_write(&pending, &bytes)?;
    if let Err(e) = apply_checked(changes, true) {
        // Only discard recovery information when every source is still intact.
        if changes
            .iter()
            .zip(&journal.identities)
            .all(|(c, id)| identity(&c.source).as_ref() == Ok(id))
        {
            let _ = fs::remove_file(&pending);
        }
        return Err(e);
    }
    if let Err(e) = atomic_write(history, &bytes) {
        let rollback = apply(&reverse(changes));
        if rollback.is_ok() {
            let _ = fs::remove_file(&pending);
        }
        return Err(format!(
            "Could not save undo history: {e}. Rollback: {rollback:?}"
        ));
    }
    let _ = fs::remove_file(pending);
    Ok(changes.len())
}
pub fn undo(history: &Path) -> Result<usize, String> {
    undo_with_changes(history).map(|changes| changes.len())
}
pub fn undo_with_changes(history: &Path) -> Result<Vec<Change>, String> {
    let _lock = journal_lock(history)?;
    let pending = history.with_extension("staged");
    let staged = pending.exists();
    let record = if staged { &pending } else { history };
    let saved = read_journal(record)?;
    let changes = match &saved {
        SavedJournal::Legacy(changes) => reverse(changes),
        SavedJournal::Current(journal) => {
            if journal.changes.len() != journal.identities.len() {
                return Err("Invalid undo record.".into());
            }
            let mut reverse = Vec::with_capacity(journal.changes.len());
            // Validate the whole batch before changing anything. Already restored
            // entries are skipped, allowing recovery after an interrupted undo.
            for (c, id) in journal.changes.iter().zip(&journal.identities).rev() {
                if identity(&c.target).as_ref() == Ok(id) && !c.source.exists() {
                    reverse.push(Change {
                        source: c.target.clone(),
                        target: c.source.clone(),
                    });
                } else if !(identity(&c.source).as_ref() == Ok(id) && !c.target.exists()) {
                    return Err(format!(
                        "File changed or destination occupied; recovery stopped: {}",
                        c.target.display()
                    ));
                }
            }
            reverse
        }
    };
    if !changes.is_empty() {
        apply_checked(&changes, matches!(saved, SavedJournal::Current(_)))?;
    }
    if staged {
        // A crash can occur after the committed record is saved but before the
        // stage is removed. Remove the matching record after recovery only.
        if let (SavedJournal::Current(a), Ok(SavedJournal::Current(b))) =
            (&saved, read_journal(history))
            && a.changes == b.changes
            && a.identities == b.identities
        {
            fs::remove_file(history).map_err(|e| e.to_string())?;
        }
    }
    fs::remove_file(record).map_err(|e| e.to_string())?;
    Ok(changes)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> PathBuf {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "anemorph-test-{}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }
    #[test]
    fn interrupted_batch_and_undo_recover_without_touching_contents() {
        let root = fixture();
        let changes: Vec<_> = (0..3)
            .map(|i| Change {
                source: root.join(format!("old{i}.mkv")),
                target: root.join(format!("new{i}.mkv")),
            })
            .collect();
        for (i, c) in changes.iter().enumerate() {
            fs::write(&c.source, format!("video{i}")).unwrap();
        }
        let journal = Journal {
            identities: changes
                .iter()
                .map(|c| identity(&c.source).unwrap())
                .collect(),
            changes: changes.clone(),
        };
        let history = root.join("history.json");
        atomic_write(
            &history.with_extension("staged"),
            &serde_json::to_vec(&journal).unwrap(),
        )
        .unwrap();
        move_no_replace(&changes[0].source, &changes[0].target).unwrap();
        assert!(rename_batch(&changes, &history).is_err());
        assert_eq!(undo(&history).unwrap(), 1);
        for (i, c) in changes.iter().enumerate() {
            assert_eq!(fs::read_to_string(&c.source).unwrap(), format!("video{i}"));
        }
        rename_batch(&changes, &history).unwrap();
        move_no_replace(&changes[1].target, &changes[1].source).unwrap();
        assert_eq!(undo(&history).unwrap(), 2);
        assert!(
            changes
                .iter()
                .all(|c| c.source.exists() && !c.target.exists())
        );
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn undo_rejects_modified_file_and_preserves_history() {
        let root = fixture();
        let c = Change {
            source: root.join("old.mkv"),
            target: root.join("new.mkv"),
        };
        fs::write(&c.source, b"video").unwrap();
        let history = root.join("history.json");
        rename_batch(std::slice::from_ref(&c), &history).unwrap();
        fs::write(&c.target, b"replacement content").unwrap();
        assert!(undo(&history).is_err());
        assert!(!c.source.exists() && history.exists());
        assert_eq!(fs::read(&c.target).unwrap(), b"replacement content");
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn legacy_undo_and_durable_settings_remain_compatible() {
        let root = fixture();
        let history = root.join("history.json");
        let c = Change {
            source: root.join("old.mkv"),
            target: root.join("new.mkv"),
        };
        fs::write(&c.target, b"legacy").unwrap();
        fs::write(&history, serde_json::to_vec(&vec![c.clone()]).unwrap()).unwrap();
        assert_eq!(undo(&history).unwrap(), 1);
        let settings = root.join("settings.json");
        Settings::default().save(&settings).unwrap();
        let s = Settings {
            dark_mode: false,
            ..Settings::default()
        };
        s.save(&settings).unwrap();
        assert!(!Settings::load(&settings).dark_mode);
        assert!(
            fs::read_dir(&root).unwrap().all(|e| !e
                .unwrap()
                .path()
                .extension()
                .is_some_and(|e| e == "tmp"))
        );
        fs::write(&settings, vec![b' '; 65537]).unwrap();
        assert_eq!(Settings::load(&settings), Settings::default());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    #[cfg(windows)]
    fn concurrent_instances_cannot_share_undo_lock() {
        let root = fixture();
        let history = root.join("history.json");
        let lock = journal_lock(&history).unwrap();
        assert!(journal_lock(&history).is_err());
        drop(lock);
        assert!(journal_lock(&history).is_ok());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn manual_number_overflow_and_extension_changes_are_rejected() {
        let root = fixture();
        let a = root.join("one.mkv");
        let b = root.join("two.mkv");
        let settings = Settings {
            auto_detect: false,
            manual_numbering: true,
            first_episode: u32::MAX,
            ..Default::default()
        };
        assert!(plan(&root, "Show", &[a.clone(), b], &settings).is_err());
        fs::write(&a, b"video").unwrap();
        assert!(
            apply(&[Change {
                source: a.clone(),
                target: root.join("one.exe")
            }])
            .is_err()
        );
        assert!(a.exists());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn multiple_container_choices_are_independent_and_clear_means_unfiltered() {
        let root = fixture();
        for name in ["E01.mkv", "E02.mp4", "E03.MOV"] {
            fs::write(root.join(name), b"video").unwrap();
        }
        let mut settings = Settings::default();
        assert!(settings.filter_extensions.is_empty());
        assert_eq!(scan(&root, &settings).unwrap().len(), 3);
        assert_eq!(settings.container_mask(), 0);
        settings.toggle_all_containers();
        assert_eq!(
            settings.filter_extensions.len(),
            settings.home_filter_options.len()
        );
        assert_eq!(settings.container_mask(), (1_u32 << 6) - 1);
        assert_eq!(scan(&root, &settings).unwrap().len(), 3);
        settings.toggle_all_containers();
        assert!(settings.filter_extensions.is_empty());
        settings.toggle_container("mkv");
        assert_eq!(scan(&root, &settings).unwrap(), vec![root.join("E01.mkv")]);
        settings.toggle_container("mp4");
        assert_eq!(scan(&root, &settings).unwrap().len(), 2);
        assert!(settings.container_selected("mkv") && settings.container_selected("mp4"));
        let path = root.join("settings.json");
        settings.save(&path).unwrap();
        assert_eq!(Settings::load(&path).filter_extensions, vec!["mkv", "mp4"]);
        settings.toggle_container("mkv");
        assert_eq!(scan(&root, &settings).unwrap(), vec![root.join("E02.mp4")]);
        settings.toggle_container("mp4");
        assert!(settings.filter_extensions.is_empty());
        assert_eq!(scan(&root, &settings).unwrap().len(), 3);
        fs::write(&path, r#"{"FilterExtension":"MOV"}"#).unwrap();
        assert_eq!(Settings::load(&path).filter_extensions, vec!["mov"]);
        fs::write(&path, r#"{"Mkv":false,"Mp4":false,"OtherVideo":false}"#).unwrap();
        assert!(Settings::load(&path).filter_extensions.is_empty());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn configuration_visibility_is_separate_from_active_filters() {
        let root = fixture();
        fs::write(root.join("E01.mkv"), b"video").unwrap();
        fs::write(root.join("E02.wmv"), b"video").unwrap();
        let mut settings = Settings::default();
        assert_eq!(
            settings.home_filter_options,
            vec!["mkv", "mp4", "mov", "avi", "webm", "m4v"]
        );
        assert!(settings.filter_extensions.is_empty());
        assert!(!settings.container_visible("wmv"));
        assert_eq!(scan(&root, &settings).unwrap().len(), 2);
        settings.toggle_container_visibility("wmv");
        assert!(settings.container_visible("wmv") && settings.filter_extensions.is_empty());
        settings.toggle_container("wmv");
        assert_eq!(scan(&root, &settings).unwrap(), vec![root.join("E02.wmv")]);
        settings.toggle_container_visibility("wmv");
        assert!(!settings.container_visible("wmv") && settings.filter_extensions.is_empty());
        assert_eq!(scan(&root, &settings).unwrap().len(), 2);
        settings.home_filter_options = VIDEO_CONTAINERS
            .iter()
            .map(|(ext, _)| (*ext).to_owned())
            .collect();
        settings.toggle_all_containers();
        assert_eq!(settings.filter_extensions.len(), VIDEO_CONTAINERS.len());
        settings.toggle_all_containers();
        settings.home_filter_options.clear();
        let path = root.join("visibility.json");
        settings.save(&path).unwrap();
        let loaded = Settings::load(&path);
        assert!(loaded.home_filter_options.is_empty() && loaded.filter_extensions.is_empty());
        assert_eq!(scan(&root, &loaded).unwrap().len(), 2);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn expanded_video_containers_obey_filters_and_preserve_extensions() {
        let root = fixture();
        for extension in OTHER_VIDEO_EXTENSIONS {
            fs::write(root.join(format!("Raw E01.{extension}")), b"video").unwrap();
        }
        fs::write(root.join("Raw E02.MOV"), b"video").unwrap();
        fs::write(root.join("ignore.txt"), b"text").unwrap();
        let files = scan(&root, &Settings::default()).unwrap();
        assert_eq!(files.len(), OTHER_VIDEO_EXTENSIONS.len() + 1);
        let rows = plan(&root, "Show", &files, &Settings::default()).unwrap();
        for row in rows {
            assert_eq!(row.source.extension(), row.target.unwrap().extension());
        }
        let settings = Settings {
            filter_extensions: vec!["mkv".into()],
            ..Default::default()
        };
        assert!(scan(&root, &settings).unwrap().is_empty());
        let mut individual = Settings::default();
        individual.toggle_container("mov");
        let selected = scan(&root, &individual).unwrap();
        assert_eq!(selected.len(), 2);
        assert!(
            selected
                .iter()
                .all(|p| p.extension().unwrap().eq_ignore_ascii_case("mov"))
        );
        let settings_path = root.join("individual-settings.json");
        individual.save(&settings_path).unwrap();
        let mut restored = Settings::load(&settings_path);
        assert_eq!(scan(&root, &restored).unwrap(), selected);
        restored.toggle_container("mov");
        assert_eq!(scan(&root, &restored).unwrap().len(), files.len());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn movies_have_separate_names_formats_and_manual_numbering() {
        let root = fixture();
        let movie = root.join("[Group] Example Movie 1080p x265.MKV");
        let movie2 = root.join("[Group] A Silent Voice (Movie) (BD 1080p).MOV");
        let ep1 = root.join("Episode one.mp4");
        let ep2 = root.join("Episode two.mp4");
        for p in [&movie, &movie2, &ep1, &ep2] {
            fs::write(p, b"original video contents").unwrap();
        }
        let settings = Settings {
            auto_detect: false,
            manual_numbering: true,
            first_episode: 9,
            ..Default::default()
        };
        let files = vec![ep1.clone(), movie.clone(), movie2.clone(), ep2.clone()];
        let rows = plan(&root, "Series", &files, &settings).unwrap();
        assert_eq!(rows[0].output, "Series S01E09.mp4");
        assert_eq!(rows[1].output, "Example Movie.MKV");
        assert_eq!(rows[2].output, "A Silent Voice Movie.MOV");
        assert_eq!(rows[3].output, "Series S01E10.mp4");
        assert_eq!(rows[1].kind, MediaKind::Movie);
        assert_eq!(
            media_kind(&root.join("Moviemaker E01.mkv")),
            MediaKind::Episode
        );
        let history = root.join("undo.json");
        let changes: Vec<_> = rows
            .iter()
            .map(|r| Change {
                source: r.source.clone(),
                target: r.target.clone().unwrap(),
            })
            .collect();
        rename_batch(&changes, &history).unwrap();
        let renamed: Vec<_> = changes.iter().map(|c| c.target.clone()).collect();
        let refreshed = plan(&root, "Series", &renamed, &settings).unwrap();
        assert_eq!(refreshed[1].kind, MediaKind::Movie);
        assert_eq!(refreshed[1].status, "No change");
        assert_eq!(fs::read(&renamed[1]).unwrap(), b"original video contents");
        undo(&history).unwrap();
        assert!(files.iter().all(|p| p.exists()));
        let movie_only = plan(&root, "", std::slice::from_ref(&movie), &settings).unwrap();
        assert_eq!(movie_only[0].output, "Example Movie.MKV");
        let custom = Settings {
            movie_format: "[Movie][Space]-[Space][Movie Name]".into(),
            ..settings
        };
        assert_eq!(
            plan(&root, "", &[movie], &custom).unwrap()[0].output,
            "Movie - Example.MKV"
        );
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn sparse_existing_numbers_are_preserved_with_detection_on_or_off() {
        let root = fixture();
        let episodes = [1, 4, 8, 9, 12, 17, 19, 20];
        for episode in episodes {
            fs::write(
                root.join(format!("Demon Slayer S01E{episode:02}.mp4")),
                b"fixture",
            )
            .unwrap();
        }
        let film =
            "Kimetsu no Yaiba - Demon Slayer - Mugen Train (2020) [1080p][Dual Audio][JP+ENG].mp4";
        fs::write(root.join(film), b"movie fixture").unwrap();
        let files = scan(&root, &Settings::default()).unwrap();
        for auto_detect in [false, true] {
            let settings = Settings {
                auto_detect,
                ..Default::default()
            };
            let rows = plan(&root, "Demon Slayer", &files, &settings).unwrap();
            assert_eq!(rows.len(), 9);
            for row in rows.iter().filter(|r| r.original != film) {
                assert_eq!(row.output, row.original);
                assert_eq!(row.status, "No change");
                assert!(!row.ready && !row.included);
            }
            let unknown = rows.iter().find(|r| r.original == film).unwrap();
            assert!(unknown.target.is_none() && !unknown.ready);
            assert!(rows.iter().all(|r| r.status != "Name conflict"));
        }
        let settings = Settings {
            auto_detect: false,
            manual_numbering: true,
            first_episode: 50,
            ..Default::default()
        };
        let renamed = plan(&root, "New name", &files[..8], &settings).unwrap();
        assert!(renamed.iter().any(|r| r.output == "New name S01E19.mp4"));
        assert!(renamed.iter().any(|r| r.output == "New name S01E20.mp4"));
        for episode in [480, 720] {
            let name = format!("Demon Slayer S02E{episode}.mkv");
            let row = plan(&root, "Demon Slayer", &[root.join(&name)], &settings).unwrap();
            assert_eq!(row[0].output, name);
        }
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn old_detection_off_settings_do_not_enable_sequential_numbering() {
        let root = fixture();
        let config = root.join("settings.json");
        fs::write(&config, br#"{"AutoDetect":false,"FirstEpisode":1}"#).unwrap();
        let settings = Settings::load(&config);
        assert!(!settings.manual_numbering);
        let rows = plan(&root, "Show", &[root.join("Unknown clip.mp4")], &settings).unwrap();
        assert!(rows[0].target.is_none() && !rows[0].ready);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn separated_season_episode_and_noop_status_match_reported_filenames() {
        let root = fixture();
        let filename = "[Moozzi2] Kuroko no Basket S3 - 23 [ 73 ] (BD 1920x1080 x.264 Flac)-23.mp4";
        let p = root.join(filename);
        fs::write(&p, b"fixture").unwrap();
        for auto_detect in [true, false] {
            let settings = Settings {
                auto_detect,
                ..Default::default()
            };
            let rows = plan(
                &root,
                "Kuroko no Basket",
                std::slice::from_ref(&p),
                &settings,
            )
            .unwrap();
            assert_eq!(rows[0].output, "Kuroko no Basket S03E23.mp4");
            assert!(rows[0].ready);
        }
        let named = root.join("Kuroko no Basket S01E19.mp4");
        fs::write(&named, b"fixture").unwrap();
        let unchanged = plan(
            &root,
            "Kuroko no Basket",
            std::slice::from_ref(&named),
            &Settings::default(),
        )
        .unwrap();
        assert_eq!(unchanged[0].status, "No change");
        assert!(!unchanged[0].ready && !unchanged[0].included);
        let changed = plan(&root, "Kuroko no Basuke", &[named], &Settings::default()).unwrap();
        assert_eq!(changed[0].status, "Title changed");
        assert!(changed[0].ready);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn manual_movie_markers_persist_and_follow_rename_and_undo() {
        let root = fixture();
        let file = root.join("Mugen Train (2020) [1080p].mp4");
        fs::write(&file, b"video contents").unwrap();
        let store = root.join("media-markers.json");
        let mut markers = MediaMarkers::new();
        markers.insert(media_marker_key(&file), MediaKind::Movie.into());
        save_media_markers(&store, &markers).unwrap();
        let mut markers = load_media_markers(&store);
        let settings = Settings {
            movie_format: "[Movie Name]".into(),
            ..Default::default()
        };
        let rows =
            plan_with_markers(&root, "", std::slice::from_ref(&file), &settings, &markers).unwrap();
        assert_eq!(rows[0].kind, MediaKind::Movie);
        assert_eq!(rows[0].output, "Mugen Train (2020).mp4");
        assert!(rows[0].ready);
        let changes = vec![Change {
            source: file.clone(),
            target: rows[0].target.clone().unwrap(),
        }];
        let history = root.join("undo.json");
        rename_batch(&changes, &history).unwrap();
        move_media_markers(&mut markers, &changes);
        assert!(markers.contains_key(&media_marker_key(&changes[0].target)));
        assert!(!markers.contains_key(&media_marker_key(&file)));
        save_media_markers(&store, &markers).unwrap();
        let markers_from_disk = load_media_markers(&store);
        assert_eq!(
            marked_media_kind(&changes[0].target, &markers_from_disk),
            MediaKind::Movie
        );
        let undo_changes = undo_with_changes(&history).unwrap();
        move_media_markers(&mut markers, &undo_changes);
        assert_eq!(marked_media_kind(&file, &markers), MediaKind::Movie);
        assert!(!markers.contains_key(&media_marker_key(&changes[0].target)));
        assert_eq!(fs::read(&file).unwrap(), b"video contents");
        let upper = PathBuf::from(file.to_string_lossy().to_uppercase());
        assert_eq!(marked_media_kind(&upper, &markers), MediaKind::Movie);
        markers.remove(&media_marker_key(&file));
        assert_eq!(marked_media_kind(&file, &markers), MediaKind::Episode);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn episode_markers_override_movie_names_and_persist_numbers_through_undo() {
        let root = fixture();
        let source = root.join("Movie clip without number.mp4");
        fs::write(&source, b"unchanged content").unwrap();
        let mut markers = MediaMarkers::new();
        markers.insert(media_marker_key(&source), MediaOverride::episode(4, 19));
        let settings = Settings {
            auto_detect: false,
            ..Default::default()
        };
        let row = plan_with_markers(
            &root,
            "Show",
            std::slice::from_ref(&source),
            &settings,
            &markers,
        )
        .unwrap()
        .remove(0);
        assert_eq!(row.kind, MediaKind::Episode);
        assert_eq!(row.output, "Show S04E19.mp4");
        let history = root.join("undo.json");
        let changes = vec![Change {
            source: source.clone(),
            target: row.target.unwrap(),
        }];
        rename_batch(&changes, &history).unwrap();
        move_media_markers(&mut markers, &changes);
        let store = root.join("markers.json");
        save_media_markers(&store, &markers).unwrap();
        let mut loaded = load_media_markers(&store);
        assert_eq!(
            loaded[&media_marker_key(&changes[0].target)].numbering(),
            Some((4, 19))
        );
        let reversed = undo_with_changes(&history).unwrap();
        move_media_markers(&mut loaded, &reversed);
        let refreshed = plan_with_markers(
            &root,
            "Show",
            std::slice::from_ref(&source),
            &settings,
            &loaded,
        )
        .unwrap();
        assert_eq!(refreshed[0].output, "Show S04E19.mp4");
        assert_eq!(fs::read(&source).unwrap(), b"unchanged content");
        let legacy = serde_json::json!([{"Path":source,"Kind":"Episode"}]);
        fs::write(&store, serde_json::to_vec(&legacy).unwrap()).unwrap();
        let legacy = load_media_markers(&store);
        assert_eq!(legacy[&media_marker_key(&source)].kind, MediaKind::Episode);
        let numbered = root.join("Movie - Ep 23.mp4");
        let mut legacy = MediaMarkers::new();
        legacy.insert(media_marker_key(&numbered), MediaKind::Episode.into());
        assert_eq!(
            plan_with_markers(&root, "Show", &[numbered], &settings, &legacy).unwrap()[0].output,
            "Show S01E23.mp4"
        );
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn formats_and_manual() {
        let root = fixture();
        let s2 = root.join("s2");
        fs::create_dir(&s2).unwrap();
        for n in [
            "Ep-02 Title.mkv",
            "Show - 13.mkv",
            "Show S03E04.mp4",
            "Movie.mkv",
        ] {
            fs::write(s2.join(n), n).unwrap()
        }
        let files = scan(&root, &Settings::default()).unwrap();
        let rows = plan(&root, "Show", &files, &Settings::default()).unwrap();
        assert!(rows.iter().any(|r| r.output == "Show S02E02.mkv"));
        assert!(rows.iter().any(|r| r.output == "Show S02E13.mkv"));
        assert!(rows.iter().any(|r| r.output == "Show S03E04.mp4"));
        assert_eq!(rows.iter().filter(|r| r.status == "Skipped").count(), 1);
        let p = Settings {
            auto_detect: false,
            manual_numbering: true,
            default_season: 7,
            first_episode: 5,
            ..Default::default()
        };
        assert!(
            plan(&root, "Show", &files, &p).unwrap()[0]
                .output
                .contains("S07E05")
        );
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn legacy_settings_and_journal() {
        let root = fixture();
        let path = root.join("settings.json");
        fs::write(&path,r##"{"DarkMode":false,"AccentHex":"#FF0000","AutoDetect":true,"DefaultSeason":3,"LastFolder":null}"##).unwrap();
        let s = Settings::load(&path);
        assert!(!s.dark_mode);
        assert!(s.animations);
        assert!(s.clear_series_on_empty);
        assert_eq!(s.number_format, crate::naming::DEFAULT_FORMAT);
        assert_eq!(s.zoom_percent, 100);
        assert_eq!(s.default_season, 3);
        let a = root.join("old.mkv");
        let b = root.join("new.mkv");
        fs::write(&a, b"contents").unwrap();
        let changes = vec![Change {
            source: a.clone(),
            target: b.clone(),
        }];
        let h = root.join("history.json");
        rename_batch(&changes, &h).unwrap();
        assert_eq!(fs::read(&b).unwrap(), b"contents");
        undo(&h).unwrap();
        assert!(a.exists());
        assert!(!b.exists());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn custom_formats_preserve_detection_extensions_and_undo() {
        let root = fixture();
        let file = root.join("Original S02E09.MP4");
        fs::write(&file, b"unchanged video").unwrap();
        let settings = Settings {
            number_format: "[FixNum]Part [SeaNum][Space]-[Space][Episode][EpsNum]".into(),
            ..Default::default()
        };
        let rows = plan(&root, "Show", std::slice::from_ref(&file), &settings).unwrap();
        assert_eq!(rows[0].output, "Show Part 2 - Episode9.MP4");
        let target = rows[0].target.clone().unwrap();
        let history = root.join("undo.json");
        rename_batch(
            &[Change {
                source: file.clone(),
                target: target.clone(),
            }],
            &history,
        )
        .unwrap();
        let refreshed = plan(&root, "Show", std::slice::from_ref(&target), &settings).unwrap();
        assert_eq!(refreshed[0].status, "No change");
        assert_eq!(refreshed[0].output, rows[0].output);
        assert_eq!(fs::read(&target).unwrap(), b"unchanged video");
        undo(&history).unwrap();
        assert!(file.exists());
        let manual = Settings {
            auto_detect: false,
            manual_numbering: true,
            default_season: 3,
            first_episode: 7,
            number_format: "[FixNum][S][SeaNum][Space][E][EpsNum]".into(),
            ..Default::default()
        };
        assert_eq!(
            plan(&root, "Show", &[file], &manual).unwrap()[0].output,
            "Show S2 E9.MP4"
        );
        let short = Settings {
            number_format: "[Num][E][EpsNum]".into(),
            ..Default::default()
        };
        let a = root.join("Old S01E09.mkv");
        let b = root.join("Old S02E09.mkv");
        fs::write(&a, b"a").unwrap();
        fs::write(&b, b"b").unwrap();
        let conflicts = plan(&root, "Show", &[a, b], &short).unwrap();
        assert!(
            conflicts
                .iter()
                .all(|r| !r.ready && !r.included && r.status == "Name conflict")
        );
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn guessed_names_omit_parenthesized_metadata() {
        for (folder, expected) in [
            (
                r"E:\Anime\Cyberpunk Edgerunners (Series)",
                "Cyberpunk Edgerunners",
            ),
            (r"E:\Anime\Show (2026) (Dual Audio)\s2", "Show"),
            (r"E:\Anime\Show (BD (1080p)) Season 2", "Show"),
            (r"E:\Anime\A (note) B", "A B"),
        ] {
            assert_eq!(guess_name(Path::new(folder)), expected);
        }
        let rows = plan(
            Path::new(r"E:\Anime\Show"),
            "My (Custom) Name",
            &[PathBuf::from(r"E:\Anime\Show\E01.mkv")],
            &Settings::default(),
        )
        .unwrap();
        assert_eq!(rows[0].output, "My (Custom) Name S01E01.mkv");
    }
    #[test]
    fn conflicts_never_overwrite() {
        let root = fixture();
        let a = root.join("a.mkv");
        let b = root.join("b.mkv");
        fs::write(&a, b"a").unwrap();
        fs::write(&b, b"b").unwrap();
        assert!(
            apply(&[Change {
                source: a.clone(),
                target: b.clone()
            }])
            .is_err()
        );
        assert_eq!(fs::read(&a).unwrap(), b"a");
        assert_eq!(fs::read(&b).unwrap(), b"b");
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn ambiguous_and_invalid() {
        let root = fixture();
        for n in ["A S02E01.mkv", "A S03E02.mkv", "Ep-03.mkv"] {
            fs::write(root.join(n), "").unwrap()
        }
        let files = scan(&root, &Settings::default()).unwrap();
        let rows = plan(&root, "Show", &files, &Settings::default()).unwrap();
        assert!(
            rows.iter().any(|r| r.output == "Season is ambiguous"),
            "{rows:?}"
        );
        assert!(plan(&root, "", &files, &Settings::default()).is_err());
        assert!(plan(&root, "Bad:name", &files, &Settings::default()).is_err());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn filters_and_natural_sort() {
        let root = fixture();
        fs::create_dir(root.join("sub")).unwrap();
        for n in ["raw1.mkv", "raw10.mkv", "raw2.mkv"] {
            fs::write(root.join(n), "").unwrap()
        }
        fs::write(root.join("sub/extra.mp4"), "").unwrap();
        let p = Settings {
            subfolders: false,
            ..Default::default()
        };
        let files = scan(&root, &p).unwrap();
        assert_eq!(files.len(), 3);
        assert!(files[1].ends_with("raw2.mkv"));
        let p = Settings {
            filter_extensions: vec!["mp4".into()],
            ..Default::default()
        };
        assert_eq!(scan(&root, &p).unwrap().len(), 1);
        fs::remove_dir_all(root).unwrap();
    }
}
