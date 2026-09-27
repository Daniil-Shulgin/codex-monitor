use crate::model::{display_name, Status, Task};
use serde_json::Value;
use std::collections::HashSet;

#[derive(Default)]
pub struct Lifecycle {
    pub task: Option<Task>,
    pub observed: bool,
    pub source_vscode: bool,
    pub thread_id: String,
    pub title: String,
    pub first_message: String,
    blocking: HashSet<String>,
}

fn string<'a>(v: &'a Value, keys: &[&str]) -> Option<&'a str> {
    keys.iter().find_map(|k| v.get(k).and_then(Value::as_str))
}
fn request_id(v: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|k| match v.get(k) {
        Some(Value::String(s)) => Some(s.clone()),
        Some(Value::Number(n)) => Some(n.to_string()),
        _ => None,
    })
}
fn timestamp(v: &Value) -> Option<i64> {
    if let Some(n) = v.as_i64() {
        Some(if n < 100_000_000_000 {
            n.saturating_mul(1000)
        } else {
            n
        })
    } else {
        v.as_str()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|d| d.timestamp_millis())
    }
}
impl Lifecycle {
    pub fn with_metadata(id: String, title: String) -> Self {
        Self {
            thread_id: id,
            title,
            source_vscode: true,
            ..Self::default()
        }
    }
    pub fn rename(&mut self, title: &str) {
        self.title = display_name(title);
        let name = self.name();
        if let Some(t) = &mut self.task {
            t.display_name = name;
        }
    }
    fn name(&self) -> String {
        if !self.title.is_empty() {
            self.title.clone()
        } else if !self.first_message.is_empty() {
            self.first_message.clone()
        } else {
            "Codex thread".into()
        }
    }
    pub fn consume(&mut self, record: &[u8], live: bool) {
        let Ok(v) = serde_json::from_slice::<Value>(record) else {
            return;
        };
        self.apply(&v, live);
    }
    pub fn apply(&mut self, v: &Value, live: bool) {
        let envelope = string(v, &["type"]).unwrap_or("");
        let p = v.get("payload").or_else(|| v.get("params")).unwrap_or(v);
        let kind = string(v, &["method"])
            .or_else(|| string(p, &["type"]))
            .unwrap_or("");
        let at = v
            .get("timestamp")
            .and_then(timestamp)
            .or_else(|| p.get("timestamp").and_then(timestamp))
            .unwrap_or_else(crate::model::now_ms);
        if envelope == "session_meta" {
            self.source_vscode = string(p, &["source"]) == Some("vscode")
                || string(p, &["originator"]) == Some("codex_vscode");
            if let Some(id) = string(p, &["id"]) {
                self.thread_id = id.into();
            }
            return;
        }
        if !self.source_vscode {
            return;
        }
        if kind == "user_message" && self.first_message.is_empty() {
            if let Some(s) = string(p, &["message"]) {
                self.first_message = display_name(s);
                let name = self.name();
                if let Some(t) = &mut self.task {
                    t.display_name = name;
                }
            }
            return;
        }
        if matches!(kind, "thread/name/updated" | "thread_name_updated") {
            if let Some(s) = string(p, &["threadName", "name", "title"]) {
                self.rename(s);
            }
            return;
        }
        let turn = string(p, &["turn_id", "turnId"])
            .or_else(|| p.pointer("/turn/id").and_then(Value::as_str));
        if matches!(kind, "task_started" | "turn_started" | "turn/started") {
            let started = p.get("started_at").and_then(timestamp).unwrap_or(at);
            let id = turn
                .map(str::to_owned)
                .unwrap_or_else(|| format!("start:{started}"));
            if self.task.as_ref().is_some_and(|t| t.turn_id == id) {
                return;
            }
            if self.task.as_ref().is_some_and(|t| started < t.started_at) {
                return;
            }
            self.blocking.clear();
            self.observed = live;
            self.task = Some(Task {
                thread_id: self.thread_id.clone(),
                turn_id: id,
                display_name: self.name(),
                status: Status::Running,
                started_at: started,
                completed_at: None,
                duration_ms: None,
                updated_at: at,
            });
            return;
        }
        let Some(t) = &mut self.task else { return };
        if turn.is_some_and(|id| id != t.turn_id) || at < t.started_at {
            return;
        }
        if t.completed_at.is_some() {
            return;
        }
        let request = request_id(p, &["request_id", "requestId"])
            .or_else(|| request_id(v, &["id"]))
            .or_else(|| request_id(p, &["call_id", "itemId", "id"]));
        let blocking = matches!(
            kind,
            "exec_approval_request"
                | "apply_patch_approval_request"
                | "request_user_input"
                | "elicitation_request"
                | "permissions_request"
                | "item/commandExecution/requestApproval"
                | "item/fileChange/requestApproval"
                | "item/permissions/requestApproval"
                | "item/tool/requestUserInput"
                | "mcpServer/elicitation/request"
                | "execCommandApproval"
                | "applyPatchApproval"
        );
        let sync_input_call = kind == "function_call"
            && string(p, &["name"])
                .is_some_and(|s| s == "request_user_input" || s.ends_with(".request_user_input"));
        if blocking || sync_input_call {
            if let Some(id) = request {
                if self.blocking.len() < 256 {
                    self.blocking.insert(id);
                }
            } else {
                self.blocking.insert("unidentified".into());
            }
            t.status = Status::NeedsAction;
            t.updated_at = at;
        } else if matches!(
            kind,
            "serverRequest/resolved"
                | "approval_resolved"
                | "request_user_input_response"
                | "elicitation_response"
                | "function_call_output"
                | "custom_tool_call_output"
        ) {
            if let Some(id) = request {
                self.blocking.remove(&id);
            }
            if self.blocking.is_empty() {
                t.status = Status::Running;
                t.updated_at = at;
            }
        } else if matches!(kind, "task_complete" | "turn_complete" | "turn/completed") {
            let status = p
                .pointer("/turn/status")
                .and_then(Value::as_str)
                .or_else(|| string(p, &["status"]));
            let failed = p.get("error").is_some_and(|v| !v.is_null())
                || matches!(status, Some("failed" | "interrupted" | "aborted"));
            t.status = if failed {
                Status::NeedsAction
            } else {
                Status::Done
            };
            t.completed_at = Some(
                p.get("completed_at")
                    .and_then(timestamp)
                    .unwrap_or(at)
                    .max(t.started_at),
            );
            t.updated_at = at;
            t.duration_ms = p.get("duration_ms").and_then(Value::as_u64);
            self.blocking.clear();
        } else if matches!(
            kind,
            "turn_aborted" | "task_aborted" | "turn_failed" | "error"
        ) && p.get("will_retry").and_then(Value::as_bool) != Some(true)
        {
            t.status = Status::NeedsAction;
            t.completed_at = Some(at);
            t.updated_at = at;
            self.blocking.clear();
        }
        if live {
            self.observed = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn engine() -> Lifecycle {
        Lifecycle::with_metadata("thread".into(), "Test".into())
    }
    fn event(e: &mut Lifecycle, kind: &str, turn: &str, at: i64) {
        e.apply(
            &json!({"timestamp":at,"type":"event_msg","payload":{"type":kind,"turn_id":turn}}),
            true,
        );
    }
    #[test]
    fn normal_and_new_turn_and_old_terminal() {
        let mut e = engine();
        event(&mut e, "task_started", "a", 100);
        event(&mut e, "task_complete", "a", 110);
        assert_eq!(e.task.as_ref().unwrap().status, Status::Done);
        event(&mut e, "turn_started", "b", 120);
        event(&mut e, "task_complete", "a", 130);
        assert_eq!(e.task.as_ref().unwrap().status, Status::Running);
        assert_eq!(e.task.as_ref().unwrap().started_at, 120000);
    }
    #[test]
    fn two_blocking_requests_require_both_resolutions() {
        let mut e = engine();
        event(&mut e, "task_started", "a", 100);
        for id in ["1", "2"] {
            e.apply(
                &json!({"timestamp":101,"payload":{"type":"request_user_input","call_id":id}}),
                true,
            );
        }
        e.apply(
            &json!({"timestamp":102,"payload":{"type":"function_call_output","call_id":"1"}}),
            true,
        );
        assert_eq!(e.task.as_ref().unwrap().status, Status::NeedsAction);
        e.apply(
            &json!({"timestamp":103,"payload":{"type":"function_call_output","call_id":"2"}}),
            true,
        );
        assert_eq!(e.task.as_ref().unwrap().status, Status::Running);
        assert_eq!(e.task.as_ref().unwrap().started_at, 100000);
        event(&mut e, "turn_complete", "a", 110);
        assert_eq!(e.task.as_ref().unwrap().status, Status::Done);
    }
    #[test]
    fn failures_and_dangling_are_never_done() {
        for kind in ["turn_aborted", "turn_failed", "error"] {
            let mut e = engine();
            event(&mut e, "task_started", "a", 100);
            event(&mut e, kind, "a", 110);
            assert_eq!(e.task.unwrap().status, Status::NeedsAction);
        }
        let mut e = engine();
        event(&mut e, "task_started", "a", 100);
        assert_eq!(e.task.unwrap().status, Status::Running);
    }
    #[test]
    fn duplicate_start_and_complete_do_not_reset_timers() {
        let mut e = engine();
        event(&mut e, "task_started", "a", 100);
        event(&mut e, "task_started", "a", 110);
        assert_eq!(e.task.as_ref().unwrap().started_at, 100000);
        event(&mut e, "task_complete", "a", 120);
        event(&mut e, "task_complete", "a", 130);
        assert_eq!(e.task.unwrap().completed_at, Some(120000));
    }
    #[test]
    fn names_are_normalized_and_bounded() {
        let mut e = engine();
        event(&mut e, "task_started", "a", 100);
        e.rename("  New  name\n ");
        assert_eq!(e.task.as_ref().unwrap().display_name, "New name");
        e.rename(&"x".repeat(1000));
        assert_eq!(e.task.unwrap().display_name.len(), 160);
    }
    #[test]
    fn parallel_threads_and_input_call() {
        let mut a = engine();
        let mut b = engine();
        event(&mut a, "task_started", "a", 100);
        event(&mut b, "task_started", "b", 100);
        a.apply(&json!({"timestamp":101,"type":"response_item","payload":{"type":"function_call","name":"request_user_input","call_id":"r"}}),true);
        assert_eq!(a.task.unwrap().status, Status::NeedsAction);
        assert_eq!(b.task.unwrap().status, Status::Running);
    }
    #[test]
    fn terminal_error_never_becomes_green_and_server_duration_is_used() {
        let mut e = engine();
        event(&mut e, "task_started", "a", 100);
        e.apply(&json!({"timestamp":120,"payload":{"type":"task_complete","turn_id":"a","error":{"message":"synthetic failure"},"completed_at":119,"duration_ms":19000}}),true);
        let t = e.task.unwrap();
        assert_eq!(t.status, Status::NeedsAction);
        assert_eq!(t.completed_at, Some(119000));
        assert_eq!(t.duration_ms, Some(19000));
    }
    #[test]
    fn explicit_approval_alias_and_request_resolution() {
        let mut e = engine();
        event(&mut e, "turn_started", "a", 100);
        e.apply(&json!({"timestamp":101,"id":42,"method":"item/fileChange/requestApproval","params":{"turnId":"a","itemId":"different-item-id"}}),true);
        assert_eq!(e.task.as_ref().unwrap().status, Status::NeedsAction);
        e.apply(
            &json!({"timestamp":102,"method":"serverRequest/resolved","params":{"requestId":42}}),
            true,
        );
        assert_eq!(e.task.unwrap().status, Status::Running);
    }
    #[test]
    fn fallback_title_is_bounded_and_explicit_rename_wins() {
        let mut e = Lifecycle::with_metadata("thread".into(), String::new());
        event(&mut e, "task_started", "a", 100);
        e.apply(&json!({"timestamp":101,"payload":{"type":"user_message","message":"  Fix\n  tests  "}}),true);
        assert_eq!(e.task.as_ref().unwrap().display_name, "Fix tests");
        e.rename("My title");
        assert_eq!(e.task.unwrap().display_name, "My title");
    }
    #[test]
    fn malformed_records_and_non_ide_sources_do_not_create_tasks() {
        let mut e = engine();
        e.consume(b"not JSON", true);
        assert!(e.task.is_none());
        e.apply(
            &json!({"type":"session_meta","payload":{"source":"cli"}}),
            false,
        );
        event(&mut e, "task_started", "a", 100);
        assert!(e.task.is_none());
    }
}
