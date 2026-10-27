//! Legacy image cleanup only. Never restart or stop Windows processes.
use std::path::{Path, PathBuf};
fn owned(name: &str) -> bool {
    name == "anemorph-broker.exe"
        || name
            .strip_prefix("anemorph-shell-")
            .and_then(|s| s.strip_suffix(".dll"))
            .is_some_and(|s| s.len() == 12 && s.bytes().all(|b| b.is_ascii_hexdigit()))
}
pub fn unlink_image(path: &Path) -> Result<(), String> {
    if !path.is_absolute()
        || !path
            .file_name()
            .is_some_and(|s| owned(&s.to_string_lossy().to_ascii_lowercase()))
    {
        return Err("Refusing to remove an unrecognized menu image.".into());
    }
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!(
            "An older menu component is still in use: {}. No Windows processes were stopped. Restart Windows normally, then retry cleanup. Details: {e}",
            path.display()
        )),
    }
}
fn images(root: &Path) -> Result<Vec<PathBuf>, String> {
    if !root.is_absolute() {
        return Err("Installation path must be absolute.".into());
    }
    if !root.exists() {
        return Ok(Vec::new());
    }
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let mut files = Vec::new();
    for entry in std::fs::read_dir(&root).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if owned(&entry.file_name().to_string_lossy().to_ascii_lowercase())
            && entry.file_type().map_err(|e| e.to_string())?.is_file()
        {
            files.push(entry.path());
        }
    }
    Ok(files)
}
pub fn release(root: &Path) -> Result<(), String> {
    crate::broker_control::stop(root)?;
    for image in images(root)? {
        unlink_image(&image)?;
    }
    Ok(())
}
pub fn prune(root: &Path) -> Result<(), String> {
    crate::broker_control::stop(root)?;
    for image in images(root)? {
        if image
            .file_name()
            .is_some_and(|s| s == "anemorph-broker.exe")
        {
            continue;
        }
        if let Err(error) = unlink_image(&image) {
            println!("Retained a legacy image for later cleanup: {error}");
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_loaded_legacy_image_is_reported_without_stopping_its_host() {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::{Foundation::FreeLibrary, System::LibraryLoader::*};
        let root =
            std::env::temp_dir().join(format!("anemorph-loaded-image-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let image = root.join("anemorph-shell-012345abcdef.dll");
        let system =
            PathBuf::from(std::env::var_os("WINDIR").unwrap()).join("System32/version.dll");
        std::fs::copy(system, &image).unwrap();
        let video = root.join("Keep.mkv");
        std::fs::write(&video, b"keep").unwrap();
        let wide: Vec<u16> = image.as_os_str().encode_wide().chain(Some(0)).collect();
        unsafe {
            let module = LoadLibraryW(wide.as_ptr());
            assert!(!module.is_null());
            assert!(unlink_image(&image).is_err());
            assert!(image.exists());
            assert!(GetProcAddress(module, c"GetFileVersionInfoSizeW".as_ptr().cast()).is_some());
            FreeLibrary(module);
        }
        unlink_image(&image).unwrap();
        assert!(!image.exists());
        assert!(unlink_image(&video).is_err());
        assert!(video.exists());
        std::fs::remove_dir_all(root).unwrap();
    }
}
