# Research: Agent Daemon

## R1. ACP Protocol Implementation in Rust

**Decision**: Use the `agent-client-protocol` crate for ACP message types, parsing, and transport primitives.

**Rationale**: The `agent-client-protocol` crate provides typed ACP messages, serialization/deserialization, and transport abstractions. Using it avoids re-implementing the protocol layer and ensures compatibility with the ACP spec.

**Alternatives considered**:
- Hand-rolled JSON-RPC layer with `serde_json` — more work, risk of spec drift
- `jsonrpsee` — heavyweight, designed for HTTP/WebSocket, doesn't model ACP semantics

## R2. Unix Domain Socket Listener with Tokio

**Decision**: Use `tokio::net::UnixListener` with per-connection task spawning.

**Rationale**: Tokio provides first-class Unix socket support. Each client connection spawns a task that owns the socket read/write halves. The daemon's session registry is shared via `Arc<Mutex<...>>` or a channel-based actor.

**Alternatives considered**:
- `mio` directly — too low-level, would reimplement what tokio already provides
- Thread-per-connection — unnecessary complexity, no blocking operations

## R3. Agent Process Communication (stdio bridging)

**Decision**: Spawn agent as a child process, communicate via stdin/stdout using `tokio::process::Command` with piped stdio. Use `tokio_util::codec::LinesCodec` for framing.

**Rationale**: ACP mandates stdio transport. Tokio's process module provides async read/write on child stdin/stdout. `LinesCodec` handles newline-delimited framing naturally.

**Alternatives considered**:
- Named pipes — adds filesystem coordination complexity, no benefit over stdin/stdout
- Unix socket per agent — deviates from ACP spec which mandates stdio

## R4. State File Persistence Strategy

**Decision**: Atomic write (write to temp file, rename) on every mutation. Load on startup. Accept staleness risk.

**Rationale**: The state file is small (session metadata only, not conversation history). Atomic rename ensures no corruption on crash. Writing on every mutation (session create/delete, event delivery) keeps the file current without a separate flush timer.

**Alternatives considered**:
- SQLite — overkill for a small JSON document that changes infrequently
- Periodic flush — risks losing state on crash between flushes
- `sled`/`rocksdb` — excessive complexity for simple key-value state

## R5. Idle Detection and Spin-Down

**Decision**: Track "last message timestamp" on the agent's stdio pipe. After turn completion (prompt/start response received) and no connected clients, start a quiescence timer (10s of silence). After quiescence, start idle timer. On expiry, SIGTERM then SIGKILL.

**Rationale**: The spec requires: (1) turn must complete, (2) pipe must be quiet for 10s, (3) idle timeout elapses. A simple state machine: `Active → TurnComplete → Quiescent → IdleTimerRunning → Kill`. Any new message or client connection resets to `Active`.

**Alternatives considered**:
- Kill immediately after turn completion — too aggressive, agent might have cleanup
- Only time-based (no quiescence check) — could kill mid-output if turn detection has a race

