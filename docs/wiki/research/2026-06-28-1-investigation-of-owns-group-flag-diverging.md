---
type: research-record
date: 2026-06-28
session: b9176726-a9a8-41a9-b806-c966e8c94ed7
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/b9176726-a9a8-41a9-b806-c966e8c94ed7.jsonl
source_lines: 407-429
agent_attribution: main
has_preregistered_criteria: true
has_method: true
has_structured_report: true
characterization: "Investigation of `owns_group` flag diverging from relay admin authority: code-traced three call sites of misuse, documented #29er failure case, proposed four-step fix to align with relay-as-truth"
captured_at: 2026-06-28T10:07:56Z
---

Investigation of `owns_group` flag diverging from relay admin authority: code-traced three call sites of misuse, documented #29er failure case, proposed four-step fix to align with relay-as-truth

---

Assistant: You're right, and the code actually proves your point against itself. Here's what I found:

## The source of truth already exists — one path uses it, the flag-path ignores it

The relay's kind:39001 admin list **is** materialized: `fabric/mod.rs:86` → `materialize_admins_snapshot` writes admins into `group_members` with `role='admin'`. And the one place that gates *real* management work — `readiness.rs:129/137` — correctly asks the **live relay**: `roles.get(&mgmt_pubkey) == Some("admin")`. That's the correct design, and it's already there.

`owns_group` / `is_group_owned` is a **separate, divergent local boolean** that means *"this daemon instance created the group"* — set only in the create paths (`readiness.rs:84/112`, `demux.rs:263`). It is **not** "am I admin per the relay." Conflating the two is the bug. Its three consumers all misuse it as a manageability proxy:

- **`turn.rs:88`** — suppresses the "ask an admin to add you" warning. If the relay shows you admin but `owns_group=0`, you get a **false** "you're not really a member, get someone else to add you" warning — exactly backwards.
- **`runtime.rs:251`** and **`chat_publish.rs:103`** — gate per-session-room auto-rename. Relay says you're admin, local flag says no → rename silently never fires.

## #29er confirms the failure mode

#29er is **not in this `state.db` at all** — no `owned_groups`, no `project_meta`, no `group_members` row. So `is_group_owned("29er")` returns false even though this backend created it. The hook still shows it active because the hook reads live daemon/relay state, not this stale cache snapshot. This is the multi-daemon / dev-build gap (or a dropped `publish_signed_checked`-gated cache write) — the local "I created it" boolean diverged from relay truth, which is the whole reason a local boolean shouldn't be the authority.

## The fix

1. Add `is_group_admin(project, pubkey)` on `Store` → `group_members` where `role='admin'` (this mirrors relay 39001).
2. Repoint the three gates at `is_group_admin(project, mgmt_pubkey)` instead of `is_group_owned`. Manageability = "relay lists my mgmt key as admin," never "I created it."
3. When `group_members` is empty for a group (cache miss like #29er), do a live 39001 fetch before deciding — the same fallback already noted for 39002 membership. Relay is authoritative; the cache is just a warm copy.
4. Demote/delete `owns_group` — at most a non-authoritative "created-here" hint, never a gate.

Want me to implement this? The only real decision is step 4: fully drop the `owns_group` column, or keep it purely informational while removing it from every gate.
