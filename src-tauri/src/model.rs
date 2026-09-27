use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Status {
    Running,
    NeedsAction,
    Done,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub thread_id: String,
    pub turn_id: String,
    pub display_name: String,
    pub status: Status,
    pub started_at: i64,
    pub completed_at: Option<i64>,
    pub duration_ms: Option<u64>,
    pub updated_at: i64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Quota {
    pub remaining_percent: Option<f64>,
    pub resets_at: Option<i64>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub five_hour: Quota,
    pub weekly: Quota,
    pub tasks: Vec<Task>,
    pub quota_connected: bool,
    pub ide_connected: bool,
    pub diagnostic: String,
    pub quota_updated_at: Option<i64>,
}

pub fn display_name(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .filter(|c| !c.is_control())
        .take(160)
        .collect()
}

pub fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}
