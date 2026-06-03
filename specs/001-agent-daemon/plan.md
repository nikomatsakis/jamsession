# Implementation Plan: Agent Daemon

**Branch**: `001-agent-daemon` | **Date**: 2026-06-02 | **Spec**: `specs/001-agent-daemon/spec.md`

**Input**: Feature specification from `/specs/001-agent-daemon/spec.md`

## Summary

A long-running Rust daemon that manages ephemeral agent processes via the Agent Client Protocol (ACP). The daemon listens on a Unix domain socket, manages session lifecycle (create/load/resume), bridges ACP messages between clients and agents, and supports MCP-over-ACP tool servers. Agents are spun down when idle and respawned on demand.

## Technical Context

**Language/Version**: Rust 2024 edition (stable)

**Primary Dependencies**:
- `agent-client-protocol` — ACP types, message parsing, connection builders, and transport primitives
- `acpr` — agent launcher (registry-based resolution, binary download/caching, `ConnectTo<Client>` transport)
- `tokio` — async runtime (multi-threaded)
- `serde` / `serde_json` — JSON-RPC serialization
- `tokio-util` (codec) — newline-delimited JSON framing on Unix socket
- `nix` — process management, signal handling
- `uuid` — session/connection IDs
- `chrono` — timestamps for state/events

**Storage**: File-based (`~/.academy/state.json`) — no database

**Testing**: `cargo test` with integration tests using real Unix sockets and mock agent processes

**Target Platform**: Linux

**Project Type**: Single binary with two modes — `daemon` (long-running background process on Unix socket) and `acp` (stdio-based ACP client for editors/tools)

**Performance Goals**: Not a priority for initial implementation

**Constraints**: Single-threaded event loop per session is acceptable; no multi-tenant; Linux-only

**Scale/Scope**: One daemon per user, managing 1-20 concurrent sessions across project directories

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Constitution is a blank template — no project-specific gates or constraints defined. Proceeding without gate violations.

## Project Structure

### Documentation (this feature)

```text
specs/001-agent-daemon/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit-tasks command)
```

### Source Code (repository root)

```text
src/
├── main.rs              # Entry point, CLI args, daemon bootstrap
├── daemon.rs            # Daemon event loop, Unix socket listener
├── session.rs           # Session state machine, agent bridging
├── agent.rs             # Agent process spawn/kill
├── bridge.rs            # Bidirectional relay between client and agent ACP streams
├── state.rs             # State file persistence (~/.academy/state.json)
├── guidelines.md        # Interaction guidelines (included via include_str!)
└── error.rs             # Error types

tests/
├── integration/
│   ├── session_lifecycle.rs   # Full session create/load/resume flows
│   └── agent_lifecycle.rs     # Spin-up, idle detection, spin-down
└── helpers/
    └── mock_agent.rs          # Minimal ACP-speaking process for tests
```

**Structure Decision**: Single-crate binary with module-based organization. No workspace or library splitting needed at this scale. Integration tests use a mock agent binary to exercise the daemon end-to-end.

## Complexity Tracking

No constitution violations to justify.
