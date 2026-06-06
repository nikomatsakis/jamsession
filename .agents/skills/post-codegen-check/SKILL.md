---
name: post-codegen-check
description: >
  Quality gate for Rust code changes — run after generating or modifying code.
  Checks formatting, flags tokio::sync::Mutex misuse, identifies duplicated logic,
  verifies mdbook documentation consistency, and enforces comment hygiene.
  Use this skill as a subagent after any code generation pass, even if the
  user doesn't explicitly ask for a review.
---

# Post-Codegen Check

Spawn a subagent to perform the quality review. Pass it:

1. A description of what code was generated or modified (files, purpose, scope of change).
2. The instructions file at `references/review-instructions.md` found in this skill's directory.

Example invocation:

```
Agent({
  description: "Post-codegen quality check",
  prompt: `Review the following code changes for quality issues.

Changed files: <list the modified/new files>
Purpose: <brief description of what was done>

Follow the instructions in .agents/skills/post-codegen-check/references/review-instructions.md exactly. Read that file first, then execute each check. Report findings in the format specified.`
})
```

Do NOT attempt the checks yourself — delegate to the subagent so findings come from a fresh perspective without context bias from the generation pass.
