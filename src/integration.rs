use eframe::egui;
use std::{os::windows::process::CommandExt, path::PathBuf, process::Command, sync::mpsc};
#[derive(Default)]
pub struct Integration {
    pub enabled: bool,
    pub message: String,
    pub pending: bool,
    checked: bool,
    receiver: Option<mpsc::Receiver<Result<bool, String>>>,
}
fn command(root: &std::path::Path, action: &str) -> Result<std::process::Output, String> {
    Command::new(root.join("anemorph-maintenance.exe"))
        .arg(action)
        .creation_flags(0x08000000)
        .output()
        .map_err(|e| format!("Run setup to repair Explorer integration: {e}"))
}
impl Integration {
    pub fn start(&mut self, root: PathBuf, enable: Option<bool>) {
        if self.pending {
            return;
        }
        self.pending = true;
        self.checked = true;
        self.message.clear();
        let (send, receive) = mpsc::channel();
        self.receiver = Some(receive);
        std::thread::spawn(move || {
            let result = (|| {
                match enable {
                    Some(true) => {
                        let register = || {
                            Command::new(root.join("anemorph-maintenance.exe"))
                                .arg("register-modern")
                                .arg(&root)
                                .creation_flags(0x08000000)
                                .output()
                                .map_err(|e| e.to_string())
                        };
                        let first = register()?;
                        if !first.status.success() {
                            let text = String::from_utf8_lossy(&first.stderr);
                            if !text.contains("0x800B0109") && !text.contains("0x800b0109") {
                                return Err(text.trim().to_owned());
                            }
                            if !command(&root, "trust-ready")?.status.success() {
                                return Err("This build needs a trusted signing certificate. Install the latest setup.".into());
                            }
                            crate::elevation::run(
                                &root.join("anemorph-maintenance.exe"),
                                "trust-modern",
                            )?;
                            let retry = register()?;
                            if !retry.status.success() {
                                return Err(String::from_utf8_lossy(&retry.stderr)
                                    .trim()
                                    .to_owned());
                            }
                        }
                    }
                    Some(false) => {
                        let out = command(&root, "unregister-modern")?;
                        if !out.status.success() {
                            return Err(String::from_utf8_lossy(&out.stderr).trim().to_owned());
                        }
                    }
                    None => {}
                }
                let out = command(&root, "modern-status")?;
                match out.status.code() {
                    Some(0) => Ok(true),
                    Some(10) => Ok(false),
                    _ => Err(String::from_utf8_lossy(&out.stderr).trim().to_owned()),
                }
            })();
            let _ = send.send(result);
        });
    }
    pub fn poll(&mut self, root: Option<PathBuf>, ctx: &egui::Context) {
        if !self.checked
            && let Some(root) = root
        {
            self.start(root, None);
        }
        if let Some(receive) = &self.receiver {
            match receive.try_recv() {
                Ok(result) => {
                    self.pending = false;
                    self.receiver = None;
                    match result {
                        Ok(enabled) => {
                            self.enabled = enabled;
                            self.message = if enabled {
                                "Available on the first right-click menu."
                            } else {
                                "Available under Show more options."
                            }
                            .into();
                        }
                        Err(error) => self.message = error,
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.pending = false;
                    self.receiver = None;
                    self.message =
                        "Integration check interrupted. Reopen Anemorph to retry.".into();
                }
                Err(mpsc::TryRecvError::Empty) => {
                    ctx.request_repaint_after(std::time::Duration::from_millis(100))
                }
            }
        }
    }
}
