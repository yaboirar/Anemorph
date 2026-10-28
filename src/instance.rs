//! One window per Windows user/session, including launches with package identity.
//! A message-only window receives bounded activations without idle polling.
use crate::{Context, StartupSelection};
use std::{
    io,
    path::Path,
    sync::{Arc, Mutex, OnceLock, mpsc},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Foundation::{
        CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE, HWND, LPARAM, LRESULT, WPARAM,
    },
    System::{
        DataExchange::COPYDATASTRUCT, LibraryLoader::GetModuleHandleW, Threading::CreateMutexW,
    },
    UI::WindowsAndMessaging::*,
};
const MAX_BYTES: usize = 16 * 1024 * 1024;
const PROTOCOL: usize = 0x414e0001;
static INBOX: OnceLock<Arc<Inbox>> = OnceLock::new();
struct Inbox {
    receiver: Mutex<mpsc::Receiver<Option<StartupSelection>>>,
    context: Mutex<Option<Context>>,
}
struct Listener {
    sender: mpsc::SyncSender<Option<StartupSelection>>,
    inbox: Arc<Inbox>,
}
pub struct Primary {
    mutex: HANDLE,
    window: usize,
    worker: Option<JoinHandle<()>>,
}
impl Drop for Primary {
    fn drop(&mut self) {
        // Our dedicated message thread owns this window; no UI/Explorer thread is stopped.
        unsafe {
            PostMessageW(self.window as HWND, WM_CLOSE, 0, 0);
        }
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
        unsafe {
            CloseHandle(self.mutex);
        }
    }
}
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
fn name(profile: &Path) -> String {
    // Stable across rebuilds; the same user must find the same running endpoint.
    let hash = profile
        .to_string_lossy()
        .to_lowercase()
        .bytes()
        .fold(0xcbf29ce484222325u64, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
        });
    format!("Anemorph.Activation.{hash:016x}")
}
// Windows copies WM_COPYDATA bytes into this process for the duration of the call.
// User data points to Listener on this thread's stack until DestroyWindow returns.
unsafe extern "system" fn window_proc(hwnd: HWND, message: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    unsafe {
        match message {
            WM_NCCREATE => {
                let create = &*(l as *const CREATESTRUCTW);
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as isize);
                // DefWindowProc stores the creation title used to find this endpoint.
                DefWindowProcW(hwnd, message, w, l)
            }
            WM_COPYDATA => {
                let state = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Listener;
                if state.is_null() || l == 0 {
                    return 0;
                }
                let data = &*(l as *const COPYDATASTRUCT);
                if data.dwData != PROTOCOL
                    || data.cbData as usize > MAX_BYTES
                    || data.lpData.is_null()
                {
                    return 0;
                }
                let bytes =
                    std::slice::from_raw_parts(data.lpData.cast::<u8>(), data.cbData as usize);
                let Ok(selection) = serde_json::from_slice::<Option<StartupSelection>>(bytes)
                else {
                    return 0;
                };
                if let Some(StartupSelection::Files(files)) = &selection
                    && (files.is_empty() || files.len() > crate::selection::MAX_FILES)
                {
                    return 0;
                }
                let state = &*state;
                if state.sender.try_send(selection).is_err() {
                    return 0;
                }
                if let Ok(context) = state.inbox.context.lock()
                    && let Some(context) = context.as_ref()
                {
                    context.request_repaint();
                }
                1
            }
            WM_CLOSE => {
                DestroyWindow(hwnd);
                0
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                0
            }
            _ => DefWindowProcW(hwnd, message, w, l),
        }
    }
}
fn listen(profile: &Path) -> io::Result<Option<(Primary, Arc<Inbox>)>> {
    let title = name(profile);
    let mutex_name = wide(&format!("Local\\{title}"));
    // Default object ACL and Local namespace limit this lock to the user/session.
    let mutex = unsafe { CreateMutexW(std::ptr::null(), 0, mutex_name.as_ptr()) };
    if mutex.is_null() {
        return Err(io::Error::last_os_error());
    }
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        unsafe {
            CloseHandle(mutex);
        }
        return Ok(None);
    }
    let (sender, receiver) = mpsc::sync_channel(8);
    let inbox = Arc::new(Inbox {
        receiver: Mutex::new(receiver),
        context: Mutex::new(None),
    });
    let worker_inbox = inbox.clone();
    let (ready, started) = mpsc::sync_channel(1);
    let worker = match thread::Builder::new()
        .name("anemorph-activation".into())
        .spawn(move || {
            let mut state = Listener {
                sender,
                inbox: worker_inbox,
            };
            let class = wide("Anemorph.Activation.Window");
            let title = wide(&title);
            unsafe {
                let module = GetModuleHandleW(std::ptr::null());
                let definition = WNDCLASSW {
                    lpfnWndProc: Some(window_proc),
                    hInstance: module,
                    lpszClassName: class.as_ptr(),
                    ..std::mem::zeroed()
                };
                RegisterClassW(&definition);
                let window = CreateWindowExW(
                    0,
                    class.as_ptr(),
                    title.as_ptr(),
                    0,
                    0,
                    0,
                    0,
                    0,
                    HWND_MESSAGE,
                    std::ptr::null_mut(),
                    module,
                    (&mut state as *mut Listener).cast(),
                );
                if window.is_null() {
                    let _ = ready.send(Err(io::Error::last_os_error()));
                    return;
                }
                let _ = ready.send(Ok(window as usize));
                let mut message: MSG = std::mem::zeroed();
                while GetMessageW(&mut message, std::ptr::null_mut(), 0, 0) > 0 {
                    TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }
        }) {
        Ok(worker) => worker,
        Err(e) => {
            unsafe {
                CloseHandle(mutex);
            }
            return Err(e);
        }
    };
    let window = match started.recv().map_err(io::Error::other).and_then(|r| r) {
        Ok(window) => window,
        Err(e) => {
            let _ = worker.join();
            unsafe {
                CloseHandle(mutex);
            }
            return Err(e);
        }
    };
    Ok(Some((
        Primary {
            mutex,
            window,
            worker: Some(worker),
        },
        inbox,
    )))
}
fn forward(profile: &Path, selection: Option<StartupSelection>) -> io::Result<()> {
    let class = wide("Anemorph.Activation.Window");
    let title = wide(&name(profile));
    let deadline = Instant::now() + Duration::from_secs(3);
    let window = loop {
        let window = unsafe {
            FindWindowExW(
                HWND_MESSAGE,
                std::ptr::null_mut(),
                class.as_ptr(),
                title.as_ptr(),
            )
        };
        if !window.is_null() {
            break window;
        }
        if Instant::now() >= deadline {
            return Err(io::Error::other(
                "The running Anemorph window is not responding.",
            ));
        }
        thread::sleep(Duration::from_millis(40));
    };
    let bytes = serde_json::to_vec(&selection)?;
    if bytes.len() > MAX_BYTES {
        return Err(io::Error::other("Selection is too large."));
    }
    let data = COPYDATASTRUCT {
        dwData: PROTOCOL,
        cbData: bytes.len() as u32,
        lpData: bytes.as_ptr() as _,
    };
    let mut result = 0;
    // Payload stays alive until this bounded synchronous copy completes.
    unsafe {
        let mut pid = 0;
        GetWindowThreadProcessId(window, &mut pid);
        AllowSetForegroundWindow(pid);
        let sent = SendMessageTimeoutW(
            window,
            WM_COPYDATA,
            0,
            (&data as *const COPYDATASTRUCT) as isize,
            SMTO_ABORTIFHUNG | SMTO_BLOCK,
            3000,
            &mut result,
        );
        if sent == 0 || result != 1 {
            return Err(io::Error::other(
                "Anemorph could not accept the selection. Try again after the current operation finishes.",
            ));
        }
    }
    Ok(())
}
pub fn start(_data: &Path, selection: Option<StartupSelection>) -> io::Result<Option<Primary>> {
    // USERPROFILE stays the same with and without sparse-package identity.
    // AppData filesystem paths can be redirected by Windows for packaged launches.
    let profile = std::env::var_os("USERPROFILE")
        .ok_or_else(|| io::Error::other("Windows user profile is unavailable."))?;
    let profile = Path::new(&profile);
    match listen(profile)? {
        Some((primary, inbox)) => {
            INBOX
                .set(inbox)
                .map_err(|_| io::Error::other("App activation already initialized."))?;
            Ok(Some(primary))
        }
        None => {
            // Consume a package-local manifest in the launching process. The existing
            // un-packaged UI cannot reliably read a path redirected by package identity.
            let selection = match selection {
                Some(StartupSelection::Request(path)) => Some(StartupSelection::Files(
                    crate::selection::read_request(&path, &crate::selection::directory())
                        .map_err(io::Error::other)?,
                )),
                other => other,
            };
            forward(profile, selection)?;
            Ok(None)
        }
    }
}
pub fn attach(ctx: &Context) {
    if let Some(inbox) = INBOX.get()
        && let Ok(mut context) = inbox.context.lock()
    {
        *context = Some(ctx.clone());
    }
}
pub fn next() -> Option<Option<StartupSelection>> {
    INBOX.get()?.receiver.lock().ok()?.try_recv().ok()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn second_launch_forwards_arbitrary_filenames_and_lock_recovers() {
        let profile =
            std::env::temp_dir().join(format!("anemorph-instance-{}", std::process::id()));
        let (primary, inbox) = listen(&profile).unwrap().unwrap();
        assert!(listen(&profile).unwrap().is_none());
        let files = vec![std::path::PathBuf::from(
            r"C:\Videos\random capture 日本語.mp4",
        )];
        forward(&profile, Some(StartupSelection::Files(files.clone()))).unwrap();
        let selection = inbox.receiver.lock().unwrap().try_recv().unwrap();
        assert!(matches!(selection, Some(StartupSelection::Files(received)) if received == files));
        forward(&profile, None).unwrap();
        assert!(inbox.receiver.lock().unwrap().try_recv().unwrap().is_none());
        drop(primary);
        let (primary, _) = listen(&profile).unwrap().unwrap();
        drop(primary);
    }
    #[test]
    fn separate_process_hands_off_without_starting_another_primary() {
        let profile =
            std::env::temp_dir().join(format!("anemorph-process-handoff-{}", std::process::id()));
        let (primary, inbox) = listen(&profile).unwrap().unwrap();
        let child = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "instance::tests::activation_child", "--ignored"])
            .env("ANEMORPH_TEST_ACTIVATION_PROFILE", &profile)
            .output()
            .unwrap();
        assert!(
            child.status.success(),
            "{}",
            String::from_utf8_lossy(&child.stdout)
        );
        let received = inbox.receiver.lock().unwrap().try_recv().unwrap();
        assert!(matches!(received, Some(StartupSelection::Files(files))
            if files == vec![std::path::PathBuf::from(r"C:\Videos\2026-10-26 22-10-06.mp4")]));
        drop(primary);
    }
    #[test]
    #[ignore = "only called by the separate-process handoff test"]
    fn activation_child() {
        let profile =
            std::path::PathBuf::from(std::env::var_os("ANEMORPH_TEST_ACTIVATION_PROFILE").unwrap());
        assert!(listen(&profile).unwrap().is_none());
        forward(
            &profile,
            Some(StartupSelection::Files(vec![std::path::PathBuf::from(
                r"C:\Videos\2026-10-26 22-10-06.mp4",
            )])),
        )
        .unwrap();
    }
}
