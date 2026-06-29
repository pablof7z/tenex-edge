---
type: noun-entry
slug: emitformat
name: "EmitFormat"
origin: extracted
source_refs:
  - transcript:377-382
---

# EmitFormat

How a context block is emitted to the harness on stdout. Selected per (host, hook-type): plain text is injected directly by Claude Code's UserPromptSubmit and opencode; Codex wraps every hook in `{systemMessage}`; Claude Code's PostToolUse only reads context from a `hookSpecificOutput` envelope.
