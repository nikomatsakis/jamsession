# Data Model: Agent Daemon

## Architecture Overview

The daemon presents itself as an **Agent** to clients. Each incoming Unix socket
connection spawns a task running `Agent.builder()...connect_to(transport)`. The
daemon handles `initialize` and `session/list` itself, then on `session/new` or
`session/load`, it connects to the real agent subprocess as a **Client**.

```text
┌─────────────────────────────────────────────────────────┐
│ Daemon Process                                          │
│                                                         │
│   Unix Socket Listener                                  │
│     │                                                   │
│     ├── Client A connection task (Agent role)           │
│     │     └── Agent subprocess (Client role)            │
│     │                                                   │
│     └── Client B connection task (Agent role)           │
│           └── Agent subprocess (Client role)            │
│                                                         │
│   State file (shared)                                   │
└─────────────────────────────────────────────────────────┘
```

## Connection Handling Pseudocode

```rust
// Main loop: accept connections on the Unix socket
loop {
    let (stream, _) = listener.accept().await?;
    let daemon_state = daemon_state.clone();
    tokio::spawn(handle_client(stream, daemon_state));
}

async fn handle_client(stream: UnixStream, state: Arc<DaemonState>) {
    let (read_half, write_half) = stream.into_split();
    let transport = ByteStreams::new(write_half, read_half);

    // The daemon IS the agent from the client's perspective.
    // Static handlers cover the pre-session phase (initialize, session/list).
    // Once a session is activated, a dynamic handler takes over and proxies
    // all subsequent messages to the real agent.
    Agent.builder()
        .name("academy-daemon")

        // Phase 1: Pre-session requests handled by the daemon directly.
        //
        // NOTE on blocking: `on_receive_request` handlers block the dispatch
        // loop. For anything async (spawning agents, network), use `cx.spawn()`
        // to move the work off the dispatch loop. The responder is Send + 'static
        // and can be used from the spawned task.

        .on_receive_request(async |req: InitializeRequest, responder, cx| {
            // Capabilities lookup may need to spawn a temp agent — don't block.
            let state = state.clone();
            cx.spawn(async move {
                let caps = state.get_or_populate_capabilities(&req).await?;
                responder.respond(caps)
            })
        }, on_receive_request!())

        .on_receive_request(async |req: SessionListRequest, responder, cx| {
            // This is fast (state file read) — could inline, but spawn for consistency.
            let sessions = state.list_sessions(&req.cwd);
            responder.respond(SessionListResponse { sessions, next_cursor: None })
        }, on_receive_request!())

        // Phase transition: session/new spawns agent, installs bridge handler.
        // Spawning an agent is async (process spawn + ACP init handshake),
        // so we spawn a task. The client waits for the response naturally
        // (JSON-RPC response arrives when the spawned task calls responder.respond).
        .on_receive_request(async |req: NewSessionRequest, responder, cx| {
            let state = state.clone();
            cx.spawn(async move {
                let live_session = state.activate_session_new(&req).await?;

                // Install a dynamic handler that proxies ALL subsequent messages
                // from this client to the agent. Dynamic handlers are checked
                // before static ones, so once installed the bridge claims everything.
                cx.add_dynamic_handler(BridgeHandler::new(
                    live_session.agent_connection.clone(),
                ))?.run_indefinitely();

                responder.respond(live_session.new_session_response())
            })
        }, on_receive_request!())

        // Phase transition: session/load — same pattern, with replay
        .on_receive_request(async |req: LoadSessionRequest, responder, cx| {
            let state = state.clone();
            cx.spawn(async move {
                let live_session = state.activate_session_load(&req).await?;

                // Replay history to client (session/update notifications)
                for msg in &live_session.replay_buffer {
                    cx.send_notification(msg.clone())?;
                }

                cx.add_dynamic_handler(BridgeHandler::new(
                    live_session.agent_connection.clone(),
                ))?.run_indefinitely();

                responder.respond(LoadSessionResponse {})
            })
        }, on_receive_request!())

        .connect_to(transport)
        .await;
}

/// Dynamic handler that proxies all messages to the real agent.
/// Installed after session activation — claims every incoming dispatch.
struct BridgeHandler {
    agent_cx: ConnectionTo<Agent>,
}

impl BridgeHandler {
    fn new(agent_cx: ConnectionTo<Agent>) -> Self {
        Self { agent_cx }
    }
}

impl HandleDispatchFrom<Client> for BridgeHandler {
    async fn handle_dispatch_from(
        &mut self,
        message: Dispatch,
        _client_cx: ConnectionTo<Client>,
    ) -> Result<Handled<Dispatch>, Error> {
        // Forward the message to the real agent.
        // Responses from the agent flow back to the client via the
        // agent_cx's response routing (set up during session activation).
        self.agent_cx.send_proxied_message(message)?;
        Ok(Handled::Yes)
    }
}
```
```

### Session Activation (after session/new or session/load)

```rust
// When the daemon needs to talk to the real agent:
async fn spawn_agent(session: &Session) -> Result<AgentConnection> {
    // Use `acpr` (v0.4+) to resolve and launch the agent.
    // Acpr resolves the agent binary from registry and
    // implements ConnectTo<Client> — so it's a ready-made transport.
    let agent_transport = Acpr::new("claude-acp");

    // Connect to the agent as a Client
    Client.builder()
        .name("academy-daemon")
        .connect_with(agent_transport, async |agent_cx| {
            // Initialize the ACP connection to the agent
            agent_cx.send_request(InitializeRequest::new(ProtocolVersion::V1))
                .block_task().await?;

            // Create or load the session on the agent side
            let session = agent_cx.build_session(&session.cwd)
                .with_mcp_servers(configured_mcp_servers)?
                .block_task()
                .start_session().await?;

            // Now bridge: relay messages between the client connection
            // (Agent-role task above) and this agent connection
            // ... (details TBD — likely involves sharing the agent_cx
            //      with the per-client Agent-role task)
            Ok(())
        })
        .await
}
```

---

## Entities

### DaemonState (persisted to `~/.academy/state.json`)

The root-level persistent state loaded on startup and updated atomically on mutations.

| Field | Type | Description |
|-------|------|-------------|
| sessions | `Vec<SessionRecord>` | All known sessions |
| capabilities_cache | `Option<CachedCapabilities>` | Cached agent initialize response |

### SessionRecord

A single session's persistent metadata.

| Field | Type | Description |
|-------|------|-------------|
| session_id | `String` | ACP session ID (e.g., `sess_abc123`) |
| cwd | `PathBuf` | Working directory for this session |
| created_at | `DateTime<Utc>` | When session was first created |
| updated_at | `DateTime<Utc>` | Last state file update for this session |

### CachedCapabilities

Cached response from agent `initialize` — avoids spawning a temp agent on every client connect.

| Field | Type | Description |
|-------|------|-------------|
| client_capabilities | `serde_json::Value` | The full `clientCapabilities` object that produced this cache |
| response | `serde_json::Value` | The full `initialize` response JSON |

---

## Runtime Entities (in-memory only)

### Daemon (runtime)

Shared state across all client connection tasks.

| Field | Type | Description |
|-------|------|-------------|
| state | `Arc<Mutex<DaemonState>>` | Persistent state (loaded from file) |
| sessions | `Arc<Mutex<HashMap<String, LiveSession>>>` | Active session state |
| listener | `UnixListener` | Accepting new client connections |

### LiveSession

In-memory representation of a session with runtime state.

| Field | Type | Description |
|-------|------|-------------|
| record | `SessionRecord` | Persistent state (synced to file) |
| agent | `Option<AgentHandle>` | Running agent process, if any |
| client_count | `usize` | Number of connected clients (0 or 1) |
| buffer | `Vec<JsonRpcMessage>` | In-memory message buffer (agent lifetime) |
| lifecycle_state | `LifecycleState` | Current lifecycle state machine position |

### AgentHandle

The daemon's Client-role connection to a running agent subprocess.

| Field | Type | Description |
|-------|------|-------------|
| connection | `ConnectionTo<Agent>` | The daemon's client-side connection to the agent |
| task | `JoinHandle<Result<()>>` | Background task running the agent process + protocol |

---

## Enums

### LifecycleState

State machine for agent process lifecycle.

```
AgentDead → Spawning → Active → TurnComplete → Quiescent → IdleTimerRunning → Kill → AgentDead
```

| Variant | Description |
|---------|-------------|
| `AgentDead` | No agent process running |
| `Spawning` | Agent being spawned + ACP init in progress |
| `Active` | Agent running, client connected or turn in progress |
| `TurnComplete` | Last prompt/start response received, no client |
| `Quiescent` | 10s of pipe silence after turn completion |
| `IdleTimerRunning` | Idle timeout counting down |

Transitions:
- Any message on pipe → resets to `Active`
- Client connects → resets to `Active`
- `AgentDead` + (client request or external event) → `Spawning`

---

## Relationships

```
Daemon 1──1 DaemonState
Daemon 1──* LiveSession
LiveSession 1──1 SessionRecord
LiveSession 1──? AgentHandle
LiveSession 1──* JsonRpcMessage (buffer)
```

---

## Validation Rules

- `session_id` must be non-empty and unique across all sessions
- `cwd` must be an absolute path; session is removed if path doesn't exist (FR-005)
- Only one client may be active on a session at a time (FR-017)
- Buffer is cleared when agent process dies

## State Transitions

### Session Creation Flow
1. Client sends `session/new` → daemon creates `SessionRecord` with new ID
2. Daemon spawns agent process, connects as Client
3. Daemon sends `initialize` then `session/new` to agent
4. Agent returns session ID confirmation
5. Daemon persists record to state file
6. Daemon bridges: relays `prompt/start`, `session/update`, etc. between client and agent

### Session Removal
- Working directory deleted → remove record, kill agent
- No explicit "delete session" API (sessions are managed by the agent internally)
