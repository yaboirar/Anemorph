//! Narrow administrator handoff. Call only after an explicit user action.
use std::{os::windows::ffi::OsStrExt, path::Path};
use windows_sys::Win32::{
    Foundation::*, System::Threading::*, UI::Shell::*, UI::WindowsAndMessaging::SW_HIDE,
};
pub fn run(path: &Path, action: &str) -> Result<(), String> {
    let file: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let verb: Vec<u16> = "runas".encode_utf16().chain(Some(0)).collect();
    let args: Vec<u16> = action.encode_utf16().chain(Some(0)).collect();
    unsafe {
        let mut info: SHELLEXECUTEINFOW = std::mem::zeroed();
        info.cbSize = std::mem::size_of_val(&info) as u32;
        info.fMask = SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NOASYNC;
        info.lpVerb = verb.as_ptr();
        info.lpFile = file.as_ptr();
        info.lpParameters = args.as_ptr();
        info.nShow = SW_HIDE;
        if ShellExecuteExW(&mut info) == 0 {
            return Err(if GetLastError() == ERROR_CANCELLED {
                "Windows approval was cancelled. Nothing was enabled.".into()
            } else {
                std::io::Error::last_os_error().to_string()
            });
        }
        if info.hProcess.is_null() {
            return Err("Windows returned no integration process.".into());
        }
        let wait = WaitForSingleObject(info.hProcess, INFINITE);
        let mut exit = 1;
        let read = GetExitCodeProcess(info.hProcess, &mut exit);
        CloseHandle(info.hProcess);
        if wait != WAIT_OBJECT_0 || read == 0 || exit != 0 {
            return Err("Windows could not complete the integration change. The classic menu remains available.".into());
        }
    }
    Ok(())
}
