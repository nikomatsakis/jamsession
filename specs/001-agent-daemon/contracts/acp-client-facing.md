# Contract: ACP Client-Facing Interface

The daemon exposes an ACP endpoint on `~/.academy/daemon.sock`. Clients connect over this Unix domain socket and communicate using newline-delimited JSON-RPC 2.0.

## Transport

- Unix domain socket stream
- One JSON object per line (newline-delimited)
- Encoding: UTF-8

## Methods (client → daemon)

### `initialize`

Negotiates protocol version and capabilities.

**Request params**:
```json
{
  "protocolVersion": 1,
  "clientCapabilities": { ... },
  "clientInfo": { "name": "...", "title": "...", "version": "..." }
}
```

**Response result**: Agent capabilities (from cache or temp agent).

**Errors**:
- Agent spawn failure (temp agent for cache miss)

---

### `session/list`

Lists available sessions.

**Request params**:
```json
{
  "cwd": "/path/to/project",   // optional filter
  "cursor": null                // for pagination
}
```

**Response result**:
```json
{
  "sessions": [{ "sessionId", "cwd", "additionalDirectories", "title", "updatedAt" }],
  "nextCursor": null
}
```

---

### `session/new`

Creates a new session.

**Request params**:
```json
{
  "cwd": "/path/to/project",
  "additionalDirectories": [],
  "mcpServers": [...]
}
```

**Response result**: `{ "sessionId", "configOptions", "modes" }`

**Side effects**: Spawns agent, performs ACP init, sends interaction guidelines.

**Errors**:
- Invalid/non-existent cwd
- Agent spawn failure

---

### `session/load`

Resumes a session with full history replay.

**Request params**:
```json
{
  "sessionId": "...",
  "cwd": "...",
  "additionalDirectories": [],
  "mcpServers": [...]
}
```

**Response**: Preceded by `session/update` notifications (history replay), then result `{}`.

**Side effects**: Disconnects existing client, spawns agent if dead.

**Errors**:
- Session not found
- Agent spawn failure

---

### `session/resume`

Resumes a session without history replay.

**Request params**: Same as `session/load`.

**Response result**: `{ "configOptions", "modes" }`

**Side effects**: Disconnects existing client, spawns agent if dead (sends `session/load` to agent internally, buffers replay).

**Errors**: Same as `session/load`.

---

### `prompt/start`

Sends a user message to the active session's agent.

**Request params**:
```json
{
  "sessionId": "...",
  "content": [{ "type": "text", "text": "..." }]
}
```

**Response result**: `{}` (after turn completes).

**Errors**:
- No active session
- Agent not running
- Turn already in progress

---

## Notifications (daemon → client)

### `session/update`

Streamed during session load (history replay) and during agent turns.

**Params**:
```json
{
  "sessionId": "...",
  "update": { "sessionUpdate": "...", ... }
}
```

---

## Passthrough

Any JSON-RPC request not matching the above methods is forwarded verbatim to the agent. The response is forwarded verbatim to the client.
