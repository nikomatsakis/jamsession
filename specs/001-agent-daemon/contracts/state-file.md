# Contract: State File (`~/.academy/state.json`)

The daemon persists minimal state to survive restarts. The file is read on startup and written atomically on every mutation.

## Format

```json
{
  "version": 1,
  "sessions": [
    {
      "session_id": "sess_abc123def456",
      "cwd": "/home/user/project",
      "created_at": "2026-06-01T10:00:00Z",
      "updated_at": "2026-06-02T14:30:00Z"
    }
  ],
  "capabilities_cache": {
    "client_capabilities_hash": 12345678901234,
    "response": { "...full initialize response..." }
  }
}
```

## Persistence Strategy

- **Write**: Serialize to JSON, write to `~/.academy/state.json.tmp`, `rename()` over `state.json`
- **Read**: On daemon startup, deserialize from `~/.academy/state.json`. Missing file = fresh state.
- **Corruption**: If JSON parse fails on startup, log error, start with empty state.

## Mutation Points

The state file is written after:
1. Session created (`session/new` completes)
2. Session removed (working directory deleted)

## Staleness

The state file may reference sessions that no longer exist in the agent's internal store (e.g., if the user deletes `~/.claude` directly). The daemon does not attempt to reconcile — it will discover the mismatch when it tries `session/load` on the agent and handle the error.
