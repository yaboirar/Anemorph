//! Ownership-based scan. No drive-wide filename matching or shared-log deletion.
use std::{
    ffi::OsString,
    fs,
    os::windows::ffi::{OsStrExt, OsStringExt},
    path::{Path, PathBuf},
};
use windows_sys::Win32::{
    Foundation::{ERROR_FILE_NOT_FOUND, ERROR_NO_MORE_ITEMS, ERROR_PATH_NOT_FOUND},
    System::Registry::*,
};
fn wide(s: &std::ffi::OsStr) -> Vec<u16> {
    s.encode_wide().chain(Some(0)).collect()
}
struct Key(HKEY);
impl Drop for Key {
    fn drop(&mut self) {
        unsafe {
            RegCloseKey(self.0);
        }
    }
}
fn open(key: &str, write: bool) -> Result<Option<Key>, String> {
    let mut handle = std::ptr::null_mut();
    let name = wide(key.as_ref());
    let status = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            name.as_ptr(),
            0,
            KEY_READ | KEY_WOW64_64KEY | if write { KEY_SET_VALUE } else { 0 },
            &mut handle,
        )
    };
    if status == ERROR_FILE_NOT_FOUND || status == ERROR_PATH_NOT_FOUND {
        return Ok(None);
    }
    if status != 0 {
        return Err(format!(
            "Cannot access Anemorph cleanup location {key}: Windows error {status}"
        ));
    }
    Ok(Some(Key(handle)))
}
fn recorded_locations() -> Result<Vec<PathBuf>, String> {
    let Some(key) = open("Software\\Anemorph\\InstallLocations", false)? else {
        return Ok(vec![]);
    };
    let mut paths = Vec::new();
    for index in 0..1024 {
        let mut name = vec![0u16; 32768];
        let mut length = name.len() as u32;
        let status = unsafe {
            RegEnumValueW(
                key.0,
                index,
                name.as_mut_ptr(),
                &mut length,
                std::ptr::null(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        if status == ERROR_NO_MORE_ITEMS {
            return Ok(paths);
        }
        if status != 0 {
            return Err(format!(
                "Cannot read installation inventory: Windows error {status}"
            ));
        }
        let path = PathBuf::from(OsString::from_wide(&name[..length as usize]));
        if !path.is_absolute() || path.components().count() < 3 {
            return Err("Invalid recorded Anemorph installation folder.".into());
        }
        paths.push(path);
    }
    Err("Installation inventory exceeds the supported limit.".into())
}
fn remove_file(path: &Path, count: &mut usize) -> Result<(), String> {
    if path.file_name().is_some_and(|n| {
        let n = n.to_string_lossy().to_ascii_lowercase();
        n == "anemorph-broker.exe" || n.starts_with("anemorph-shell-") && n.ends_with(".dll")
    }) {
        crate::shell_locks::unlink_image(path)?;
        *count += 1;
        return Ok(());
    }
    match fs::remove_file(path) {
        Ok(()) => {
            *count += 1;
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("Could not remove {}: {e}", path.display())),
    }
}
fn program_leftovers(root: &Path, count: &mut usize) -> Result<(), String> {
    let metadata = match fs::symlink_metadata(root) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e.to_string()),
    };
    if metadata.file_type().is_symlink() {
        return Err(format!(
            "Recorded installation is a link; cleanup stopped: {}",
            root.display()
        ));
    }
    let integration = root.join("Anemorph.Integration");
    let assets = root.join("assets");
    let screenshots = assets.join("screenshots");
    if fs::symlink_metadata(&assets).is_ok_and(|m| m.is_dir() && !m.file_type().is_symlink())
        && fs::symlink_metadata(&screenshots)
            .is_ok_and(|m| m.is_dir() && !m.file_type().is_symlink())
    {
        for name in ["home.jpg", "explorer-menu.png"] {
            remove_file(&screenshots.join(name), count)?;
        }
        for folder in [&screenshots, &assets] {
            match fs::remove_dir(folder) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::DirectoryNotEmpty => {}
                Err(e) => return Err(e.to_string()),
            }
        }
    }
    let docs = root.join("docs");
    if fs::symlink_metadata(&docs).is_ok_and(|m| !m.file_type().is_symlink() && m.is_dir()) {
        for name in [
            "usage.md",
            "building.md",
            "performance.md",
            "signing.md",
            "contributing.md",
            "changelog.md",
        ] {
            remove_file(&docs.join(name), count)?;
        }
        let licenses = docs.join("licenses");
        if fs::symlink_metadata(&licenses).is_ok_and(|m| m.is_dir() && !m.file_type().is_symlink())
        {
            for name in ["third-party-notices.md", "dependency-licenses.md"] {
                remove_file(&licenses.join(name), count)?;
            }
            match fs::remove_dir(&licenses) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::DirectoryNotEmpty => {}
                Err(e) => return Err(e.to_string()),
            }
        }
        match fs::remove_dir(&docs) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::DirectoryNotEmpty => {}
            Err(e) => return Err(e.to_string()),
        }
    }
    if fs::symlink_metadata(&integration).is_ok_and(|m| !m.file_type().is_symlink() && m.is_dir()) {
        for name in ["Anemorph.identity.msix", "icon.png"] {
            remove_file(&integration.join(name), count)?;
        }
        match fs::remove_dir(&integration) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::DirectoryNotEmpty => {}
            Err(e) => return Err(e.to_string()),
        }
    }
    for entry in fs::read_dir(root).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
        let owned = [
            "anemorph.exe",
            "anemorph-maintenance.exe",
            "anemorph-broker.exe",
            "uninstall.exe",
            "license",
            "readme.md",
            "changelog.md",
            "third_party_notices.md",
            "dependency_licenses.md",
            "nsis-license.txt",
        ]
        .contains(&name.as_str())
            || (name.starts_with("anemorph-shell-") && name.ends_with(".dll"));
        if owned && !entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            remove_file(&entry.path(), count)?;
        }
    }
    // A custom install folder may contain unrelated user files. Never recursively delete it.
    match fs::remove_dir(root) {
        Ok(()) => Ok(()),
        Err(e)
            if matches!(
                e.kind(),
                std::io::ErrorKind::NotFound | std::io::ErrorKind::DirectoryNotEmpty
            ) =>
        {
            Ok(())
        }
        Err(e) => Err(e.to_string()),
    }
}
pub fn clean() -> Result<(), String> {
    let locations = recorded_locations()?;
    let mut count = 0;
    println!("Scanning recorded Anemorph folders and app-specific Windows caches...");
    for root in &locations {
        program_leftovers(root, &mut count)?;
    }
    let mut executables: Vec<_> = locations
        .iter()
        .flat_map(|root| [root.join("Anemorph.exe"), root.join("Uninstall.exe")])
        .collect();
    if let Some(key) = open(
        r"Software\Classes\Local Settings\Software\Microsoft\Windows\Shell\MuiCache",
        false,
    )? {
        executables.extend(discover_cached_executables(&key)?);
    }
    executables.sort();
    executables.dedup();
    // Delete only exact executable-path values, preserving all other apps' cached entries.
    for key_name in [
        r"Software\Classes\Local Settings\Software\Microsoft\Windows\Shell\MuiCache",
        r"Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Compatibility Assistant\Store",
    ] {
        let Some(key) = open(key_name, true)? else {
            continue;
        };
        for path in &executables {
            for suffix in if key_name.ends_with("MuiCache") {
                &[".FriendlyAppName", ".ApplicationCompany"][..]
            } else {
                &[""][..]
            } {
                let mut name = path.as_os_str().to_os_string();
                name.push(suffix);
                let name = wide(&name);
                let status = unsafe { RegDeleteValueW(key.0, name.as_ptr()) };
                if status == 0 {
                    count += 1;
                } else if status != ERROR_FILE_NOT_FOUND {
                    return Err(format!(
                        "Could not remove an Anemorph cache entry: Windows error {status}"
                    ));
                }
            }
        }
    }
    println!("Removed {count} verified leftover items.");
    Ok(())
}
fn discover_cached_executables(key: &Key) -> Result<Vec<PathBuf>, String> {
    let mut result = Vec::new();
    let mut name = vec![0u16; 32768];
    let mut data = vec![0u16; 32768];
    for index in 0..65536 {
        let mut length = name.len() as u32;
        let mut bytes = (data.len() * 2) as u32;
        let mut kind = 0;
        let status = unsafe {
            RegEnumValueW(
                key.0,
                index,
                name.as_mut_ptr(),
                &mut length,
                std::ptr::null(),
                &mut kind,
                data.as_mut_ptr().cast(),
                &mut bytes,
            )
        };
        if status == ERROR_NO_MORE_ITEMS {
            return Ok(result);
        }
        if status != 0 {
            return Err(format!(
                "Cannot finish app-cache scan: Windows error {status}"
            ));
        }
        if kind != REG_SZ {
            continue;
        }
        let text = String::from_utf16_lossy(&data[..bytes as usize / 2])
            .trim_end_matches('\0')
            .to_ascii_lowercase();
        if !["anemorph", "anemorph episode renamer", "anemorph installer"].contains(&text.as_str())
        {
            continue;
        }
        let count = length as usize;
        let label = String::from_utf16_lossy(&name[..count]).to_ascii_lowercase();
        let suffix = ".friendlyappname";
        if !label.ends_with(suffix) || count < suffix.len() {
            continue;
        }
        let path = PathBuf::from(OsString::from_wide(&name[..count - suffix.len()]));
        if path.is_absolute()
            && path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("exe"))
        {
            result.push(path);
        }
    }
    Err("App-cache scan exceeds the supported entry limit.".into())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn old_install_scan_preserves_unrelated_files() {
        let root =
            std::env::temp_dir().join(format!("anemorph-leftover-scan-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("Anemorph.exe"), b"app").unwrap();
        fs::write(
            root.join("anemorph-shell-012345abcdef.dll"),
            b"old component",
        )
        .unwrap();
        fs::write(root.join("My video.mkv"), b"video").unwrap();
        let screenshots = root.join("assets/screenshots");
        fs::create_dir_all(&screenshots).unwrap();
        for name in ["home.jpg", "explorer-menu.png", "Keep.png"] {
            fs::write(screenshots.join(name), b"fixture").unwrap();
        }
        fs::create_dir_all(root.join("docs")).unwrap();
        fs::write(root.join("docs/performance.md"), b"report").unwrap();
        fs::write(root.join("docs/Keep.md"), b"unrelated").unwrap();
        fs::write(root.join("docs/changelog.md"), b"changelog").unwrap();
        fs::create_dir_all(root.join("docs/licenses")).unwrap();
        fs::write(
            root.join("docs/licenses/dependency-licenses.md"),
            b"licenses",
        )
        .unwrap();
        fs::write(root.join("docs/licenses/Keep.txt"), b"unrelated").unwrap();
        let mut count = 0;
        program_leftovers(&root, &mut count).unwrap();
        assert_eq!(count, 7);
        assert!(!root.join("docs/licenses/dependency-licenses.md").exists());
        assert!(!root.join("docs/changelog.md").exists());
        assert!(root.join("docs/licenses/Keep.txt").is_file());
        assert!(root.join("docs/Keep.md").is_file());
        assert!(screenshots.join("Keep.png").is_file());
        assert!(root.join("My video.mkv").is_file());
        fs::remove_dir_all(root).unwrap();
    }
}
