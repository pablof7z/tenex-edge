---
type: episode-card
date: 2026-07-10
session: a62822c5-d09c-4a83-9251-a3856d276ac4
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/a62822c5-d09c-4a83-9251-a3856d276ac4.jsonl
salience: product
status: active
subjects:
  - distill-failure-surfacing
  - agent-warning-injection
  - ollama-config
supersedes:
  - 2026-07-09-a62822c5d09c-fbf565a0-4-persistent-status-generation-failures-now-surfaced
related_claims: []
source_lines:
  - 1117-1118
  - 1120-1232
  - 1262-1262
  - 1317-1392
  - 1405-1449
  - 1699-1833
  - 1891-1936
captured_at: 2026-07-10T10:32:45Z
---

# Episode: Distillation failures surfaced as throttled agent-facing warnings

## Prior State

Session title/activity distillation failures were completely silent — the error branch in `runtime.rs` only wrote a debug log line to the per-session log file. No `WARN` in `daemon.log`, no error on the wire, no status event, no agent-visible signal. The old `session_errors` table and `distill_error` statusline field had been deleted in a prior schema rewrite and were dead code (always `None`). Titles stayed empty forever with no indication anything was wrong. Additionally, the `OLLAMA_HOST` env var and `providers.json` ollama entry both pointed at `localhost:8081` (squatted by Docker Desktop) instead of Ollama's actual port `11434`, so every distill call had been failing silently for an extended period.

## Trigger

User noticed empty session titles happening live and demanded analysis. Investigation revealed Ollama was unreachable on port 8081 (Docker proxy, not Ollama). Root cause: distill failures were silently swallowed. User directive: fix the Ollama setup locally, and patch the code so that if distillation fails, agents are told (using non-internal wording — not 'distillation') so they can alert the user, injected only a few times per hour to avoid pestering.

## Decision

(1) Fixed `OLLAMA_HOST` in `~/.zshrc` and `providers.json` to point at `localhost:11434`; started Ollama via `brew services start ollama`. (2) Added `distill_fail_streak INTEGER NOT NULL DEFAULT 0` and `distill_notice_at INTEGER NOT NULL DEFAULT 0` columns to `sessions` table via additive migration (`session_distill_notice.rs`, mirroring the `outbox_backoff` ensure_columns pattern — no schema-version bump, no wipe). (3) Added `record_distill_failure` and `mark_distill_notice` Store methods; wired the failure branch in `runtime.rs` observe loop to call `record_distill_failure`. (4) `assemble_turn_start` now injects a `<warnings>` entry — phrased as status/title updates not generating successfully — after 3 consecutive failures, throttled to at most once per 15 minutes while the streak persists; a successful distill resets the streak. Avoids the word 'distillation' in agent-facing text.

## Consequences

- Distill failures are now persistent state (DB columns) rather than ephemeral log lines, enabling throttled re-notification.
- Agents receive a turn-context warning they can relay to the user, closing the silent-failure gap.
- The additive migration means existing on-disk DBs gain the columns without a wipe-and-rebuild.
- 3 regression tests cover: below-threshold silence, first-fire + throttle suppression, and streak-reset-on-success.
- Not yet deployed to the live daemon — requires a daemon rebuild + restart, which would interrupt all live sessions on the machine.

## Open Tail

- Daemon restart needed to deploy the new distill-notice code to the live system.
- The `tenex-edge who` roster-cursor bug (session seen_cursor gates agent roster, showing only 2 of 14 agents for deep sessions) is diagnosed but not yet fixed.

## Evidence

- transcript lines 1117-1118
- transcript lines 1120-1232
- transcript lines 1262-1262
- transcript lines 1317-1392
- transcript lines 1405-1449
- transcript lines 1699-1833
- transcript lines 1891-1936

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-07-10-a62822c5d09c-fbf565a0-2-distillation-failures-surfaced-as-throttled-agent.json`](transcripts/2026-07-10-a62822c5d09c-fbf565a0-2-distillation-failures-surfaced-as-throttled-agent.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-07-10-a62822c5d09c-fbf565a0-2-distillation-failures-surfaced-as-throttled-agent.json`](transcripts/raw/2026-07-10-a62822c5d09c-fbf565a0-2-distillation-failures-surfaced-as-throttled-agent.json)
