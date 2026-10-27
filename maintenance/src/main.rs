//! Installer-only helper. Never scans or deletes user video folders.
#[path = "../../src/elevation.rs"]
mod elevation;
mod leftovers;
mod local_signing;
mod modern;
mod shell_locks;
mod trust;
use std::{
    fs,
    path::{Path, PathBuf},
};
use windows_sys::Win32::{
    Foundation::{CloseHandle, INVALID_HANDLE_VALUE},
    System::{
        Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
            TH32CS_SNAPPROCESS,
        },
        RemoteDesktop::ProcessIdToSessionId,
        Threading::GetCurrentProcessId,
    },
};
fn running() -> Result<bool, String> {
    // Only inspect process names in this logon session. Never terminate a process.
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return Err(std::io::Error::last_os_error().to_string());
        }
        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of_val(&entry) as u32;
        let mut session = 0;
        if ProcessIdToSessionId(GetCurrentProcessId(), &mut session) == 0 {
            CloseHandle(snapshot);
            return Err(std::io::Error::last_os_error().to_string());
        }
        let mut found = false;
        let mut valid = Process32FirstW(snapshot, &mut entry) != 0;
        while valid {
            let end = entry
                .szExeFile
                .iter()
                .position(|c| *c == 0)
                .unwrap_or(entry.szExeFile.len());
            if String::from_utf16_lossy(&entry.szExeFile[..end])
                .eq_ignore_ascii_case("anemorph.exe")
            {
                let mut other = u32::MAX;
                if ProcessIdToSessionId(entry.th32ProcessID, &mut other) != 0 && other == session {
                    found = true;
                    break;
                }
            }
            valid = Process32NextW(snapshot, &mut entry) != 0;
        }
        CloseHandle(snapshot);
        Ok(found)
    }
}
fn clear_profile(path: &Path, parent: &Path) -> Result<(), String> {
    // Resolve and check the parent before removal. The leaf is fixed, never a user-supplied folder.
    if path.file_name().is_none_or(|name| name != "EpisodeRenamer")
        || path.parent().and_then(|p| p.canonicalize().ok())
            != Some(parent.canonicalize().map_err(|e| e.to_string())?)
    {
        return Err("Refusing to remove data outside Anemorph's profile folder.".into());
    }
    match fs::symlink_metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.to_string()),
        Ok(_) => {
            // Rust's remove_dir_all does not follow directory links or junctions.
            fs::remove_dir_all(path).map_err(|e| format!("Could not remove Anemorph data: {e}"))
        }
    }
}
fn run() -> Result<i32, String> {
    match std::env::args().nth(1).as_deref() {
        Some("modern-status") => Ok(if modern::enabled()? { 0 } else { 10 }),
        Some("trust-ready") => Ok(if trust::available() { 0 } else { 10 }),
        Some("trust-modern") => {
            trust::change(true)?;
            Ok(0)
        }
        Some("remove-modern-trust") => {
            trust::change(false)?;
            Ok(0)
        }
        Some("clean-integration-trust") => {
            if trust::available() && trust::owned() {
                elevation::run(
                    &std::env::current_exe().map_err(|e| e.to_string())?,
                    "remove-modern-trust",
                )?;
            }
            Ok(0)
        }
        Some("release-shell") => {
            let root = std::env::args_os()
                .nth(2)
                .map(PathBuf::from)
                .ok_or("Missing installation directory.")?;
            shell_locks::release(&root)?;
            Ok(0)
        }
        Some("shell-locked") => {
            let root = std::env::args_os()
                .nth(2)
                .map(PathBuf::from)
                .ok_or("Missing installation directory.")?;
            Ok(if shell_locks::locked(&root)? { 10 } else { 0 })
        }
        Some("check-running") => Ok(if running()? { 10 } else { 0 }),
        Some("register-modern") => {
            let root = std::env::args_os()
                .nth(2)
                .map(PathBuf::from)
                .ok_or("Missing installation directory.")?;
            modern::register(&root)?;
            Ok(0)
        }
        Some("unregister-modern") => {
            modern::unregister()?;
            Ok(0)
        }
        Some("clean-local-signing") => {
            local_signing::remove()?;
            Ok(0)
        }
        Some("clean-data") => {
            if running()? {
                return Err("Close Anemorph before removing its saved data.".into());
            }
            let parent = std::env::var_os("LOCALAPPDATA")
                .map(PathBuf::from)
                .ok_or("Windows local app-data directory is unavailable.")?;
            clear_profile(&parent.join("EpisodeRenamer"), &parent)?;
            Ok(0)
        }
        Some("clean-leftovers") => {
            if running()? {
                return Err("Close Anemorph before removing leftovers.".into());
            }
            leftovers::clean()?;
            Ok(0)
        }
        _ => Err("Unknown maintenance action.".into()),
    }
}
fn main() {
    match run() {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_locked_profile_is_reported_and_can_be_retried() {
        use std::os::windows::fs::OpenOptionsExt;
        let root =
            std::env::temp_dir().join(format!("anemorph-locked-profile-{}", std::process::id()));
        let data = root.join("EpisodeRenamer");
        fs::create_dir_all(&data).unwrap();
        let file = data.join("settings.json");
        fs::write(&file, b"settings").unwrap();
        let lock = fs::OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(&file)
            .unwrap();
        assert!(clear_profile(&data, &root).is_err());
        assert!(data.exists());
        drop(lock);
        clear_profile(&data, &root).unwrap();
        assert!(!data.exists());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn cleanup_removes_all_owned_data_and_preserves_adjacent_files() {
        let root =
            std::env::temp_dir().join(format!("anemorph-maintenance-{}", std::process::id()));
        let data = root.join("EpisodeRenamer");
        fs::create_dir_all(data.join("Selections")).unwrap();
        fs::write(data.join("settings.json"), b"{}").unwrap();
        fs::write(data.join("Selections/request.json"), b"{}").unwrap();
        fs::write(data.join("native-last-batch.json"), b"{}").unwrap();
        let video = root.join("Video.mkv");
        fs::write(&video, b"video").unwrap();
        assert!(clear_profile(&root, &root).is_err());
        assert!(clear_profile(&data, &root.join("Selections")).is_err());
        clear_profile(&data, &root).unwrap();
        assert!(!data.exists());
        assert!(video.is_file());
        clear_profile(&data, &root).unwrap();
        fs::remove_dir_all(root).unwrap();
    }
}
