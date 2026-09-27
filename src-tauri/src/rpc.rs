use crate::{model::Quota, quotas, scanner::Scanner};
use serde_json::{json, Value};
use std::os::windows::process::CommandExt;
use std::{
    io::{Read, Write},
    path::PathBuf,
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc::{self, Receiver},
    time::{Duration, Instant},
};

pub fn discover() -> Option<PathBuf> {
    let user = std::env::var_os("USERPROFILE")?;
    let root = PathBuf::from(user).join(".vscode/extensions");
    let mut dirs = std::fs::read_dir(root)
        .ok()?
        .flatten()
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("openai.chatgpt-")
        })
        .map(|e| e.path())
        .collect::<Vec<_>>();
    dirs.sort();
    for dir in dirs.into_iter().rev() {
        for relative in [
            "bin/windows-x86_64/codex.exe",
            "bin/windows-aarch64/codex.exe",
        ] {
            let p = dir.join(relative);
            if p.is_file() {
                return Some(p);
            }
        }
    }
    std::env::var_os("PATH").and_then(|p| {
        std::env::split_paths(&p)
            .map(|p| p.join("codex.exe"))
            .find(|p| p.is_file())
    })
}
pub struct Rpc {
    child: Child,
    input: ChildStdin,
    rx: Receiver<Value>,
    next: u64,
    _job: crate::process_job::ProcessJob,
}
impl Drop for Rpc {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
impl Rpc {
    pub fn connect() -> Result<Self, String> {
        let exe = discover().ok_or("Codex executable unavailable")?;
        let mut child = Command::new(exe)
            .args(["app-server", "--stdio", "-c", "analytics.enabled=false"])
            .creation_flags(0x08000000)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| "Cannot start local Codex server")?;
        let job = match crate::process_job::ProcessJob::attach(&child) {
            Ok(job) => job,
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("Cannot contain local Codex process".into());
            }
        };
        let input = child.stdin.take().ok_or("Missing Codex stdin")?;
        let mut output = child.stdout.take().ok_or("Missing Codex stdout")?;
        let (tx, rx) = mpsc::sync_channel(16);
        std::thread::spawn(move || {
            let mut scanner = Scanner::default();
            let mut bytes = [0; 16384];
            while let Ok(n) = output.read(&mut bytes) {
                if n == 0 {
                    break;
                }
                scanner.feed(&bytes[..n], |line| {
                    if let Ok(v) = serde_json::from_slice(line) {
                        let _ = tx.try_send(v);
                    }
                });
            }
        });
        let mut rpc = Self {
            child,
            input,
            rx,
            next: 1,
            _job: job,
        };
        rpc.request("initialize",Some(json!({"clientInfo":{"name":"codex_monitor","version":"0.1.0"},"capabilities":{"experimentalApi":true}})))?;
        rpc.send(&json!({"method":"initialized"}))?;
        Ok(rpc)
    }
    fn send(&mut self, v: &Value) -> Result<(), String> {
        serde_json::to_writer(&mut self.input, v).map_err(|_| "Codex write failed")?;
        self.input
            .write_all(b"\n")
            .and_then(|_| self.input.flush())
            .map_err(|_| "Codex disconnected".into())
    }
    pub fn request(&mut self, method: &str, params: Option<Value>) -> Result<Value, String> {
        let id = self.next;
        self.next += 1;
        let mut request = json!({"id":id,"method":method});
        if let Some(p) = params {
            request["params"] = p;
        }
        self.send(&request)?;
        let deadline = Instant::now() + Duration::from_secs(12);
        loop {
            let wait = deadline.saturating_duration_since(Instant::now());
            if wait.is_zero() {
                return Err("Codex request timed out".into());
            }
            let v = self
                .rx
                .recv_timeout(wait)
                .map_err(|_| "Codex unavailable or request timed out")?;
            if v.get("id").and_then(Value::as_u64) == Some(id) {
                if v.get("error").is_some() {
                    return Err("Codex rejected read request".into());
                }
                return Ok(v["result"].clone());
            }
            // This read-only client never answers approval or credential-refresh requests.
        }
    }
    pub fn quotas(&mut self) -> Result<(Quota, Quota), String> {
        self.request("account/rateLimits/read", None)
            .map(|v| quotas::parse(&v))
    }
}
