---
type: episode-card
date: 2026-06-26
session: a3e59cbb-77a6-4d97-87e0-5609354f2d19
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/a3e59cbb-77a6-4d97-87e0-5609354f2d19.jsonl
salience: architecture
status: active
subjects:
  - tenex-edge-launch
  - eager-provisioning
  - nip29-group-membership
supersedes: []
related_claims: []
source_lines:
  - 1-5
  - 483-495
  - 579-628
captured_at: 2026-06-26T20:05:45Z
---

# Episode: Eager channel provisioning moved to tenex-edge launch time

## Prior State

When `tenex-edge launch <agent> <project>` is invoked, the agent's tmux session spawns immediately without ensuring the target NIP-29 group exists or the agent is a member. Provisioning (ensure_channel_ready, open_project) is deferred until the harness fires its session-start hook.

## Trigger

User directive that provisioning machinery exists in the codebase but is not wired to tenex-edge launch — should proactively provision at launch time, not be deferred to session-start.

## Decision

Implement provision_before_spawn in rpc_tmux_spawn to eagerly provision before spawn_agent. For --channel launches call ensure_channel_ready; for bare projects call open_project. Detect second-instance scenarios. Cap both at 8 seconds (fail-open) to prevent slow relays blocking spawn.

## Consequences

- Channel/group and membership provisioning now synchronous at launch time instead of deferred to session-start hook
- System invariant: by rpc_tmux_spawn return, group exists and agent is a member (best-effort, fail-open at 8s timeout)
- User perceives channel ready when agent pane opens, not after session-start fires
- Second-instance detection added (checks latest_alive_session_for_agent_in_project) but ordinal-based alternate pubkeys (slug-1.json) remain unimplemented
- Per-session room minting stays deferred to session-start (requires harness session ID not available at launch)

## Open Tail

- Ordinal support for multiple instances of same agent slug (slug-1.json, slug-2.json) — detection exists but alternate key provisioning unimplemented
- Regression testing for sessions using old deferred-provisioning behavior

## Evidence

- transcript lines 1-5
- transcript lines 483-495
- transcript lines 579-628

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-06-26-1-eager-channel-provisioning-moved-to-tenex.json`](transcripts/2026-06-26-1-eager-channel-provisioning-moved-to-tenex.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-06-26-1-eager-channel-provisioning-moved-to-tenex.json`](transcripts/raw/2026-06-26-1-eager-channel-provisioning-moved-to-tenex.json)
