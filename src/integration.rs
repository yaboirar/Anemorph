use eframe::egui;
use std::{
    os::windows::process::CommandExt,
    path::PathBuf,
    process::{Command, Output, Stdio},
    sync::mpsc,
    time::{Duration, Instant},
};
#[derive(Default)]
pub struct Integration {
    pub enabled: bool,
    pub message: String,
    pub pending: bool,
    checked: bool,
    receiver: Option<mpsc::Receiver<Result<bool, String>>>,
}
fn command(root: &std::path::Path, action: &str) -> Result<std::process::Output, String> {
    run_command(
        Command::new(root.join("anemorph-maintenance.exe")).arg(action),
        Duration::from_secs(10),
    )
}
fn root_command(root: &std::path::Path, action: &str) -> Result<Output, String> {
    run_command(
        Command::new(root.join("anemorph-maintenance.exe"))
            .arg(action)
            .arg(root),
        Duration::from_secs(60),
    )
}
fn run_command(command: &mut Command, timeout: Duration) -> Result<Output, String> {
    let mut child = command
        .creation_flags(0x08000000)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Run setup to repair Explorer integration: {e}"))?;
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return child.wait_with_output().map_err(|e| e.to_string()),
            Ok(None) => {}
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(error.to_string());
            }
        }
        if started.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Err("Windows integration did not respond in time. Try again.".into());
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}
impl Integration {
    pub fn start(&mut self, root: PathBuf, enable: Option<bool>) {
        self.start_action(root, enable, false);
    }
    pub fn repair(&mut self, root: PathBuf) {
        self.start_action(root, None, true);
    }
    fn start_action(&mut self, root: PathBuf, enable: Option<bool>, repair: bool) {
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
                if repair {
                    let out = root_command(&root, "sync-menu")?;
                    if !out.status.success() {
                        return Err(String::from_utf8_lossy(&out.stderr).trim().to_owned());
                    }
                }
                match enable {
                    Some(true) => {
                        let register = || root_command(&root, "register-modern");
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
                        let out = root_command(&root, "classic-menu")?;
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
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn helper_waits_only_in_child() {
        if std::env::var_os("ANEMORPH_TIMEOUT_TEST_CHILD").is_some() {
            std::thread::sleep(Duration::from_secs(30));
        }
    }
    #[test]
    fn an_unresponsive_check_returns_an_error_instead_of_waiting_forever() {
        let started = Instant::now();
        let result = run_command(
            Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "integration::tests::helper_waits_only_in_child"])
                .env("ANEMORPH_TIMEOUT_TEST_CHILD", "1"),
            Duration::from_millis(80),
        );
        assert!(result.unwrap_err().contains("did not respond"));
        assert!(started.elapsed() < Duration::from_secs(3));
    }
    #[test]
    fn failed_checks_leave_the_button_ready_to_retry() {
        let (send, receive) = mpsc::channel();
        send.send(Err("timed out".into())).unwrap();
        let mut integration = Integration {
            pending: true,
            checked: true,
            receiver: Some(receive),
            ..Default::default()
        };
        integration.poll(None, &egui::Context::default());
        assert!(!integration.pending);
        assert_eq!(integration.message, "timed out");
    }
}
