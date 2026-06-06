# Post-Codegen Review Instructions

You are reviewing code that was just generated or modified. Your job is to catch issues that the generation pass may have introduced. Use your tools — read files, run commands, grep the codebase. Do not guess. Verify every finding against actual source.

## Phase 1: Research

Read the modified files. Understand what they do and how they interact with the rest of the codebase. Check imports, callers, and downstream usage. Take your time.

## Phase 2: Execute checks

Run each check below. For each, either report concrete findings or report "No issues."

### Check 1: Formatting

Run from the workspace root:

```bash
cargo fmt --check
```

List any unformatted files.

### Check 2: Mutex audit

Search modified `.rs` files for `tokio::sync::Mutex`.

This codebase uses `std::sync::Mutex` exclusively. Lock guards are never held across await points — the code acquires the lock, does synchronous work, and drops the guard before any `.await`.

For each occurrence:
- Confirm whether the guard actually crosses an await.
- If it doesn't: flag it, recommend `std::sync::Mutex`.
- If it genuinely must span an await: note it as a justified exception and explain why.

### Check 3: Duplication

Look for duplicated logic:
- Repeated blocks (5+ lines) within the new/modified code.
- Patterns that re-implement something already available elsewhere in the workspace.

Suggest concrete fixes: extract a function, call an existing helper, introduce a shared type. Only flag meaningful duplication — boilerplate and short similar patterns are fine.

### Check 4: User-facing documentation

**Skip if the change is purely internal with no observable behavior change.**

If the change affects user-visible behavior (CLI flags, config options, protocol messages, error text), check these mdbook pages:

- `md/README.md` — feature overview
- `md/quickstart.md` — usage examples and protocol snippets
- `md/configuration.md` — config file reference, file locations, CLI options

Report stale or missing documentation. If a new user-visible feature has no docs, recommend where to add them.

### Check 5: Architecture documentation

**Skip if the change doesn't affect internal structure.**

If the change adds modules, alters data flow, or introduces new design patterns, check:

- `md/design/README.md` — module table, architecture diagram, design decisions
- `md/design/sequence_diagrams.md` — interaction flows

Report inconsistencies or missing coverage.

### Check 6: Comment and docs hygiene

Review comments in modified code and any new mdbook content. Flag:

- References to task IDs, ticket numbers, or PR numbers ("T036", "fixes #123").
- Changelog language ("added for", "changed from", "replaces the old", "previously").
- References to prior implementations or removed code.
- Comments that merely restate what the code does.

Keep: explanations of current behavior and its rationale, gotcha warnings, invariant documentation.

## Phase 3: Self-evaluation

Before reporting, critically evaluate each finding:
- Did you verify it against the actual source, or are you assuming?
- Does it hold up in full context (check callers, check the types involved)?
- Is it actually a problem, or is it acceptable in this codebase's conventions?

Drop findings you can't substantiate. Add anything you missed during research.

## Phase 4: Final report

Write your final report. Use this format:

```
## 1. Formatting
[findings or "No issues."]

## 2. Mutex audit
[findings or "No issues."]

## 3. Duplication
[findings or "No issues."]

## 4. User-facing documentation
[findings or "Skipped (internal change)." or "No issues."]

## 5. Architecture documentation
[findings or "Skipped (no structural change)." or "No issues."]

## 6. Comment hygiene
[findings or "No issues."]
```

For each finding include: file path with line number, what's wrong, and a concrete fix.

Prioritize by impact: correctness > consistency > cleanliness.
