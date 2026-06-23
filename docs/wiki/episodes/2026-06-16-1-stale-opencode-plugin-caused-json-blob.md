---
type: episode-card
date: 2026-06-16
session: ea5dd578-ca5d-4f31-8427-3a253dd735e8
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/ea5dd578-ca5d-4f31-8427-3a253dd735e8.jsonl
salience: root-cause
status: active
subjects:
  - session-id-tag-format
  - opencode-plugin-sync
supersedes: []
related_claims: []
source_lines:
  - 1-43
  - 586-622
  - 748-764
captured_at: 2026-06-16T09:36:05Z
---

# Episode: Stale opencode plugin caused JSON blob to leak into Nostr session-id tags

## Prior State

The `d` and `session-id` Nostr tags were publishing a stringified JSON object (e.g. `tenex-edge:{"session_id":"te-…","short_code":"90fe1f"}`) instead of the bare `te-*` session ID string. The system assumed session IDs flowing through the hook/plugin layer were always bare strings.

## Trigger

User observed the published kind 30315 event and identified that the `d` tag contained a full JSON blob and the `session-id` tag also contained stringified JSON, explicitly stating: 'NO json in the string! it should be: d, tenex-edge:$long-session-id and the session-id should have only a single id, not a stringified json'

## Decision

Root cause was that the daemon binary was updated to output JSON (`{"session_id":"te-…","short_code":"…"}`) on session-start, but the installed opencode plugin (`~/.config/opencode/plugin/tenex-edge.ts`) was never re-dropped. The old plugin did `SID = out.trim()`, capturing the entire JSON string as the session ID. Fix: updated the plugin to parse JSON output first (`JSON.parse(trimmed).session_id`) with a bare-string fallback, and reinstalled it.

## Consequences

- New opencode sessions will correctly store and publish bare session IDs in Nostr tags
- Existing sessions with corrupted JSON-blob session IDs will persist until those opencode processes restart
- The plugin now handles both output formats (JSON and bare string) for forward/backward compatibility
- Any future binary output format changes must be paired with a plugin re-drop — the install/reinstall path needs to be part of the deployment checklist

## Open Tail

- Corrupted sessions from already-running opencode processes remain in the relay; no cleanup mechanism discussed
- No CI or runtime guard added to prevent format drift between binary output and plugin parsing

## Evidence

- transcript lines 1-43
- transcript lines 586-622
- transcript lines 748-764

