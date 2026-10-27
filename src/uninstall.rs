use std::{
    os::windows::ffi::OsStringExt,
    path::{Path, PathBuf},
};
use windows_sys::Win32::System::Registry::{
    HKEY_CURRENT_USER, RRF_RT_REG_SZ, RRF_SUBKEY_WOW6464KEY, RegGetValueW,
};
pub fn installed() -> Option<PathBuf> {
    let key: Vec<_> = "Software\\Anemorph".encode_utf16().chain(Some(0)).collect();
    let name: Vec<_> = "InstallDir".encode_utf16().chain(Some(0)).collect();
    let mut buffer = vec![0u16; 32768];
    let mut bytes = (buffer.len() * 2) as u32;
    // Bounded, read-only lookup of the per-user installer registration.
    let result = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            key.as_ptr(),
            name.as_ptr(),
            RRF_RT_REG_SZ | RRF_SUBKEY_WOW6464KEY,
            std::ptr::null_mut(),
            buffer.as_mut_ptr().cast(),
            &mut bytes,
        )
    };
    if result != 0 || bytes < 2 {
        return None;
    }
    let path = PathBuf::from(std::ffi::OsString::from_wide(
        &buffer[..(bytes as usize / 2).saturating_sub(1)],
    ));
    let uninstaller = path.join("Uninstall.exe");
    (path.is_absolute() && uninstaller.is_file()).then_some(uninstaller)
}
pub fn launch(path: &Path, parent_pid: u32) -> Result<(), String> {
    if !path.is_absolute() || !path.is_file() {
        return Err("The installed uninstaller is missing. Run setup to repair Anemorph.".into());
    }
    std::process::Command::new(path)
        .arg(format!("/FROMAPP={parent_pid}"))
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Could not open the uninstaller: {e}"))
}
