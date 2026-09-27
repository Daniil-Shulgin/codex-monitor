use crate::model::display_name;
use rusqlite::{Connection, OpenFlags};
use std::path::{Path, PathBuf};

pub struct Metadata {
    pub id: String,
    pub path: PathBuf,
    pub name: String,
}
pub fn read(home: &Path) -> Result<Vec<Metadata>, String> {
    let mut databases = std::fs::read_dir(home)
        .map_err(|_| "Codex home unavailable")?
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|s| s.starts_with("state_") && s.ends_with(".sqlite"))
        })
        .collect::<Vec<_>>();
    databases.sort_by_key(|p| {
        p.file_stem()
            .and_then(|s| s.to_str())
            .and_then(|s| s.strip_prefix("state_"))
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0)
    });
    let path = databases.last().ok_or("Codex state database unavailable")?;
    let db = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|_| "Codex metadata locked")?;
    db.busy_timeout(std::time::Duration::from_millis(100))
        .map_err(|_| "Metadata timeout unavailable")?;
    let columns = db
        .prepare("PRAGMA table_info(threads)")
        .and_then(|mut s| {
            s.query_map([], |r| r.get::<_, String>(1))?
                .collect::<Result<Vec<_>, _>>()
        })
        .map_err(|_| "Unknown metadata schema")?;
    let name = if columns.iter().any(|c| c == "name") {
        "NULLIF(trim(name),''),"
    } else {
        ""
    };
    // substr in SQLite bounds data before it enters this process; no conversation is copied.
    let sql=format!("SELECT id, rollout_path, substr(COALESCE({name}NULLIF(trim(title),''),NULLIF(trim(first_user_message),''),'Codex thread'),1,160) FROM threads WHERE source='vscode' AND archived=0 ORDER BY updated_at DESC LIMIT 256");
    let mut statement = db
        .prepare(&sql)
        .map_err(|_| "Unsupported metadata schema")?;
    let rows = statement
        .query_map([], |r| {
            Ok(Metadata {
                id: r.get(0)?,
                path: PathBuf::from(r.get::<_, String>(1)?),
                name: display_name(&r.get::<_, String>(2)?),
            })
        })
        .map_err(|_| "Metadata read failed")?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|_| "Metadata decode failed".into())
}
