//! Session-local shutdown for Anemorph's own out-of-process menu host.
use std::path::Path;
pub fn event_name(root: &Path) -> Result<String, String> {
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let mut hash = 0xcbf29ce484222325u64;
    for byte in root.to_string_lossy().to_lowercase().as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    Ok(format!(r"Local\Anemorph.Broker.{hash:016x}"))
}
#[allow(dead_code)]
pub fn stop(root: &Path) -> Result<(), String> {
    use windows_sys::Win32::{Foundation::*, System::Threading::*};
    if !root.exists() {
        return Ok(());
    }
    let name = event_name(root)?;
    let wide = |s: &str| s.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
    unsafe {
        let stop = OpenEventW(EVENT_MODIFY_STATE, 0, wide(&name).as_ptr());
        if stop.is_null() {
            return if GetLastError() == ERROR_FILE_NOT_FOUND {
                Ok(())
            } else {
                Err("Could not contact Anemorph's menu host.".into())
            };
        }
        let done = OpenEventW(
            SYNCHRONIZATION_SYNCHRONIZE,
            0,
            wide(&(name + ".Stopped")).as_ptr(),
        );
        let signalled = SetEvent(stop);
        CloseHandle(stop);
        let wait = if !done.is_null() && signalled != 0 {
            WaitForSingleObject(done, 5000)
        } else {
            WAIT_FAILED
        };
        if !done.is_null() {
            CloseHandle(done);
        }
        if wait != WAIT_OBJECT_0 {
            return Err("Anemorph's menu host is finishing a request. Retry in a moment.".into());
        }
    }
    wait_for_exit(root)?;
    Ok(())
}
#[allow(dead_code)]
fn wait_for_exit(root: &Path) -> Result<(), String> {
    use windows_sys::Win32::{
        Foundation::*,
        System::{Diagnostics::ToolHelp::*, Threading::*},
    };
    let expected = root
        .join("anemorph-broker.exe")
        .canonicalize()
        .map_err(|e| e.to_string())?;
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return Err("Cannot verify menu host shutdown.".into());
        }
        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..std::mem::zeroed()
        };
        let mut valid = Process32FirstW(snapshot, &mut entry) != 0;
        let mut handles = Vec::new();
        while valid {
            let end = entry
                .szExeFile
                .iter()
                .position(|c| *c == 0)
                .unwrap_or(entry.szExeFile.len());
            if String::from_utf16_lossy(&entry.szExeFile[..end])
                .eq_ignore_ascii_case("anemorph-broker.exe")
            {
                let process = OpenProcess(
                    PROCESS_QUERY_LIMITED_INFORMATION | SYNCHRONIZATION_SYNCHRONIZE,
                    0,
                    entry.th32ProcessID,
                );
                if !process.is_null() {
                    let mut name = vec![0u16; 32768];
                    let mut length = name.len() as u32;
                    let ok = QueryFullProcessImageNameW(process, 0, name.as_mut_ptr(), &mut length);
                    let path = std::path::PathBuf::from(String::from_utf16_lossy(
                        &name[..length as usize],
                    ));
                    if ok != 0 && path.canonicalize().ok().as_ref() == Some(&expected) {
                        handles.push(process);
                    } else {
                        CloseHandle(process);
                    }
                }
            }
            valid = Process32NextW(snapshot, &mut entry) != 0;
        }
        CloseHandle(snapshot);
        let mut stopped = true;
        for handle in handles {
            if WaitForSingleObject(handle, 5000) != WAIT_OBJECT_0 {
                stopped = false;
            }
            CloseHandle(handle);
        }
        if !stopped {
            return Err("The menu host is still closing. Retry in a moment.".into());
        }
    }
    Ok(())
}
