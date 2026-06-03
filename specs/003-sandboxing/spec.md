# Feature Specification: Agent Sandboxing

**Feature Branch**: `003-sandboxing`

**Created**: 2026-06-03

**Status**: Draft

**Depends on**: `001-agent-daemon`

## Goal

Wrap agent processes spawned by the academy daemon in bubblewrap (`bwrap`) sandboxes, providing filesystem and network isolation. This ensures agents cannot access arbitrary host filesystem paths or make network connections outside of the ACP stdio channel.

## Requirements

### Filesystem Isolation

- **FR-001**: The agent's project directory (session `cwd`) MUST be mounted read-write inside the sandbox.
- **FR-002**: System libraries required for the agent runtime (e.g., `/usr`, `/lib`) MUST be mounted read-only.
- **FR-003**: Agent-specific paths MUST be mounted as needed (e.g., `~/.claude` read-write for Claude Code). These are hard-coded per agent type for now.
- **FR-004**: No other paths from the host are visible inside the sandbox.
- **FR-005**: Secrets within the project directory itself are not filtered (accepted risk for now).

### Network Isolation

- **FR-006**: The agent MUST have no network access (empty network namespace via `--unshare-net`).
- **FR-007**: All external interactions happen through the ACP stdio channel — there is no other communication path out of the sandbox.

### Configuration

- **FR-008**: Mount configuration (which agent-specific paths to expose) is hard-coded per agent type initially.
- **FR-009**: The sandbox is applied by using `acpr`'s `with_command_wrapper` API to wrap the resolved agent command in a `bwrap` invocation.

## Implementation Approach

The daemon uses `Acpr::new("claude-acp").with_command_wrapper(...)` to wrap the agent binary in a `bwrap` invocation. The command wrapper builds the bwrap argument list:

```rust
.with_command_wrapper(|cmd| {
    let mut wrapped = ResolvedCommand {
        program: "bwrap".into(),
        args: vec![
            "--ro-bind", "/usr", "/usr",
            "--ro-bind", "/lib", "/lib",
            "--bind", &session.cwd, &session.cwd,
            "--bind", &claude_dir, &claude_dir,
            "--unshare-net",
            "--",
        ].into_iter().map(OsString::from).collect(),
        envs: cmd.envs,
    };
    wrapped.args.push(cmd.program);
    wrapped.args.extend(cmd.args);
    wrapped
})
```

## Assumptions

- Bubblewrap (`bwrap`) is available on the host system (Linux with user namespaces enabled).
- The `acpr` crate (v0.4+) provides `with_command_wrapper` for wrapping agent commands.
- The daemon from 001-agent-daemon provides the agent spawning infrastructure that this feature extends.

## Source Module

Implementation lives in `src/sandbox.rs`:
- `build_bwrap_command(cwd, agent_paths) -> ResolvedCommand` — constructs the bwrap argument list
- Per-agent-type path configuration (hard-coded initially)

## Related Specifications

- **Agent Daemon**: `specs/001-agent-daemon/` — provides the daemon and agent spawning infrastructure that sandboxing wraps around.
