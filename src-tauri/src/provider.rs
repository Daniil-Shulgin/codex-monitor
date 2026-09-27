use crate::{
    lifecycle::Lifecycle,
    metadata,
    model::{Snapshot, Status, Task},
    scanner::Scanner,
};
use fs2::FileExt;
use notify::Watcher;
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Mutex,
    },
    time::{Duration, Instant},
};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};
use tauri::{AppHandle, Emitter};

pub trait CodexDataProvider {
    fn reconcile(&mut self) -> (Vec<crate::model::Task>, bool, String);
}
struct Entry {
    scanner: Scanner,
    engine: Lifecycle,
    path: PathBuf,
    initial: bool,
    owned: bool,
    hidden_turn: Option<(String, i64)>,
}
const STATUS_IDLE_LIMIT_MS: i64 = 60 * 60 * 1000;
fn show_task(task: &Task, hidden_turn: &mut Option<(String, i64)>, now: i64) -> bool {
    if hidden_turn
        .as_ref()
        .is_some_and(|(id, started)| id == &task.turn_id && *started == task.started_at)
    {
        return false;
    }
    if now.saturating_sub(task.updated_at) >= STATUS_IDLE_LIMIT_MS {
        *hidden_turn = Some((task.turn_id.clone(), task.started_at));
        return false;
    }
    *hidden_turn = None;
    true
}
pub struct LocalVscodeCodexProvider {
    home: PathBuf,
    entries: HashMap<String, Entry>,
    started: i64,
    system: System,
    ide: bool,
    last_metadata: Instant,
    notifications: Vec<crate::model::Task>,
}
impl LocalVscodeCodexProvider {
    pub fn new(home: PathBuf) -> Self {
        Self {
            home,
            entries: HashMap::new(),
            started: crate::model::now_ms(),
            system: System::new(),
            ide: false,
            last_metadata: Instant::now() - Duration::from_secs(60),
            notifications: Vec::new(),
        }
    }
    fn ownership(&self, id: &str) -> bool {
        let path = self
            .home
            .join("thread-writer-locks")
            .join(format!("{id}.lock"));
        if let Ok(file) = File::open(path) {
            match FileExt::try_lock_shared(&file) {
                Ok(()) => {
                    let _ = FileExt::unlock(&file);
                    false
                }
                Err(e) => {
                    e.kind() == std::io::ErrorKind::WouldBlock || e.raw_os_error() == Some(33)
                }
            }
        } else {
            false
        }
    }
}
impl CodexDataProvider for LocalVscodeCodexProvider {
    fn reconcile(&mut self) -> (Vec<crate::model::Task>, bool, String) {
        let mut health = "Approval visibility is limited by the VS Code protocol.".to_string();
        if self.last_metadata.elapsed() >= Duration::from_secs(5) {
            self.last_metadata = Instant::now();
            self.system.refresh_processes_specifics(
                ProcessesToUpdate::All,
                true,
                ProcessRefreshKind::nothing(),
            );
            self.ide = self.system.processes().values().any(|p| {
                p.name().to_string_lossy().eq_ignore_ascii_case("codex.exe")
                    && p.parent()
                        .and_then(|id| self.system.process(id))
                        .is_some_and(|p| {
                            p.name().to_string_lossy().eq_ignore_ascii_case("Code.exe")
                        })
            });
            match metadata::read(&self.home) {
                Ok(rows) => {
                    let ids: HashSet<_> = rows.iter().map(|r| r.id.clone()).collect();
                    self.entries.retain(|id, _| ids.contains(id));
                    for row in rows {
                        let owned = self.ide && self.ownership(&row.id);
                        let modified = std::fs::metadata(&row.path)
                            .ok()
                            .and_then(|m| m.modified().ok())
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| d.as_millis() as i64)
                            .unwrap_or(0);
                        if let Some(e) = self.entries.get_mut(&row.id) {
                            e.engine.rename(&row.name);
                            e.owned = owned;
                        } else if owned || modified >= self.started {
                            self.entries.insert(
                                row.id.clone(),
                                Entry {
                                    scanner: Scanner::default(),
                                    engine: Lifecycle::with_metadata(row.id, row.name),
                                    path: row.path,
                                    initial: true,
                                    owned,
                                    hidden_turn: None,
                                },
                            );
                        }
                    }
                }
                Err(e) => health = e,
            }
        }
        let mut tasks = Vec::new();
        let now = crate::model::now_ms();
        let mut budget = 16 * 1024 * 1024u64;
        for entry in self.entries.values_mut() {
            if budget > 0 {
                if let Ok(mut file) = File::open(&entry.path) {
                    if file
                        .metadata()
                        .is_ok_and(|m| m.len() < entry.scanner.offset)
                    {
                        entry.scanner = Scanner::default();
                        entry.engine = Lifecycle::with_metadata(
                            entry.engine.thread_id.clone(),
                            entry.engine.title.clone(),
                        );
                        entry.initial = true;
                        entry.hidden_turn = None;
                    }
                    let before = entry.scanner.offset;
                    let live = !entry.initial;
                    let _ = entry
                        .scanner
                        .tail(&mut file, budget.min(4 * 1024 * 1024), |line| {
                            let before = entry
                                .engine
                                .task
                                .as_ref()
                                .map(|t| (t.turn_id.clone(), t.status));
                            entry.engine.consume(line, live);
                            if live {
                                if let Some(t) = &entry.engine.task {
                                    if t.status != Status::Running
                                        && before.as_ref().is_some_and(|(id, status)| {
                                            *id == t.turn_id && *status != t.status
                                        })
                                        && self.notifications.len() < 64
                                    {
                                        self.notifications.push(t.clone());
                                    }
                                }
                            }
                        });
                    budget = budget.saturating_sub(entry.scanner.offset - before);
                    let caught_up = file
                        .metadata()
                        .is_ok_and(|m| entry.scanner.offset >= m.len());
                    if entry.initial && caught_up {
                        if let Some(t) = &entry.engine.task {
                            entry.engine.observed = t.started_at >= self.started
                                || (entry.owned && t.completed_at.is_none());
                        }
                        entry.initial = false;
                    }
                }
            }
            if !self.ide {
                entry.engine.observed = false;
                continue;
            }
            if entry.initial || !entry.engine.observed {
                continue;
            }
            if let Some(task) = &entry.engine.task {
                if !show_task(task, &mut entry.hidden_turn, now) {
                    continue;
                }
                if task.completed_at.is_none() && !entry.owned {
                    // A released owner is not success. Preserve observed work as requiring intervention.
                    let mut task = task.clone();
                    task.status = Status::NeedsAction;
                    tasks.push(task);
                } else {
                    tasks.push(task.clone());
                }
            }
        }
        tasks.sort_by(|a, b| {
            let rank = |s| match s {
                Status::NeedsAction => 0,
                Status::Running => 1,
                Status::Done => 2,
            };
            rank(a.status)
                .cmp(&rank(b.status))
                .then(b.updated_at.cmp(&a.updated_at))
                .then(a.thread_id.cmp(&b.thread_id))
        });
        (tasks, self.ide, health)
    }
}

