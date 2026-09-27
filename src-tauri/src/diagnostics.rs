use std::{io::Write, path::Path, sync::Mutex};
static LOG_LOCK: Mutex<()> = Mutex::new(());
/// Callers supply static categories only, never raw errors or source content.
pub fn log(dir: &Path, category: &str) {
    let Ok(_guard) = LOG_LOCK.lock() else { return };
    let _ = std::fs::create_dir_all(dir);
    let path = dir.join("monitor.log");
    if std::fs::metadata(&path).is_ok_and(|m| m.len() > 256 * 1024) {
        let _ = std::fs::rename(&path, dir.join("monitor.previous.log"));
    }
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        let _ = writeln!(
            file,
            "{{\"at\":{},\"event\":\"{}\"}}",
            crate::model::now_ms(),
            category
        );
    }
}
