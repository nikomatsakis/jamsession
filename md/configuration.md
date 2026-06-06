# Configuration

Academy reads configuration from `~/.academy/config.toml` on startup. If the file doesn't exist, defaults are used.

## Config file

```toml
# Log verbosity: error, warn, info, debug, trace
log_level = "info"
```

### Log levels

| Level | What's logged |
|-------|--------------|
| `error` | Failures only |
| `warn` | Warnings + errors |
| `info` | Lifecycle events (agent spawn/kill, client connect/disconnect) |
| `debug` | Detailed lifecycle (timer starts, state transitions) |
| `trace` | Every ACP message flowing through the daemon |

## File locations

| Path | Purpose |
|------|---------|
| `~/.academy/daemon.sock` | Unix domain socket (created at startup, `0600` permissions) |
| `~/.academy/state.json` | Persistent session registry |
| `~/.academy/config.toml` | Daemon configuration |
| `~/.academy/daemon.log` | Main daemon log (daily rotation) |
| `~/.academy/sessions/<id>/session.log` | Per-session log |

## CLI options

```
academy daemon [OPTIONS]

Options:
    --state-path <PATH>    Override the state file location
    -h, --help             Print help
```

## Environment variables

- `RUST_LOG` -- Overrides the `log_level` setting in config.toml (standard `tracing` filter syntax).

## Idle timeout

The agent idle timeout defaults to 15 minutes. After a client disconnects and 10 seconds of pipe silence pass (quiescence), the idle timer starts. When it expires, the agent process is killed.

The timeout is currently not user-configurable via config.toml (it can be overridden programmatically for integration tests).
