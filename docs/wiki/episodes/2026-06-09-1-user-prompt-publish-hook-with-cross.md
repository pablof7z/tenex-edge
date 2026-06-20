---
type: episode-card
date: 2026-06-09
session: 98f9939c-f42b-43dd-baba-d9a176d4b2d7
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/98f9939c-f42b-43dd-baba-d9a176d4b2d7.jsonl
salience: product
status: active
subjects:
  - user-prompt-hook
  - usernsec-config
  - cross-key-publishing
supersedes: []
related_claims: []
source_lines:
  - 1-1
  - 2239-2243
  - 2311-2319
captured_at: 2026-06-17T23:52:07Z
---

# Episode: User prompt publish hook with cross-key signing

## Prior State

No mechanism existed for user-submitted prompts to be published to the Nostr fabric; the daemon had no access to the human user's signing key

## Trigger

User requested a hook that publishes the user's prompt as a kind:1 OP (no e-tag) on prompt submission, signed by the nsec stored in ~/.tenex/config.json

## Decision

Added a `user-prompt-submit` CLI hook → `user_prompt` daemon RPC that resolves the session, reads `userNsec` from config, builds a kind:1 event with `h` (project) and `p` (agent) tags, signs with the user's key, and publishes over the daemon's existing relay connection (cross-key: daemon authenticates as its own key, publishes event signed by userNsec). Fail-open: errors are eprintln'd, never blocking the editor.

## Consequences

- userNsec field added to Config struct with serde rename='userNsec' to match the camelCase config.json key
- Daemon now publishes events signed by a different key than its relay-auth key — validated by probe that B-signed events over an A-authed connection land under B's authorship
- Project list/edit commands also use userNsec for signing kind:9002 group management events
- Created the feedback-loop risk that led to the codec Mention disambiguation fix

## Open Tail

- Hook integration with editor workflows (Claude Code, Codex) needs per-editor wiring

## Evidence

- transcript lines 1-1
- transcript lines 2239-2243
- transcript lines 2311-2319

