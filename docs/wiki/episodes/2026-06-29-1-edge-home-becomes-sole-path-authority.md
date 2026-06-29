---
type: episode-card
date: 2026-06-29
session: b07a57a3-67a1-4c44-a8fc-58a1bb97860a
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/b07a57a3-67a1-4c44-a8fc-58a1bb97860a.jsonl
salience: architecture
status: active
subjects:
  - config-path-resolution
  - tenex-edge-home
  - tenex-dir-removal
supersedes: []
related_claims: []
source_lines:
  - 606-855
  - 886-933
  - 951-954
captured_at: 2026-06-29T10:36:07Z
---

# Episode: edge_home() becomes sole path authority — config_path() bypass and tenex_dir() eliminated

## Prior State

config_path() hardcoded home_dir().join(".tenex-edge") instead of calling edge_home(), so TENEX_EDGE_HOME was silently ignored for config loading. A separate tenex_dir() function (overridable via TENEX_DIR) existed as a parallel abstraction for llmconfig.rs and relay_log.rs, defaulting to the same path but creating a second env-var-controlled root.

## Trigger

User ran TENEX_EDGE_HOME=/tmp/te tenex-edge channels create and the daemon hung — no config.json existed at /tmp/te because config_path() was reading from ~/.tenex-edge regardless. Root-cause investigation revealed the bypass.

## Decision

config_path() now calls edge_home() so TENEX_EDGE_HOME controls config loading. tenex_dir() was deleted entirely; llmconfig.rs and relay_log.rs were migrated to use edge_home(). TENEX_EDGE_HOME is now the single source of truth for all runtime paths (config, state.db, logs, agents, relay.log).

## Consequences

- TENEX_DIR env var is no longer functional — all consumers must use TENEX_EDGE_HOME
- Isolated test/dev environments (TENEX_EDGE_HOME=/tmp/te) now correctly read config from the isolated root instead of silently falling back to ~/.tenex-edge
- The relay-info print added to cli::run() also respects edge_home(), giving users immediate visibility into which config is loaded

## Open Tail

- User requested that --channel <name> with a non-existent channel should prompt to create it rather than silently passing the string as a literal NIP-29 h-value — proposed but not implemented in this session

## Evidence

- transcript lines 606-855
- transcript lines 886-933
- transcript lines 951-954

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-06-29-1-edge-home-becomes-sole-path-authority.json`](transcripts/2026-06-29-1-edge-home-becomes-sole-path-authority.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-06-29-1-edge-home-becomes-sole-path-authority.json`](transcripts/raw/2026-06-29-1-edge-home-becomes-sole-path-authority.json)
