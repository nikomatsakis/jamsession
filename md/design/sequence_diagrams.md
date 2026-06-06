# Key sequence diagrams

## Fresh connection -- new session

```mermaid
sequenceDiagram
    participant C as Client
    participant D as Daemon
    participant A as Agent

    C->>D: connect [Unix socket]
    C->>D: initialize
    Note over D: Cache hit or spawn temp agent
    D-->>C: capabilities

    C->>D: session/list
    D-->>C: sessions [from state file]

    C->>D: session/new [dir: /project]
    D->>A: spawn [/project]
    D->>A: initialize
    A-->>D: capabilities
    D->>A: session/new [mcpServers]
    A-->>D: session created [sessionId]
    D-->>C: session created
    Note over D: Persist session to state file

    D->>A: prompt/start [interaction guidelines]
    A-->>D: prompt/start response

    Note over C,A: Bridged — daemon records + relays
```

## Reconnect -- load session (agent dead)

```mermaid
sequenceDiagram
    participant C as Client
    participant D as Daemon
    participant A as Agent

    C->>D: connect [Unix socket]
    C->>D: initialize
    D-->>C: capabilities [from cache]

    C->>D: session/load [sessionId]
    D->>A: spawn
    D->>A: initialize
    A-->>D: capabilities
    D->>A: session/load [sessionId]
    A-->>D: session/update [history replay]
    D-->>C: session/update [forwarded]
    A-->>D: session/load response
    D-->>C: session/load response

    Note over C,A: Bridged — daemon records + relays
```

## Reconnect -- load session (agent alive)

```mermaid
sequenceDiagram
    participant C as Client
    participant D as Daemon
    participant A as Agent

    Note over A: Already running

    C->>D: connect
    C->>D: initialize
    D-->>C: capabilities [from cache]

    C->>D: session/load [sessionId]
    Note over D: Disconnect previous client if any
    D-->>C: session/update [replay from in-memory buffer]
    D-->>C: session/load response

    Note over C,A: Client bridged to live stream
```

## Idle spin-down

```mermaid
sequenceDiagram
    participant C as Client
    participant D as Daemon
    participant A as Agent

    Note over A: Running, turn complete

    C->>D: disconnect
    Note over D: No clients remain

    Note over D,A: 10s quiescence timer starts
    Note over D,A: Reset on any bridge activity
    Note over D,A: 10s of silence achieved
    Note over D,A: Idle timeout (15min default)...

    D->>A: kill
    Note over A: Terminated

    Note over D: Buffer discarded<br/>Session record persists
```

## Auto-respawn on crash

```mermaid
sequenceDiagram
    participant C as Client
    participant D as Daemon
    participant A as Agent
    participant A2 as Agent (respawned)

    C->>D: (bridged, active session)
    Note over A: Crashes unexpectedly
    A--xD: connection lost

    Note over D: Detect crash, attempt respawn (once)
    D->>A2: spawn
    D->>A2: initialize
    D->>A2: session/load
    A2-->>D: ready

    Note over C,A2: Bridge re-installed, client continues
```

## One-client-per-session enforcement

```mermaid
sequenceDiagram
    participant C1 as Client 1
    participant C2 as Client 2
    participant D as Daemon
    participant A as Agent

    C1->>D: session/load [sessionId]
    Note over C1,A: C1 bridged to agent

    C2->>D: session/load [sessionId]
    Note over D: Cancel C1's connection
    D--xC1: disconnected
    Note over C2,A: C2 bridged to agent
```

## Directory deleted -- session cleanup

```mermaid
sequenceDiagram
    participant D as Daemon
    participant A as Agent

    Note over D: Periodic health check (60s)
    D->>D: detect /project gone

    D->>A: kill
    Note over A: Terminated

    Note over D: Remove session from state file
```
