# Quickstart: Agent Daemon

## Build & Run

```bash
# Build
cargo build

# Run daemon in foreground (for development)
cargo run

# The daemon listens on ~/.academy/daemon.sock
```

## Test Scenarios

### 1. Daemon starts and listens

```bash
# Start daemon
cargo run &

# Verify socket exists
ls ~/.academy/daemon.sock
```

### 2. Client connects and initializes

```bash
# Connect and send initialize (using socat for manual testing)
echo '{"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":1,"clientCapabilities":{"fs":{"readTextFile":true,"writeTextFile":true},"terminal":true},"clientInfo":{"name":"test","title":"Test","version":"0.1.0"}}}' | socat - UNIX-CONNECT:~/.academy/daemon.sock
```

Expected: capabilities response from cached agent init.

### 3. Session list (empty initially)

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"session/list","params":{"cwd":"/tmp/test-project"}}' | socat - UNIX-CONNECT:~/.academy/daemon.sock
```

Expected: `{"sessions":[], "nextCursor": null}`

### 4. Create new session

Requires a valid project directory and agent binary available:

```bash
echo '{"jsonrpc":"2.0","id":2,"method":"session/new","params":{"cwd":"/tmp/test-project","additionalDirectories":[],"mcpServers":[]}}' | socat - UNIX-CONNECT:~/.academy/daemon.sock
```

Expected: session created response with `sessionId`.

### 5. Integration test with mock agent

The test suite includes a mock agent that speaks minimal ACP (responds to `initialize`, `session/new`, `session/load`). This allows testing the full daemon without a real Claude Code agent.

```bash
cargo test --test integration
```

## Key Verification Points

| Scenario | What to verify |
|----------|---------------|
| Daemon startup | Socket created, state file loaded |
| Client init | Capabilities returned (cache or temp agent) |
| Session new | Agent spawned, ACP handshake, guidelines sent |
| Session load (dead) | Agent spawned, history replayed to client |
| Session load (alive) | Buffer replayed, client bridged to live stream |
| Session resume | No replay to client, bridged immediately |
| Idle spin-down | Agent killed after quiescence + timeout |
| Directory deleted | Agent killed, session removed |