pub fn start(app: AppHandle, shared: Arc<Mutex<crate::runtime::Shared>>, home: PathBuf) {
    let app_quota = app.clone();
    let quota_shared = shared.clone();
    std::thread::spawn(move || {
        let mut rpc = None;
        let mut next = Instant::now();
        let mut last_reset = None;
        loop {
            std::thread::sleep(Duration::from_millis(500));
            let (settings, expired) = if let Ok(s) = quota_shared.lock() {
                let now = chrono::Utc::now().timestamp();
                (
                    s.settings.clone(),
                    [s.snapshot.five_hour.resets_at, s.snapshot.weekly.resets_at]
                        .into_iter()
                        .flatten()
                        .filter(|t| *t <= now)
                        .max(),
                )
            } else {
                continue;
            };
            if Instant::now() < next && (expired.is_none() || expired == last_reset) {
                continue;
            }
            last_reset = expired;
            let result = match rpc.as_mut() {
                Some(r) => crate::rpc::Rpc::quotas(r),
                None => match crate::rpc::Rpc::connect() {
                    Ok(mut r) => {
                        let result = r.quotas();
                        rpc = Some(r);
                        result
                    }
                    Err(e) => Err(e),
                },
            };
            if let Ok(mut s) = quota_shared.lock() {
                match result {
                    Ok((f, w)) => {
                        s.snapshot.five_hour = f;
                        s.snapshot.weekly = w;
                        s.snapshot.quota_connected = true;
                        s.snapshot.quota_updated_at = Some(crate::model::now_ms());
                    }
                    Err(_) => {
                        rpc = None;
                        s.snapshot.quota_connected = false;
                        crate::diagnostics::log(&s.dir, "quota_read_failed");
                    }
                }
                let _ = app_quota.emit("snapshot", &s.snapshot);
            }
            next = Instant::now() + Duration::from_secs(settings.refresh_seconds);
        }
    });
    std::thread::spawn(move || {
        let (tx, rx) = mpsc::sync_channel(1);
        let discovery = Arc::new(AtomicBool::new(false));
        let discover_event = discovery.clone();
        let mut watcher =
            notify::recommended_watcher(move |event: Result<notify::Event, notify::Error>| {
                if event.is_ok_and(|e| matches!(e.kind, notify::EventKind::Create(_))) {
                    discover_event.store(true, Ordering::Relaxed);
                }
                let _ = tx.try_send(());
            })
            .ok();
        if let Some(w) = watcher.as_mut() {
            let _ = w.watch(&home.join("sessions"), notify::RecursiveMode::Recursive);
        }
        let mut provider = LocalVscodeCodexProvider::new(home);
        let mut previous = Snapshot::default();
        let mut first = true;
        loop {
            if discovery.swap(false, Ordering::Relaxed) {
                provider.last_metadata = Instant::now() - Duration::from_secs(60);
            }
            let (tasks, ide, diagnostic) = provider.reconcile();
            let mut notifications = std::mem::take(&mut provider.notifications);
            if let Ok(mut s) = shared.lock() {
                if !first {
                    for task in crate::alerts::transitions(&previous.tasks, &tasks) {
                        if !notifications.iter().any(|n| {
                            n.thread_id == task.thread_id
                                && n.turn_id == task.turn_id
                                && n.status == task.status
                        }) {
                            notifications.push(task.clone());
                        }
                    }
                    for task in &notifications {
                        crate::alerts::send(task, &s.settings);
                    }
                }
                s.snapshot.tasks = tasks;
                s.snapshot.ide_connected = ide;
                s.snapshot.diagnostic = diagnostic;
                if s.snapshot != previous {
                    let _ = app.emit("snapshot", &s.snapshot);
                    crate::diagnostics::log(&s.dir, "snapshot_changed");
                }
                previous = s.snapshot.clone();
            }
            first = false;
            let _ = rx.recv_timeout(Duration::from_millis(500));
            // Coalesce file-system bursts; never rescan complete history or busy-loop on writes.
            std::thread::sleep(Duration::from_millis(100));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_turn_stays_hidden_until_a_new_turn_starts() {
        let mut task = Task {
            thread_id: "thread".into(),
            turn_id: "first".into(),
            display_name: "Task".into(),
            status: Status::Running,
            started_at: 1_000,
            completed_at: None,
            duration_ms: None,
            updated_at: 1_000,
        };
        let mut hidden = None;
        assert!(show_task(
            &task,
            &mut hidden,
            1_000 + STATUS_IDLE_LIMIT_MS - 1
        ));
        assert!(!show_task(&task, &mut hidden, 1_000 + STATUS_IDLE_LIMIT_MS));
        task.status = Status::Done;
        task.updated_at += STATUS_IDLE_LIMIT_MS + 1;
        assert!(!show_task(&task, &mut hidden, task.updated_at));
        task.turn_id = "second".into();
        task.started_at = task.updated_at + 1;
        task.updated_at = task.started_at;
        assert!(show_task(&task, &mut hidden, task.updated_at));
        assert!(hidden.is_none());
    }

    #[test]
    fn a_status_change_before_the_hour_resets_the_idle_period() {
        let task = Task {
            thread_id: "thread".into(),
            turn_id: "turn".into(),
            display_name: "Task".into(),
            status: Status::NeedsAction,
            started_at: 1_000,
            completed_at: None,
            duration_ms: None,
            updated_at: 1_000 + STATUS_IDLE_LIMIT_MS / 2,
        };
        let mut hidden = None;
        assert!(show_task(&task, &mut hidden, 1_000 + STATUS_IDLE_LIMIT_MS));
    }
}
