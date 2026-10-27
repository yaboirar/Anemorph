//! Classic menu activation runs here, never inside Explorer.
#![windows_subsystem = "windows"]
#[path = "../../../src/broker_control.rs"]
mod broker_control;
use std::time::{Duration, Instant};
use windows::{
    Win32::{
        Foundation::{CloseHandle, WAIT_OBJECT_0, WAIT_TIMEOUT},
        System::{
            Com::*,
            Threading::{CreateEventW, SetEvent},
        },
        UI::WindowsAndMessaging::*,
    },
    core::{GUID, PCWSTR, Result},
};
const CLASSIC: GUID = GUID::from_u128(0x9ca3d8f3_5068_49e3_b792_94c125f1eacc);
fn run() -> Result<()> {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
        let root = std::env::current_exe()
            .map_err(|e| {
                windows::core::Error::new(windows::Win32::Foundation::E_FAIL, e.to_string())
            })?
            .parent()
            .unwrap()
            .to_path_buf();
        let name = broker_control::event_name(&root)
            .map_err(|e| windows::core::Error::new(windows::Win32::Foundation::E_FAIL, e))?;
        let wide = |s: &str| s.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
        let done = CreateEventW(
            None,
            true,
            false,
            PCWSTR(wide(&(name.clone() + ".Stopped")).as_ptr()),
        )?;
        let stop = CreateEventW(None, true, false, PCWSTR(wide(&name).as_ptr()))?;
        let factory = anemorph_shell::class_factory();
        let cookie =
            CoRegisterClassObject(&CLASSIC, &factory, CLSCTX_LOCAL_SERVER, REGCLS_MULTIPLEUSE)?;
        let modern = CoRegisterClassObject(
            &anemorph_shell::CLASS_ID,
            &factory,
            CLSCTX_LOCAL_SERVER,
            REGCLS_MULTIPLEUSE,
        )?;
        let ready = CreateEventW(None, true, true, PCWSTR(wide(&(name + ".Ready")).as_ptr()))?;
        let resident = std::env::args().any(|s| s == "--resident");
        let mut idle = Instant::now();
        'pump: loop {
            let wait = MsgWaitForMultipleObjects(
                Some(&[stop]),
                false,
                if resident { u32::MAX } else { 1000 },
                QS_ALLINPUT,
            );
            if wait == WAIT_OBJECT_0 {
                break;
            }
            if wait != WAIT_TIMEOUT && wait.0 != WAIT_OBJECT_0.0 + 1 {
                break;
            }
            let mut msg = MSG::default();
            while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                if msg.message == WM_QUIT {
                    break 'pump;
                }
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            if !anemorph_shell::server_idle() {
                idle = Instant::now();
            }
            if !resident && idle.elapsed() >= Duration::from_secs(30) {
                break;
            }
        }
        let result = CoRevokeClassObject(cookie);
        let modern_result = CoRevokeClassObject(modern);
        drop(factory);
        CoUninitialize();
        let _ = SetEvent(done);
        let _ = CloseHandle(stop);
        let _ = CloseHandle(done);
        let _ = CloseHandle(ready);
        result.and(modern_result)
    }
}
fn main() {
    if let Err(error) = run() {
        eprintln!("Anemorph menu host: {error}");
        std::process::exit(1);
    }
}
