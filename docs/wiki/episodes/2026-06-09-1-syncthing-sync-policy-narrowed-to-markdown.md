---
type: episode-card
date: 2026-06-09
session: 561703ff-71f3-43ce-923c-c69c735f83c5
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/561703ff-71f3-43ce-923c-c69c735f83c5.jsonl
salience: product
status: active
subjects:
  - syncthing-sync-policy
  - stignore-rules
supersedes: []
related_claims: []
source_lines:
  - 1-52
captured_at: 2026-06-17T23:49:42Z
---

# Episode: Syncthing sync policy narrowed to markdown-only

## Prior State

The syncthing directory had no .stignore filter, so all files (git objects, Rust code, Cargo artifacts, build outputs) were being synced.

## Trigger

User directive: only markdown documents should sync — no git, no code, no build artifacts.

## Decision

Adopt a three-rule .stignore exploiting Syncthing's first-match-wins semantics: !*/ (allow directory recursion), !*.md (allow markdown files), * (ignore everything else).

## Consequences

- Only .md files (README.md, M1.md, docs/**/*.md, Plans/**/*.md) will be synced across devices
- All code (.rs), config (.toml), lockfiles (.lock), git objects, and build artifacts are excluded
- Future non-markdown files added to the repo will be automatically excluded without updating .stignore

## Open Tail

*(none)*

## Evidence

- transcript lines 1-52

