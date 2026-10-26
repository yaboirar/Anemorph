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
    pub dark_mode: bool,
    pub compact: bool,
    pub auto_detect: bool,
    pub auto_name: bool,
    pub clear_series_on_empty: bool,
    pub number_format: String,
    pub subfolders: bool,
    pub mkv: bool,
    pub mp4: bool,
    pub other_video: bool,
    pub remember_folder: bool,
    pub animations: bool,
    pub default_season: u32,
    pub first_episode: u32,
    pub zoom_percent: u32,
    pub accent_hex: Option<String>,
    pub last_folder: Option<PathBuf>,
    pub window_width: f32,
    pub window_height: f32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            dark_mode: true,
            compact: false,
            auto_detect: true,
            auto_name: true,
            clear_series_on_empty: true,
            number_format: crate::naming::DEFAULT_FORMAT.into(),
            subfolders: true,
            mkv: true,
            mp4: true,
            other_video: true,
            remember_folder: false,
            animations: true,
            default_season: 1,
            first_episode: 1,
            zoom_percent: 100,
            accent_hex: None,
            last_folder: None,
            window_width: 1080.,
            window_height: 740.,
        }
    }
}
impl Settings {
    pub fn load(path: &Path) -> Self {
        let mut s: Self = read_bounded(path, 64 * 1024)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        s.zoom_percent = s.zoom_percent.clamp(75, 150);
        s.number_format = crate::naming::migrate_legacy(&s.number_format);
        s.default_season = s.default_season.min(999);
        s.first_episode = s.first_episode.min(9999);
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
#[derive(Clone, Debug)]
pub struct Row {
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
        .or_else(|| CROSS.captures(name))
        .and_then(|c| Some((c["s"].parse().ok()?, c["e"].parse().ok()?)))
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
        if match ext.as_str() {
            "mkv" => settings.mkv,
            "mp4" => settings.mp4,
            "avi" | "m4v" | "webm" => settings.other_video,
            _ => false,
        } {
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
    let name = name.trim();
    if name.is_empty() {
        return Err("Enter the series name to preview filenames.".into());
    }
    if name.chars().any(|c| c < ' ' || "<>:\"/\\|?*".contains(c)) || name.ends_with('.') {
        return Err(
            "The series name contains a character Windows cannot use in a filename.".into(),
        );
    }
    let number_format = crate::naming::Format::parse(&settings.number_format)?;
    let mut siblings: HashMap<PathBuf, BTreeSet<u32>> = HashMap::new();
    // Parse each filename once and reuse its result for sibling inference and output.
    let detected: Vec<_> = files
        .iter()
        .map(|p| {
            if !settings.auto_detect {
                return (None, None);
            }
            let n = p.file_stem().unwrap_or_default().to_string_lossy();
            let known = explicit(&n)
                .map(|(s, e)| (Some(s), e))
                .or_else(|| number_format.detect(&n));
            let season = known.and_then(|(s, _)| s);
            let episode = known.map(|(_, e)| e).or_else(|| {
                EPISODE
                    .iter()
                    .find_map(|r| r.captures(&n).and_then(|c| c["e"].parse().ok()))
            });
            (season, episode)
        })
        .collect();
    for (p, (season, _)) in files.iter().zip(&detected) {
        if let Some(s) = *season {
            siblings
                .entry(p.parent().unwrap().into())
                .or_default()
                .insert(s);
        }
    }
    let mut rows = Vec::with_capacity(files.len());
    let mut seasons = HashMap::new();
    for (index, p) in files.iter().enumerate() {
        let mut row = Row {
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
        let (season, episode) = if !settings.auto_detect {
            (
                settings.default_season,
                settings
                    .first_episode
                    .checked_add(u32::try_from(index).map_err(|_| "Too many files")?)
                    .ok_or("Episode number exceeds the supported range")?,
            )
        } else {
            let (known_season, ep) = detected[index];
            let Some(ep) = ep else {
                rows.push(row);
                continue;
            };
            if ep == 480 || ep == 720 {
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
        };
        row.output = format!(
            "{name} {}.{}",
            number_format.render(season, episode),
            p.extension().unwrap_or_default().to_string_lossy()
        );
        if row.output.encode_utf16().count() > 255 {
            return Err(
                "The resulting filename is too long. Shorten the series name or format.".into(),
            );
        }
        let target = p.with_file_name(&row.output);
        row.ready = key(p) != key(&target);
        row.included = row.ready;
        row.status = if !row.ready {
            "Already named".into()
        } else if row.assumed {
            format!("S{season:02} assumed").into()
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
    Ok(changes.len())
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
        let rows = plan(&root, "Show", &[file.clone()], &settings).unwrap();
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
        let refreshed = plan(&root, "Show", &[target.clone()], &settings).unwrap();
        assert_eq!(refreshed[0].status, "Already named");
        assert_eq!(refreshed[0].output, rows[0].output);
        assert_eq!(fs::read(&target).unwrap(), b"unchanged video");
        undo(&history).unwrap();
        assert!(file.exists());
        let manual = Settings {
            auto_detect: false,
            default_season: 3,
            first_episode: 7,
            number_format: "[FixNum][S][SeaNum][Space][E][EpsNum]".into(),
            ..Default::default()
        };
        assert_eq!(
            plan(&root, "Show", &[file], &manual).unwrap()[0].output,
            "Show S3 E7.MP4"
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
            mkv: false,
            ..Default::default()
        };
        assert_eq!(scan(&root, &p).unwrap().len(), 1);
        fs::remove_dir_all(root).unwrap();
    }
}
