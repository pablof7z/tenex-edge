---
type: episode-card
date: 2026-06-09
session: 561703ff-71f3-43ce-923c-c69c735f83c5
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/561703ff-71f3-43ce-923c-c69c735f83c5.jsonl
salience: root-cause
status: active
subjects:
  - syncthing-stignore
  - markdown-sync-policy
supersedes: []
related_claims: []
source_lines:
  - 1-53
captured_at: 2026-06-12T20:04:01Z
---

# Episode: Syncthing .stignore: markdown-only sync with correct first-match-wins semantics

## Prior State

No .stignore existed; Syncthing would sync everything including .git, Rust code, build artifacts, and .DS_Store files

## Trigger

User directive to sync only markdown documents. Initial naive ignore pattern was wrong — Syncthing uses first-match-wins, so a top-level * exclusion would prevent directory traversal before .md files could be discovered inside nested paths.

## Decision

Adopt a three-rule .stignore: !*/ (un-ignore all directories so Syncthing recurses into them), !*.md (un-ignore markdown files), * (ignore everything else). This exploits first-match-wins ordering to allow traversal while excluding all non-markdown content.

## Consequences

- Only .md files (README.md, M1.md, docs/**/*.md, Plans/**/*.md) are synced
- Git repo, Cargo artifacts, Rust source, scripts, and .DS_Store are excluded
- Directory traversal is preserved — nested markdown inside src/ or other code directories would still sync

## Open Tail

*(none)*

## Evidence

- transcript lines 1-53

