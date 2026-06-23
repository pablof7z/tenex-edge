---
type: episode-card
date: 2026-06-19
session: 460b104d-e734-4dbd-9a8b-6e8182b1d699
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/460b104d-e734-4dbd-9a8b-6e8182b1d699.jsonl
salience: reversal
status: active
subjects:
  - session-codename
  - session-identity
  - session-short-code
supersedes:
  - 2026-06-10-2-sessionid-newtype-enforces-correct-display-formatting
  - 2026-06-09-3-session-short-ids-changed-from-uuid
related_claims: []
source_lines:
  - 1-1
  - 139-151
  - 796-805
  - 907-915
  - 1040-1066
  - 1122-1137
  - 1752-1754
captured_at: 2026-06-23T14:48:15Z
---

# Episode: Session display identity replaced: 6-char hex hash → NATO phonetic codename

## Prior State

Sessions displayed as a 6-character hex hash produced by `session_short_code` in `src/util.rs`. `SessionId`'s `Display` impl routed through this function, so every `who`/statusline/TUI/turn-intro/envelope render showed the hash. The JSON protocol field was named `short_code`.

## Trigger

User directive (line 1): replace the 6-char hash with a human-readable codename (NATO phonetic word + number) everywhere — `codex@laptop [session bravo42]` style. Later refined (line 907) to use 4 digits instead of 2 for larger namespace.

## Decision

Renamed `session_short_code` → `session_codename`, now generating `word+NNNN` (e.g. `bravo4217`). JSON protocol field renamed `short_code` → `codename`. All ~20 Rust call sites, docs, and agent-facing prompt text updated. Codename space is 26×10000 = 260,000. Codename is display/addressing only — canonical session id remains the identity.

## Consequences

- All user-visible session references (statusline, who, turn intro, envelopes, tmux resume TUI) now show codenames instead of hex hashes
- Wire protocol field `short_code` → `codename` in daemon `session_start` RPC results and opencode hook stdout; safe because no consumer parses that field
- Codename resolution added to `tmux resume` (`resume_by_codename`) since the TUI now shows codenames as the only session handle
- Codename min length is 6 (word+4digits), so the existing `>= 6` lookup gate in recipient resolution passes automatically
- Running daemon must be rebuilt/restarted to show new-format codenames
- Codename space (260,000) is not collision-free at scale — explicitly a display convenience, never identity

## Open Tail

*(none)*

## Evidence

- transcript lines 1-1
- transcript lines 139-151
- transcript lines 796-805
- transcript lines 907-915
- transcript lines 1040-1066
- transcript lines 1122-1137
- transcript lines 1752-1754

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-06-19-1-session-display-identity-replaced-6-char.json`](transcripts/2026-06-19-1-session-display-identity-replaced-6-char.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-06-19-1-session-display-identity-replaced-6-char.json`](transcripts/raw/2026-06-19-1-session-display-identity-replaced-6-char.json)
