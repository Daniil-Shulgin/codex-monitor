# Architecture decisions

- Rust owns source discovery, parsing, task lifecycle, persistence, and native integration.
- Renderer consumes normalized snapshots only. No demo fallback in production.
- Use authenticated local app-server only for quota; never access private HTTP endpoints or credentials.
- Runtime activity in a separate app-server must be proven, not assumed.
- Keep rollout scanner memory bounded; skip oversized lines without allocating their contents.
- Only synthetic fixtures may be committed.
