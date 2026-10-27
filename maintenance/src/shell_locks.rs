//! Release only Anemorph's hashed shell DLLs, after setup asks permission to restart Explorer.
use std::{
    os::windows::ffi::OsStrExt,
    path::{Path, PathBuf},
};
use windows_sys::Win32::{
    Foundation::*,
    System::{RestartManager::*, Threading::*},
};
struct Session(u32);
impl Drop for Session {
    fn drop(&mut self) {
        unsafe {
            RmEndSession(self.0);
        }
    }
}
fn dlls(root: &Path) -> Result<Vec<PathBuf>, String> {
    if !root.is_absolute() {
        return Err("Installation path must be absolute.".into());
    }
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let mut files = Vec::new();
    for entry in std::fs::read_dir(&root).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
        if let Some(hash) = name
            .strip_prefix("anemorph-shell-")
            .and_then(|s| s.strip_suffix(".dll"))
            && hash.len() == 12
            && hash.bytes().all(|b| b.is_ascii_hexdigit())
            && entry.file_type().map_err(|e| e.to_string())?.is_file()
        {
            files.push(entry.path());
        }
    }
    Ok(files)
}
fn session(files: &[PathBuf]) -> Result<Session, String> {
    unsafe {
        let mut handle = 0;
        let mut key = [0u16; 33];
        let code = RmStartSession(&mut handle, 0, key.as_mut_ptr());
        if code != 0 {
            return Err(format!("Cannot start Windows Restart Manager: {code}"));
        }
        let s = Session(handle);
        let names: Vec<Vec<u16>> = files
            .iter()
            .map(|p| p.as_os_str().encode_wide().chain(Some(0)).collect())
            .collect();
        let pointers: Vec<_> = names.iter().map(|s| s.as_ptr()).collect();
        if !files.is_empty() {
            let code = RmRegisterResources(
                handle,
                pointers.len() as u32,
                pointers.as_ptr(),
                0,
                std::ptr::null(),
                0,
                std::ptr::null(),
            );
            if code != 0 {
                return Err(format!("Cannot inspect menu DLL locks: {code}"));
            }
        }
        Ok(s)
    }
}
fn processes(s: &Session) -> Result<Vec<RM_PROCESS_INFO>, String> {
    unsafe {
        for _ in 0..3 {
            let mut needed = 0;
            let mut count = 0;
            let mut reasons = 0;
            let code = RmGetList(
                s.0,
                &mut needed,
                &mut count,
                std::ptr::null_mut(),
                &mut reasons,
            );
            if code == 0 {
                return Ok(Vec::new());
            }
            if code != ERROR_MORE_DATA || needed > 1024 {
                return Err(format!("Cannot inspect Explorer locks: {code}"));
            }
            let mut items = vec![RM_PROCESS_INFO::default(); needed as usize];
            count = needed;
            let code = RmGetList(
                s.0,
                &mut needed,
                &mut count,
                items.as_mut_ptr(),
                &mut reasons,
            );
            if code == ERROR_MORE_DATA {
                continue;
            }
            if code != 0 {
                return Err(format!("Cannot inspect Explorer locks: {code}"));
            }
            items.truncate(count as usize);
            return Ok(items);
        }
        Err("Menu locks changed during cleanup. Please retry.".into())
    }
}
pub fn locked(root: &Path) -> Result<bool, String> {
    let files = dlls(root)?;
    if files.is_empty() {
        return Ok(false);
    }
    Ok(!processes(&session(&files)?)?.is_empty())
}
pub fn release(root: &Path) -> Result<(), String> {
    let files = dlls(root)?;
    if files.is_empty() {
        return Ok(());
    }
    let s = session(&files)?;
    let processes = processes(&s)?;
    let windows = PathBuf::from(std::env::var_os("WINDIR").ok_or("Windows directory unavailable")?);
    let allowed = [
        windows.join("explorer.exe"),
        windows.join("System32/dllhost.exe"),
        windows.join("SysWOW64/dllhost.exe"),
    ];
    for process in &processes {
        unsafe {
            let handle = OpenProcess(
                PROCESS_QUERY_LIMITED_INFORMATION,
                0,
                process.Process.dwProcessId,
            );
            if handle.is_null() {
                return Err("Cannot verify the process holding Anemorph's menu.".into());
            }
            let mut path = vec![0u16; 32768];
            let mut size = path.len() as u32;
            let ok = QueryFullProcessImageNameW(handle, 0, path.as_mut_ptr(), &mut size);
            CloseHandle(handle);
            let path = String::from_utf16_lossy(&path[..size as usize]);
            if ok == 0
                || !allowed
                    .iter()
                    .any(|p| p.to_string_lossy().eq_ignore_ascii_case(&path))
            {
                return Err("Another application is using the menu DLL. Close it and retry. No applications were stopped.".into());
            }
        }
    }
    if processes.is_empty() {
        return Ok(());
    }
    // Explorer requires forced shutdown through Restart Manager. Only the verified shell hosts
    // holding these exact DLLs are registered; setup obtains explicit consent before this call.
    let stopped = unsafe { RmShutdown(s.0, RmForceShutdown as u32, None) };
    let removal = if stopped == 0 {
        files
            .iter()
            .try_for_each(|p| match std::fs::remove_file(p) {
                Ok(()) => Ok(()),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(e) => Err(e.to_string()),
            })
    } else {
        Err(format!("Windows could not release the menu: {stopped}"))
    };
    let restarted = unsafe { RmRestart(s.0, 0, None) };
    if restarted != 0 {
        return Err(format!(
            "Windows reported an Explorer restart error: {restarted}"
        ));
    }
    removal
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_accepts_owned_hash_names() {
        let root = std::env::temp_dir().join(format!("anemorph-dll-filter-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        for name in [
            "anemorph-shell-012345abcdef.dll",
            "anemorph-shell-user.dll",
            "video.mkv",
        ] {
            std::fs::write(root.join(name), b"fixture").unwrap();
        }
        assert_eq!(
            dlls(&root).unwrap(),
            [root
                .join("anemorph-shell-012345abcdef.dll")
                .canonicalize()
                .unwrap()]
        );
        assert!(!locked(&root).unwrap());
        assert!(release(Path::new("relative")).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}
