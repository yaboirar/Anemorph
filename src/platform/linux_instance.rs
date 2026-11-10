//! Per-user Linux activation through a private Unix socket; no idle polling.
use crate::{Context, StartupSelection};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    os::{
        fd::AsRawFd,
        unix::{
            fs::{DirBuilderExt, FileTypeExt, MetadataExt, OpenOptionsExt},
            net::{UnixListener, UnixStream},
        },
    },
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
const MAX_BYTES: usize = 16 * 1024 * 1024;
static INBOX: OnceLock<Arc<Inbox>> = OnceLock::new();
struct Inbox {
    receiver: Mutex<mpsc::Receiver<Option<StartupSelection>>>,
    context: Mutex<Option<Context>>,
}
pub struct Primary {
    _lock: File,
    socket: PathBuf,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}
impl Drop for Primary {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        let _ = UnixStream::connect(&self.socket);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
        let _ = fs::remove_file(&self.socket);
        // A concurrently forked helper may briefly inherit the file description
        // before exec closes it. Unlock explicitly so shutdown never leaves a
        // new launch waiting for that unrelated helper to finish.
        unsafe {
            libc::flock(self._lock.as_raw_fd(), libc::LOCK_UN);
        }
    }
}
fn private_directory(path: &Path) -> io::Result<()> {
    match fs::DirBuilder::new().mode(0o700).create(path) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
        Err(e) => return Err(e),
    }
    let metadata = fs::symlink_metadata(path)?;
    // Only this Unix user may discover or write activation requests.
    if !metadata.is_dir()
        || metadata.file_type().is_symlink()
        || metadata.uid() != unsafe { libc::geteuid() }
        || metadata.mode() & 0o077 != 0
    {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Unsafe app runtime directory.",
        ));
    }
    Ok(())
}
fn listen(dir: &Path) -> io::Result<Option<(Primary, Arc<Inbox>)>> {
    private_directory(dir)?;
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(dir.join("instance.lock"))?;
    // The kernel releases flock after a crash; a stale socket cannot strand the app.
    if unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
        let error = io::Error::last_os_error();
        return if error.kind() == io::ErrorKind::WouldBlock {
            Ok(None)
        } else {
            Err(error)
        };
    }
    let socket = dir.join("instance.sock");
    if let Ok(metadata) = fs::symlink_metadata(&socket) {
        if !metadata.file_type().is_socket() || metadata.uid() != unsafe { libc::geteuid() } {
            return Err(io::Error::other("Unexpected app activation file."));
        }
        fs::remove_file(&socket)?;
    }
    let listener = UnixListener::bind(&socket)?;
    let (sender, receiver) = mpsc::sync_channel(8);
    let inbox = Arc::new(Inbox {
        receiver: Mutex::new(receiver),
        context: Mutex::new(None),
    });
    let worker_inbox = inbox.clone();
    let stop = Arc::new(AtomicBool::new(false));
    let worker_stop = stop.clone();
    let worker = thread::Builder::new()
        .name("anemorph-activation".into())
        .spawn(move || {
            for stream in listener.incoming() {
                if worker_stop.load(Ordering::Acquire) {
                    break;
                }
                let Ok(mut stream) = stream else { continue };
                let _ = stream.set_read_timeout(Some(Duration::from_secs(1)));
                let _ = stream.set_write_timeout(Some(Duration::from_secs(1)));
                let request = (|| -> io::Result<Option<StartupSelection>> {
                    let mut length = [0u8; 4];
                    stream.read_exact(&mut length)?;
                    let length = u32::from_le_bytes(length) as usize;
                    if length > MAX_BYTES {
                        return Err(io::Error::other("Selection is too large."));
                    }
                    let mut bytes = vec![0; length];
                    stream.read_exact(&mut bytes)?;
                    Ok(serde_json::from_slice(&bytes)?)
                })();
                if let Ok(request) = request
                    && sender.try_send(request).is_ok()
                {
                    if let Ok(context) = worker_inbox.context.lock()
                        && let Some(context) = context.as_ref()
                    {
                        context.request_repaint();
                    }
                    let _ = stream.write_all(b"OK");
                }
            }
        })?;
    Ok(Some((
        Primary {
            _lock: lock,
            socket,
            stop,
            worker: Some(worker),
        },
        inbox,
    )))
}
fn forward(dir: &Path, selection: Option<StartupSelection>) -> io::Result<()> {
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut stream = loop {
        match UnixStream::connect(dir.join("instance.sock")) {
            Ok(stream) => break stream,
            Err(e) if Instant::now() >= deadline => return Err(e),
            Err(_) => thread::sleep(Duration::from_millis(30)),
        }
    };
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;
    let bytes = serde_json::to_vec(&selection)?;
    if bytes.len() > MAX_BYTES {
        return Err(io::Error::other("Selection is too large."));
    }
    stream.write_all(&(bytes.len() as u32).to_le_bytes())?;
    stream.write_all(&bytes)?;
    let mut reply = [0; 2];
    stream.read_exact(&mut reply)?;
    if reply != *b"OK" {
        return Err(io::Error::other("Activation was not accepted."));
    }
    Ok(())
}
pub fn start(_data: &Path, selection: Option<StartupSelection>) -> io::Result<Option<Primary>> {
    let base = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute() && p.is_dir())
        .unwrap_or_else(std::env::temp_dir);
    let dir = base.join(format!("anemorph-{}", unsafe { libc::geteuid() }));
    match listen(&dir)? {
        Some((primary, inbox)) => {
            INBOX
                .set(inbox)
                .map_err(|_| io::Error::other("Activation already initialized."))?;
            Ok(Some(primary))
        }
        None => {
            forward(&dir, selection)?;
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
    fn selection_handoff_reuses_primary_and_recovers_stale_socket() {
        let dir = std::env::temp_dir().join(format!("anemorph-ipc-test-{}", std::process::id()));
        let (primary, inbox) = listen(&dir).unwrap().unwrap();
        assert!(listen(&dir).unwrap().is_none());
        let files = vec![PathBuf::from("/tmp/日本語 recording.mp4")];
        forward(&dir, Some(StartupSelection::Files(files.clone()))).unwrap();
        assert!(
            matches!(inbox.receiver.lock().unwrap().try_recv().unwrap(),Some(StartupSelection::Files(received)) if received==files)
        );
        let inherited_lock = primary._lock.try_clone().unwrap();
        drop(primary);
        // Simulate a socket inode left by a terminated process.
        drop(UnixListener::bind(dir.join("instance.sock")).unwrap());
        let (primary, _) = listen(&dir).unwrap().unwrap();
        drop(inherited_lock);
        drop(primary);
        fs::remove_dir_all(dir).unwrap();
    }
}
