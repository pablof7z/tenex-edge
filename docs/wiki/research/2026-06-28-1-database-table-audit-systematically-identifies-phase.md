---
type: research-record
date: 2026-06-28
session: b9176726-a9a8-41a9-b806-c966e8c94ed7
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/b9176726-a9a8-41a9-b806-c966e8c94ed7.jsonl
source_lines: 1107-1168
agent_attribution: main
has_preregistered_criteria: true
has_method: true
has_structured_report: true
characterization: "Database table audit: systematically identifies Phase-1 tables that are write-only dead code vs. actually read in live paths; produces structured verdict table; concludes target schema should consolidate to relay_* caches + minimal local plumbing"
captured_at: 2026-06-28T10:44:31Z
---

Database table audit: systematically identifies Phase-1 tables that are write-only dead code vs. actually read in live paths; produces structured verdict table; concludes target schema should consolidate to relay_* caches + minimal local plumbing

---

Assistant: Let me confirm a suspicion: which of these tables are actually *read* by live code paths vs. write-only remnants of an unfinished migration.

User: [Tool result]: chpwd:5: command not found: _iterm2_set_profile
## Phase-1 'canonical read model' tables — are they READ anywhere outside tests/materialization?
### projects — SELECT sites:
src/state/membership.rs:29:            "INSERT INTO projects (project_id, display_slug, about, created_at, updated_at)
src/state/membership.rs:177:                    "UPDATE projects SET about=?2, updated_at=?3 WHERE project_id=?1",
src/state/membership.rs:226:            .query_row("SELECT COUNT(*) FROM projects", [], |r| r.get(0))
src/state/membership.rs:313:                .query_row("SELECT COUNT(*) FROM projects", [], |r| r.get(0))
src/state/membership.rs:336:                "SELECT about FROM projects WHERE project_id=?1",

### project_origins — SELECT sites:
src/state/membership.rs:34:            "INSERT INTO project_origins (project_id, fabric, provider_instance, native_project_key)
src/state/membership.rs:50:                "SELECT project_id FROM project_origins
src/state/membership.rs:160:        

Assistant: Now I can see what's actually going on, and you're right that it's a mess — but it's a *legible* mess. Here's the honest accounting.

## Root cause: two abandoned migrations left parallel data models

The DB isn't one design — it's three half-built ones stacked:

- **Live model** (actually read on every hook): `sessions`, `session_state`, `group_members`, `project_meta`, `chat_*`.
- **Phase-1 "canonical read model"** (issue from June 16–18): `projects`, `project_origins`, `membership`, `inbound_quarantine`. Grep proves it: **every SELECT on these lives inside the file that writes them** (`membership.rs` self-reads + backfill counts). Nothing in the hook/CLI path reads them. They're write-mostly. The cutover ("Phase 6 resolves the canonical/legacy duality") never happened.
- **Peer-state split** (also unfinished): three peer tables — `peer_sessions` (legacy), `peer_session_state`, `presence_state` — all materializing kind:30315.

So `membership` duplicates `group_members`; `projects`/`project_meta` overlap; three tables hold one event kind. That duplication *is* the mess.

## The deeper point you're driving at

Right now each relay event kind gets a **bespoke materialization table**: 39000→`project_meta`, 39001/39002→`group_members`, 30315→`peer_session_state`+`presence_state`, kind:9→`chat_messages`. That's the same sin as `owned_groups`, repeated five times: hand-rolled projections that can drift from the events.

The end-state you're describing is **one `group_events` cache** that stores raw events and applies **NIP-01 replacement** (regular events append; addressable 3xxxx replace by `kind:pubkey:d`; replaceable by `kind:pubkey`). Then *every* question — "is X admin", "channel about", "who's present", "chat log" — is a **query/view over `group_events`**, not a separate table. Nothing to keep in sync because there's nothing duplicated.

## Verdict on each table you flagged

| Table | What it is | Verdict |
|---|---|---|
| `chat_messages` | kind:9 event log per group | **→ `group_events`** (it's just events) |
| `chat_inbox` | per-(event, session) `delivered`/`notified` flags — *local* "did I inject this into this prompt yet" | Not relay state. Shrinks to a **per-session delivery cursor**, not a row per message |
| `peer_sessions` | legacy peer map (pubkey/slug/host by session) | **Migration residue** — superseded by 30315 views; drop after rewiring its 8 readers in `peers.rs` |
| `peer_session_state` / `presence_state` | two materializations of kind:30315 | **→ `group_events`** (presence is an addressable event; one view) |
| `session_aliases` | harness-native id (claude/codex/tmux/pid) → daemon's canonical `session_id`. Harness ids never touch the wire | **Keep** — load-bearing; it's the whole daemon-minted-identity model |
| `turn_state` | `working`/`turn_started_at`, flipped by turn-start/end hooks | **Duplicates** `session_state.turn_id/turn_started_at`. Local (hook state), but should **merge into `session_state`** |
| `session_endpoints` | tmux pane id (`%5`) + socket, for delivering a message to a running session | **Keep** — local OS control-plane, no relay equivalent |
| `session_errors` | last distill-LLM error, for a statusline warning | **Weakest local table** — one ephemeral row per session; could be in-memory |
| `processed_orchestration` | "I already handled this kind:9 add-agents event" — at-most-once across relay redelivery + restart replay | Legit idempotency, **but** if `group_events` dedups by event id, this **folds into a query** over it |
| `inbound_quarantine` | events that arrive **before** you've confirmed the sender is a group member get **parked, not dropped**, then replayed once membership hydrates (temporal ACL) | Concept is sound; but it's wired to the dead Phase-1 surrogate model (`project_id`). Under `group_events`, "is sender a member at event time" is a query — quarantine becomes a **flag on the event**, not a table |

## Local process state that genuinely can't be relay (keep)
`sessions` (child_pid/watch_pid/transcript/alive), `session_pubkeys`, `agent_ordinals`, `session_identity_routes`, `status_outbox`, `project_paths`. These are OS-process and derived-key facts the relay has no concept of.

---

So the target collapses to roughly: **one `group_events` relay cache** (every 30xxx/9/0 event, NIP-01 replacement) + **~7 local-process tables** + the identity/ordinal set. That deletes `owned_groups`, `membership`, `projects`, `project_origins`, `peer_sessions`, `peer_session_state`, `presence_state`, `project_meta`, `group_members`, `chat_messages`, and probably `turn_state` and `processed_orchestration`.

That's a large, load-bearing rewrite — the materializer, the ACL gate, every read model, and the who/chat/turn paths all move onto `group_events`. Before I touch anything, two scoping calls are yours:

1. **How far this round?** (a) just kill `owned_groups` as already agreed, (b) also delete the dead Phase-1 tables (`projects`/`project_origins`/`membership`/`inbound_quarantine`) since nothing reads them, or (c) commit to the full `group_events` collapse.
2. **`group_events` schema** — do you want it storing the **raw signed event JSON** (purest cache, replace by NIP-01, views parse tags on read) or **decoded columns** (`kind, pubkey, d_tag, group_h, created_at, content, tags_json`)? That choice sets everything downstream.

What's the cut?
