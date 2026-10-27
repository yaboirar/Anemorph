//! Keep packaged and classic registrations mutually exclusive.
use std::path::{Path, PathBuf};
use windows_sys::Win32::{
    Foundation::{ERROR_FILE_NOT_FOUND, ERROR_PATH_NOT_FOUND},
    System::Registry::*,
    UI::Shell::{SHCNE_ASSOCCHANGED, SHCNF_FLUSHNOWAIT, SHChangeNotify},
};
#[allow(dead_code)]
#[path = "../../src/containers.rs"]
mod containers;
const CLASS: &str = "{B8A9BDB0-03E4-4F1A-BDEA-28654A8F7DB4}";
const BROKER: &str = "{9CA3D8F3-5068-49E3-B792-94C125F1EACC}";
const CLASSES: &str = r"Software\Classes";
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
fn set(key: &str, name: &str, value: &str) -> Result<(), String> {
    unsafe {
        let mut handle = std::ptr::null_mut();
        let code = RegCreateKeyExW(
            HKEY_CURRENT_USER,
            wide(key).as_ptr(),
            0,
            std::ptr::null(),
            0,
            KEY_SET_VALUE | KEY_WOW64_64KEY,
            std::ptr::null(),
            &mut handle,
            std::ptr::null_mut(),
        );
        if code != 0 {
            return Err(format!("Cannot create Anemorph menu registration: {code}"));
        }
        let value = wide(value);
        let code = RegSetValueExW(
            handle,
            wide(name).as_ptr(),
            0,
            REG_SZ,
            value.as_ptr().cast(),
            (value.len() * 2) as u32,
        );
        RegCloseKey(handle);
        if code != 0 {
            return Err(format!("Cannot write Anemorph menu registration: {code}"));
        }
        Ok(())
    }
}
fn remove(path: &str) -> Result<(), String> {
    unsafe {
        // Open the 64-bit view explicitly. Delete only Anemorph's named key.
        let mut key = std::ptr::null_mut();
        let code = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            wide(path).as_ptr(),
            0,
            KEY_ALL_ACCESS | KEY_WOW64_64KEY,
            &mut key,
        );
        if code == ERROR_FILE_NOT_FOUND || code == ERROR_PATH_NOT_FOUND {
            return Ok(());
        }
        if code != 0 {
            return Err(format!("Cannot open Anemorph menu registration: {code}"));
        }
        let code = RegDeleteTreeW(key, std::ptr::null());
        RegCloseKey(key);
        if code != 0 {
            return Err(format!("Cannot clear Anemorph menu registration: {code}"));
        }
        let code = RegDeleteKeyExW(HKEY_CURRENT_USER, wide(path).as_ptr(), KEY_WOW64_64KEY, 0);
        if code != 0 && code != ERROR_FILE_NOT_FOUND {
            return Err(format!("Cannot remove Anemorph menu registration: {code}"));
        }
        Ok(())
    }
}
fn apply(base: &str, root: &Path, dll: Option<&Path>) -> Result<(), String> {
    remove(&format!(r"{base}\CLSID\{CLASS}"))?;
    if let Some(dll) = dll {
        set(
            &format!(r"{base}\CLSID\{BROKER}"),
            "",
            "Anemorph Explorer command",
        )?;
        set(
            &format!(r"{base}\CLSID\{BROKER}\LocalServer32"),
            "",
            &format!("\"{}\" --com-server", dll.display()),
        )?;
        set(
            &format!(r"{base}\CLSID\{BROKER}\LocalServer32"),
            "ServerExecutable",
            &dll.to_string_lossy(),
        )?;
    }
    for (ext, _) in containers::VIDEO_CONTAINERS {
        let key = format!(r"{base}\SystemFileAssociations\.{ext}\shell\Anemorph");
        remove(&key)?;
        if dll.is_some() {
            set(&key, "", "Rename with Anemorph")?;
            set(&format!(r"{key}\command"), "DelegateExecute", BROKER)?;
            set(&format!(r"{key}\command"), "", "")?;
            set(
                &key,
                "Icon",
                &format!("{},0", root.join("Anemorph.exe").display()),
            )?;
            set(&key, "MultiSelectModel", "Player")?;
        } else {
            remove(&key)?;
        }
    }
    if dll.is_none() {
        remove(&format!(r"{base}\CLSID\{BROKER}"))?;
    }
    Ok(())
}
fn notify() {
    unsafe {
        SHChangeNotify(
            SHCNE_ASSOCCHANGED as i32,
            SHCNF_FLUSHNOWAIT,
            std::ptr::null(),
            std::ptr::null(),
        );
    }
}
fn startup(root: Option<&Path>) -> Result<(), String> {
    let key = r"Software\Microsoft\Windows\CurrentVersion\Run";
    if let Some(root) = root {
        return set(
            key,
            "AnemorphMenuHost",
            &format!(
                "\"{}\" --com-server --resident",
                root.join("anemorph-broker.exe").display()
            ),
        );
    }
    unsafe {
        let mut handle = std::ptr::null_mut();
        let code = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            wide(key).as_ptr(),
            0,
            KEY_SET_VALUE | KEY_WOW64_64KEY,
            &mut handle,
        );
        if code == ERROR_FILE_NOT_FOUND {
            return Ok(());
        }
        if code != 0 {
            return Err(format!("Cannot update Anemorph menu startup: {code}"));
        }
        let code = RegDeleteValueW(handle, wide("AnemorphMenuHost").as_ptr());
        RegCloseKey(handle);
        if code != 0 && code != ERROR_FILE_NOT_FOUND {
            return Err(format!("Cannot remove Anemorph menu startup: {code}"));
        }
    }
    Ok(())
}
fn start_host(root: &Path) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::{Foundation::*, System::Threading::*};
    let executable = root.join("anemorph-broker.exe");
    let application: Vec<u16> = executable
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect();
    let mut command = wide(&format!(
        "\"{}\" --com-server --resident",
        executable.display()
    ));
    let mut startup: STARTUPINFOW = unsafe { std::mem::zeroed() };
    startup.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
    let mut process: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };
    // The resident host must inherit no handles, including redirected helper
    // pipes held by installers and callers. It never needs the parent's stdio.
    unsafe {
        if CreateProcessW(
            application.as_ptr(),
            command.as_mut_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            0,
            CREATE_NO_WINDOW,
            std::ptr::null(),
            std::ptr::null(),
            &startup,
            &mut process,
        ) == 0
        {
            return Err(std::io::Error::last_os_error().to_string());
        }
        CloseHandle(process.hThread);
        CloseHandle(process.hProcess);
    }
    let name = wide(&(crate::broker_control::event_name(root)? + ".Ready"));
    for _ in 0..100 {
        unsafe {
            let event = OpenEventW(SYNCHRONIZATION_SYNCHRONIZE, 0, name.as_ptr());
            if !event.is_null() {
                let ready = WaitForSingleObject(event, 0) == WAIT_OBJECT_0;
                CloseHandle(event);
                if ready {
                    return Ok(());
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    Err("Anemorph's menu host did not become ready. Run setup to repair integration.".into())
}
pub fn packaged() -> Result<(), String> {
    startup(None)?;
    apply(CLASSES, Path::new(""), None)?;
    remove(r"Software\Classes\AppID\{9CA3D8F3-5068-49E3-B792-94C125F1EACC}")?;
    remove(r"Software\Classes\AppID\anemorph-broker.exe")?;
    notify();
    Ok(())
}
pub fn clear() -> Result<(), String> {
    packaged()
}
fn dll(root: &Path) -> Result<PathBuf, String> {
    if !root.is_absolute() {
        return Err("The integration location must be absolute.".into());
    }
    let dll = root.join("anemorph-broker.exe");
    if !dll.is_file() {
        return Err(
            "The installed Explorer handler is missing. Run setup to repair Anemorph.".into(),
        );
    }
    Ok(dll)
}
pub fn classic(root: &Path) -> Result<(), String> {
    let dll = dll(root)?;
    crate::broker_control::stop(root)?;
    crate::modern::unregister()?;
    apply(CLASSES, root, Some(&dll))?;
    startup(Some(root))?;
    start_host(root)?;
    notify();
    Ok(())
}
pub fn sync(root: &Path) -> Result<(), String> {
    if crate::modern::enabled()? {
        packaged()
    } else {
        classic(root)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn exists(path: &str) -> bool {
        unsafe {
            let mut key = std::ptr::null_mut();
            let code = RegOpenKeyExW(
                HKEY_CURRENT_USER,
                wide(path).as_ptr(),
                0,
                KEY_READ | KEY_WOW64_64KEY,
                &mut key,
            );
            if code == 0 {
                RegCloseKey(key);
            }
            code == 0
        }
    }
    #[test]
    fn repeated_mode_switches_preserve_other_commands_and_never_leave_both() {
        let base = format!(r"Software\AnemorphMenuTest-{}", std::process::id());
        let root = Path::new(r"C:\Anemorph fixture");
        let dll = root.join("anemorph-shell-012345abcdef.dll");
        let other = format!(r"{base}\SystemFileAssociations\.mkv\shell\AnotherApp");
        set(&other, "", "Keep me").unwrap();
        for _ in 0..3 {
            apply(&base, root, Some(&dll)).unwrap();
            for (ext, _) in containers::VIDEO_CONTAINERS {
                assert!(exists(&format!(
                    r"{base}\SystemFileAssociations\.{ext}\shell\Anemorph"
                )));
            }
            assert!(exists(&format!(r"{base}\CLSID\{BROKER}\LocalServer32")));
            assert!(!exists(&format!(r"{base}\CLSID\{CLASS}")));
            apply(&base, root, None).unwrap();
            for (ext, _) in containers::VIDEO_CONTAINERS {
                assert!(!exists(&format!(
                    r"{base}\SystemFileAssociations\.{ext}\shell\Anemorph"
                )));
            }
            assert!(!exists(&format!(r"{base}\CLSID\{CLASS}")));
            assert!(!exists(&format!(r"{base}\CLSID\{BROKER}")));
            assert!(exists(&other));
        }
        remove(&base).unwrap();
    }
}
