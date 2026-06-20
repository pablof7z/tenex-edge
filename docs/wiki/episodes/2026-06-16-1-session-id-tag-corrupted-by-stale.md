---
type: episode-card
date: 2026-06-16
session: ea5dd578-ca5d-4f31-8427-3a253dd735e8
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/ea5dd578-ca5d-4f31-8427-3a253dd735e8.jsonl
salience: root-cause
status: active
subjects:
  - session-id-format
  - opencode-plugin
  - nostr-tag-contract
supersedes: []
related_claims: []
source_lines:
  - 1-43
  - 285-622
  - 748-764
captured_at: 2026-06-18T00:37:28Z
---

# Episode: Session-id tag corrupted by stale opencode plugin after JSON output change

## Prior State

The daemon's session-start hook was updated to return JSON `{"session_id":"te-...","short_code":"..."}` instead of a bare string, but the installed opencode plugin at `~/.config/opencode/plugin/tenex-edge.ts` was never re-dropped. The old plugin did `SID = out.trim()`, storing the entire JSON string as the session ID.

## Trigger

User observed published Nostr events where the `d` tag contained `tenex-edge:{"session_id":"te-18b9855f015daef0-29127","short_code":"90fe1f"}` and the `session-id` tag held the same stringified JSON — a plain bare id was expected.

## Decision

Updated the installed opencode plugin to parse the daemon's JSON output format: try `JSON.parse(trimmed)` and extract `.session_id`, falling back to treating the output as a bare string for backward compatibility.

## Consequences

- New opencode sessions will correctly use bare `te-*` session IDs in Nostr `d` and `session-id` tags.
- Existing sessions with corrupted JSON-string session IDs remain until those opencode processes restart.
- Plugin is now forward-compatible: handles both JSON and bare-string output from the daemon.
- Deploying a binary change that alters a hook-output contract requires re-running `tenex-edge install` to drop the updated plugin file.

## Open Tail

- Existing corrupted session events on relays are not cleaned up; they age out naturally or persist with bad tags until those processes restart.
- The `resolve_recipient` change in the diff (slug@project lookup now falls back to relay-sourced DB) appears unrelated but was also in-flight.

## Evidence

- transcript lines 1-43
- transcript lines 285-622
- transcript lines 748-764

