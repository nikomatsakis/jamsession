# Design and implementation

This section documents the internal architecture of Jamsession for contributors and anyone curious about how the daemon works.

## Architecture overview

Jamsession is structured as a single-process daemon with several cooperating components:

```
┌─────────────────────────────────────────────────────────┐
│                        Daemon                            │
│                                                         │
│  ┌──────────┐    ┌────────────────┐    ┌────────────┐  │
│  │  Unix    │    │   Session      │    │   Agent    │  │
│  │  Socket  │───>│   Manager      │───>│   Manager  │  │
│  │ Listener │    │                │    │            │  │
│  └──────────┘    └────────────────┘    └────────────┘  │
│       │                  │                    │         │
│       │           ┌──────┴──────┐      ┌─────┴─────┐   │
│       │           │  LiveSession │      │   Agent   │   │
│       │           │  - lifecycle │      │  Process  │   │
│       │           │  - buffer    │      │  (stdio)  │   │
│       │           │  - bridge    │      └───────────┘   │
│       │           └─────────────┘                       │
│       │                                                 │
│  ┌────┴─────┐                                           │
│  │  Client  │  (one per-connection task)                │
│  │  Handler │                                           │
│  └──────────┘                                           │
└─────────────────────────────────────────────────────────┘
```

## Key modules

| Module | File | Responsibility |
|--------|------|----------------|
| `daemon` | `src/daemon.rs` | Socket listener, per-client task, request routing |
| `session` | `src/session.rs` | Session lifecycle state machine, idle timers, bridge installation |
| `agent` | `src/agent.rs` | Agent spawning, ACP init handshake, capabilities probing |
| `bridge` | `src/bridge.rs` | Bidirectional message relay between client and agent |
| `state` | `src/state.rs` | Persistent state file (session registry, capabilities cache) |
| `logging` | `src/logging.rs` | Per-session log file routing via tracing layer |

## Design decisions

### Ephemeral agents

Agent processes are treated as disposable. They can be killed at any time after a turn completes. On respawn, the daemon sends `session/load` and the agent reconstructs its state from its own internal store (`~/.claude`). The daemon never owns conversation history.

### In-memory buffer

While an agent is alive, the daemon records all ACP messages flowing through the stdio pipe. This buffer serves `session/load` requests from late-joining clients when the agent is already running -- the daemon replays the buffer instead of asking the agent to replay.

### One client per session

Only one client connection can be active on a session at a time. When a second client connects, the first is disconnected. This simplifies the relay model (no fan-out) and matches the expected editor workflow.

### std::sync::Mutex over tokio::sync::Mutex

The daemon uses `std::sync::Mutex` for all shared state. Lock guards are never held across await points -- they're acquired, data is read/written, and released before any async work begins. This avoids the overhead and footgun of async-aware mutexes.

## Further reading

- [Key sequence diagrams](./sequence_diagrams.md) -- Visual walkthroughs of the major flows
- `specs/001-agent-daemon/spec.md` -- Full specification with requirements and acceptance scenarios
