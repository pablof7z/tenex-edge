---
type: research-record
date: 2026-06-28
session: b9176726-a9a8-41a9-b806-c966e8c94ed7
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/b9176726-a9a8-41a9-b806-c966e8c94ed7.jsonl
source_lines: 1673-1710
agent_attribution: main
has_preregistered_criteria: true
has_method: true
has_structured_report: true
characterization: "Schema rewrite verification: independent build and test validation — 255/0 lib tests passing, −5,785 LOC removed, 11 tables confirmed, but 77 integration test errors remain unresolved"
captured_at: 2026-06-28T11:55:04Z
---

Schema rewrite verification: independent build and test validation — 255/0 lib tests passing, −5,785 LOC removed, 11 tables confirmed, but 77 integration test errors remain unresolved

---

Assistant: Verified status — and one real gap the workflow under-reported:

## ✅ Core rewrite landed and independently verified
- **Schema is exactly the 11 agreed tables** (`relay_channels/_channel_members/_profiles/_status/_events` + `sessions/session_aliases/identities/inbox/outbox/project_roots`).
- **`cargo build` (lib+bins) clean** — 5 dead-code warnings only.
- **`cargo test --lib` = 255 passed / 0 failed**, including new invariant tests that actually prove the design: NIP-01 replacement (addressable *and* replaceable), pubkey-unique-per-channel (admin supersedes member), idempotent register-per-external-id, outbox retry.
- **−5,785 LOC net** (3,048 added, 8,833 deleted). `owns_group`, `is_session_room`, the three peer tables, the dead Phase-1 model, `processed_orchestration` — all gone.

## ⚠️ Gap: integration test crates don't compile
The workflow only ran `cargo test --lib`. The **`tests/` integration crates have 77 compile errors** — all mechanical staleness against the old API (`Session.project`→`channel_h`, `is_group_member`→`is_channel_member`, `upsert_project_meta`→`upsert_channel`, plus refs to removed `is_session_room`/`is_group_owned`/`route_scope`). No structural problems, just an unmigrated test harness.

## ⚠️ Still true (unchanged)
- Stale *comments* in a few files still name old tables (`presence_state`, `processed_orchestration`) — cosmetic.
- **No live e2e through the hook path** — your standing rule. A clean build ≠ a working daemon.

I'm launching a focused agent to migrate the integration test crates to the new API and iterate to green — this is bounded, mechanical work.

User: [Tool result]: Async agent launched successfully.
agentId: a54dd116ac9d1b44e (internal ID - do not mention to user. Use SendMessage with to: 'a54dd116ac9d1b44e', summary: '<5-10 word recap>' to continue this agent.)
The agent is working in the background. You will be notified automatically when it completes.
Do not duplicate this agent's work — avoid working with the same files or topics it is using.
output_file: /private/tmp/claude-501/-Users-pablofernandez-src-tenex-edge/b9176726-a9a8-41a9-b806-c966e8c94ed7/tasks/a54dd116ac9d1b44e.output
Do NOT Read or tail this file via the shell tool — it is the full subagent JSONL transcript and reading it will overflow your context. If the user asks for progress, say the agent is still running; you'll get a completion notification.

Assistant: The integration-test migration agent is running in the background. I'll be notified when it's done.

While it works, here's where things stand:

**Landed and verified (lib):** the 11-table relay-cache/local-state schema is in, builds clean, 255/0 lib tests green, −5,785 LOC. The design invariants you specified are enforced and tested — relay caches are rebuildable, a pubkey is unique per channel, admin authority comes only from `relay_channel_members`, idempotency lives in the `inbox` ledger, NIP-01 replacement in `relay_events`.

**In flight:** integration test crates being migrated to the new API.

**Still your call after this converges:**
1. This is all on `master` in your working tree (uncommitted). When the test agent finishes I'd suggest moving it to a branch before anything else.
2. The standing e2e-through-the-hook-path requirement is unmet — that needs real claude/codex/opencode + a relay, which no agent can do for you. That's the gate between "compiles and unit-tests pass" and "actually works."

I'll report back with the integration-test result the moment the agent completes.

