# Codex data sources

Tested on Windows, 2026-09-26: `codex-cli 0.155.0-alpha.16.3`, VS Code extension `openai.chatgpt 26.917.62051`.

| Metric | Verified source / behavior |
|---|---|
| Quotas | Spawn extension's executable with `app-server --stdio`, initialize, call `account/rateLimits/read`. Both 300-minute and 10080-minute windows returned on this account. Classify by duration, convert used to remaining. Prefer `rateLimitsByLimitId.codex`. |
| Cross-process runtime | Probe: `thread/loaded/list` returned empty; `thread/list` reported the active VS Code thread as `notLoaded`. A new server cannot observe the IDE server's live approval stream. No thread is resumed or modified. |
| Identity / names | SQLite `state_*.sqlite`, read-only connection. Filter `source='vscode'`, unarchived. Prefer `name`, then `title`, then bounded first-message preview. Names reconciled every five seconds. Only 160 normalized characters are retained in memory. |
| Session ownership | VS Code parent process of Codex, plus an existing held `thread-writer-locks/<id>.lock` lock. A shared nonblocking lock probe never creates or writes the file. No stale historical Done rows on startup. |
| Lifecycle | Shared rollout `event_msg`: `task_started`/`turn_started`, `task_complete`/`turn_complete`, `turn_aborted`. Tolerate protocol aliases `turn/started`, `turn/completed`. Turn IDs protect against late completion. Explicit errors/interruption map to Needs action. Never infer success from inactivity. |
| Blocking requests | Adapter recognizes exec/edit/permission approval, synchronous user input, elicitation, request IDs and their resolutions. Durable synchronous `request_user_input` function-call/output pairs can be recognized when present. Async input requests are deliberately not treated as blocking. **The current IDE server does not broadcast its pending approvals to independent clients. Its normal rollout excludes these transient approval events. Therefore reliable universal approval detection is unavailable in this installed version.** |

SQLite schema and the extension's existing log categories were inspected without exposing conversation text. Neither provides a supported complete pending-request/resolution feed. Log text heuristics are deliberately not used to guess approvals.

Parsing uses offsets, 16 KiB reads, a 64 KiB projected record cap, and a 1 KiB per-string cap. Giant strings are drained while preserving JSON boundaries, so a large `last_agent_message` does not hide a terminal event. Oversized arrays/objects are skipped. Partial trailing records wait for newline. No complete rollout is loaded into RAM. No conversation content is persisted by the monitor.

Fallbacks: missing quota is `—`; stale quota is marked and tooltip identifies it as last known; missing local metadata produces a diagnostic instead of synthetic tasks. Unsupported format changes degrade visibly. This implementation reads at most the 256 most recently updated unarchived IDE threads per metadata reconciliation.

References: [official app-server documentation](https://learn.chatgpt.com/docs/app-server); [upstream rollout persistence policy](https://github.com/openai/codex/blob/main/codex-rs/rollout/src/policy.rs). Upstream policy corroborates why approvals/elicitation are transient; the live cross-process probe documented above was version-specific evidence.
