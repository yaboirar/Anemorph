//! Bounded, per-user handoff between Explorer and the preview application.
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};
pub const MAX_FILES: usize = 10_000;
const MAX_BYTES: u64 = 16 * 1024 * 1024;
#[derive(Serialize, Deserialize)]
struct Request {
    version: u32,
    files: Vec<PathBuf>,
}
pub fn supported(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()).is_some_and(|e| {
        crate::containers::VIDEO_CONTAINERS
            .iter()
            .any(|(ext, _)| e.eq_ignore_ascii_case(ext))
    })
}
pub fn directory() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("EpisodeRenamer")
        .join("Selections")
}
#[allow(dead_code)] // Shared with the Explorer DLL; exercised by the app's parser tests.
pub fn write_request(dir: &Path, id: &str, files: &[PathBuf]) -> Result<PathBuf, String> {
    if files.is_empty()
        || files.len() > MAX_FILES
        || files.iter().any(|p| !p.is_absolute() || !supported(p))
    {
        return Err("Select supported video files (up to 10,000 per batch).".into());
    }
    if id.is_empty() || !id.bytes().all(|c| c.is_ascii_hexdigit() || c == b'-') {
        return Err("Invalid selection identifier.".into());
    }
    let bytes = serde_json::to_vec(&Request {
        version: 1,
        files: files.to_vec(),
    })
    .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err("Selection is too large.".into());
    }
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{id}.json"));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|e| e.to_string())?;
    if let Err(e) = file.write_all(&bytes) {
        drop(file);
        let _ = fs::remove_file(&path);
        return Err(e.to_string());
    }
    Ok(path)
}
pub fn read_request(path: &Path, dir: &Path) -> Result<Vec<PathBuf>, String> {
    // Only consume our own one-use handoff files, never an arbitrary CLI path.
    let parent = path.parent().and_then(|p| p.canonicalize().ok());
    if parent.is_none()
        || parent != dir.canonicalize().ok()
        || path.extension().and_then(|e| e.to_str()) != Some("json")
        || fs::symlink_metadata(path).map_or(true, |m| m.file_type().is_symlink())
    {
        return Err("Invalid Explorer selection file.".into());
    }
    let file = fs::File::open(path).map_err(|e| e.to_string())?;
    if file.metadata().map_err(|e| e.to_string())?.len() > MAX_BYTES {
        return Err("Selection is too large.".into());
    }
    let mut bytes = Vec::new();
    file.take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err("Selection is too large.".into());
    }
    let request: Request = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    if request.version != 1 || request.files.is_empty() || request.files.len() > MAX_FILES {
        return Err("Invalid Explorer selection.".into());
    }
    let files = validate(request.files)?;
    let _ = fs::remove_file(path);
    Ok(files)
}
pub fn validate(files: Vec<PathBuf>) -> Result<Vec<PathBuf>, String> {
    if files.is_empty() || files.len() > MAX_FILES {
        return Err("Select 1 to 10,000 video files.".into());
    }
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for path in files {
        if !path.is_absolute() || !supported(&path) || !path.is_file() {
            return Err("The selection contains a missing or unsupported video file.".into());
        }
        let canonical = path.canonicalize().map_err(|e| e.to_string())?;
        if seen.insert(canonical.to_string_lossy().to_lowercase()) {
            result.push(path);
        }
    }
    Ok(result)
}
pub fn common_folder(files: &[PathBuf]) -> Option<PathBuf> {
    let mut root = files.first()?.parent()?;
    while !files.iter().all(|p| p.starts_with(root)) {
        root = root.parent()?;
    }
    Some(root.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn requests_are_bounded_one_use_and_cannot_consume_unrelated_files() {
        let root = std::env::temp_dir().join(format!("anemorph-selection-{}", std::process::id()));
        let requests = root.join("requests");
        fs::create_dir_all(&root).unwrap();
        let video = root.join("日本語 & quoted ' S01E19.MKV");
        fs::write(&video, b"video").unwrap();
        let request =
            write_request(&requests, "a123-b456", &[video.clone(), video.clone()]).unwrap();
        assert!(write_request(&requests, "a123-b456", std::slice::from_ref(&video)).is_err());
        assert_eq!(
            read_request(&request, &requests).unwrap(),
            vec![video.clone()]
        );
        assert!(!request.exists());
        assert!(video.is_file());
        assert!(write_request(&requests, "../escape", std::slice::from_ref(&video)).is_err());
        let outside = root.join("unrelated.json");
        fs::write(&outside, b"{}").unwrap();
        assert!(read_request(&outside, &requests).is_err());
        assert!(outside.is_file());
        assert!(validate(vec![root.join("missing.mp4")]).is_err());
        let unsupported = root.join("file.txt");
        fs::write(&unsupported, b"text").unwrap();
        assert!(validate(vec![unsupported]).is_err());
        let oversized = requests.join("oversized.json");
        fs::File::create(&oversized)
            .unwrap()
            .set_len(MAX_BYTES + 1)
            .unwrap();
        assert!(read_request(&oversized, &requests).is_err());
        assert!(write_request(&requests, "abc", &vec![video; MAX_FILES + 1]).is_err());
        fs::remove_dir_all(root).unwrap();
    }
}
