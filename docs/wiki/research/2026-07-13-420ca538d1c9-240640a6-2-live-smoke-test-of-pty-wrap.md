---
type: research-record
date: 2026-07-13
session: 420ca538-d1c9-4af5-91fc-3e634d2d8442
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/420ca538-d1c9-4af5-91fc-3e634d2d8442.jsonl
source_lines: 1546-1594
agent_attribution: subagent (Live smoke-test pty-wrap-me)
has_preregistered_criteria: true
has_method: true
has_structured_report: true
characterization: "Live smoke test of pty-wrap-me command: 5-step empirical test (non-PTY repro → black-hole confirmation → fix → re-home verification) against real Claude agent and croissant relay, VERDICT: PASS"
captured_at: 2026-07-13T08:03:44Z
---

Live smoke test of pty-wrap-me command: 5-step empirical test (non-PTY repro → black-hole confirmation → fix → re-home verification) against real Claude agent and croissant relay, VERDICT: PASS

---

User: [Agent task result: Agent "Live smoke-test pty-wrap-me" finished]
Cleanup complete: both my relays stopped, containers removed, host fleet daemon left intact. Here is the report.

---

# pty-wrap-me live smoke test — VERDICT: PASS (all 5 steps)

**Overall:** `tenex-edge my session pty-wrap-me --self` **does** re-home a live non-PTY agent into a daemon-owned PTY and **unblocks its stuck mail** — proven end-to-end against a real Claude agent, real host auth, and a real croissant relay.

## Run metadata
- **Merged code confirmed:** `45e18b63` (PR #407) is an ancestor of HEAD; the lab compiled the current master `tenex-edge v0.1.0` in-container (doctor build 2m20s) and every RPC ran that binary at `/state/target/debug/tenex-edge`.
- **Relay:** `ws://192.168.65.1:28471` (fresh, run_id `20260713-105300`, isolated `relay-data/`)
- **Profile:** `claude` (harness `claude-code`, transport **pty**), backend pubkey `ac4518b2…`, isolated `.container-state/claude`, real host Claude OAuth (profile `doctor` passed: "claude hooks: host-auth install present").
- **Model:** `haiku` (`claude-haiku-4-5-20251001`) — cheapest useful.
- **Topology note:** Apple `container` invocations are per-call micro-VMs (the wrapper even clears the stale `daemon.sock` each run), so the agent, the mention, and the wrap were driven inside **one** container/daemon (shared pid-ns) via a scripted orchestration — the only faithful way to exercise a live kill+resume. No secrets/nsec/auth printed.

## Per-step results

**STEP 1 — reproduce non-PTY condition — PASS.** Direct `claude --model haiku` launched under a raw `python pty.fork` (NOT `tenex-edge launch`). Its SessionStart hook registered: session `te-18c1ca2f2b3b38ba-0`, pubkey `da75174887a4…`, channel `workspace`, `child_pid=44`, `resume_id=438cd82a-d11c-4bda-bd1a-2fd3f81058ae` (its Claude session).

**STEP 2 — confirm genuinely non-PTY — PASS.** `tenex-edge pty list` → "No portable-pty sessions found." Aliases = `harness_session` + `watch_pid` only; **no `pty_session`**. *Bonus (PR #405):* the injected turn context carried, verbatim:
> "This session is not hosted in a daemon PTY. Messages sent to you while you are idle are NOT pushed to you — they wait in your inbox until your next turn. Run `tenex-edge my session pty-wrap-me --self` to re-home into a daemon PTY, or keep taking turns."

And it correctly **disappears** from the post-wrap PTY turns in the same hook log.

**STEP 3 — confirm the idle black-hole — PASS.** Mention published as backend `ac4518b2…` (≠ victim) — `kind:9` p-tagging the victim, relay `publishing… success`, stored (id `35d43ebea71e`). Inbox BEFORE wrap:
```
INBOX: 35d43ebea71e | te-18c1ca2f2b3b38b | pending | delivered_at=0 | from ac4518b2
```
No inject/deliver line in daemon log → `DeferNoEndpoint` (no live pty endpoint).

**STEP 4 — run the fix — PASS.** `TENEX_EDGE_SESSION=<sid> tenex-edge my session pty-wrap-me --self` → `session te-18c1ca2f2b3b38ba-0 re-homed into daemon PTY claude-1783929249-24` (exit 0). Refusal paths: outside a session → exit 1, `must run inside a tenex-edge PTY session`; second call → `session is already inside a live daemon PTY; nothing to do` (exit 0, no-op).

**STEP 5 — verify re-home — PASS (a,b,c).**
- (a) old process killed: `OLD PID 44 DEAD (killed)`.
- (b) same session now in a daemon PTY: `pty list` → `meadow-claude … claude --resume 438cd82a-…` (same resume token); aliases now include `pty_session|claude-1783929249-24` + `pty_socket`; sessions row unchanged id `te-18c1ca2f2b3b38b` / **same pubkey `da75174887a4`** / alive / child_pid `44 → 337`.
- (c) stuck mention delivers: inbox flipped
```
INBOX: 35d43ebea71e | te-18c1ca2f2b3b38b | injected | delivered_at=1783929251
```
daemon log: `[pty] pending messages delivered to endpoint claude-1783929249-24 for te-18c1ca2f2b3b38ba-0`. (`injected` is the correct PTY terminal state — mention pasted as a prompt — with `delivered_at>0`, per `src/state/inbox.rs`.)

## One environment issue worth flagging (not a product bug)
The first relay came up on the **default shared port 19888** and another agent from your running fleet/another lab (a `claude-acp` daemon, key `32aba804…`, at container IP `.40`) created the NIP-29 `workspace` group first and owned membership, so my keys were `blocked: unknown member` for `kind:9`. This is exactly the shared-port hazard the skill warns about. Re-running on a unique port (`28471`) let my daemon create the group cleanly (members `[ac4518b2 backend, 934fa264, da75174887a4 victim]`) and everything flowed. Host fleet daemon (PID 72830) and its pty-supervisors were left untouched.

## Cleanup
`scripts/cleanup-lab` run for both lab.env files: relays `12279` and `52804` stopped, containers removed, host fleet daemon `72830` intact.

Evidence artifacts (all absolute): `/private/tmp/claude-501/-Users-pablofernandez-src-tenex-edge/420ca538-d1c9-4af5-91fc-3e634d2d8442/tasks/bdipns1w7.output` (final PASS run), `…/tasks/b0ek4g1l2.output` (wrap-proven, mention-blocked run), and the hook log `/Users/pablofernandez/src/tenex-edge/.container-state/claude/tenex/edge/sessions/438cd82a-d11c-4bda-bd1a-2fd3f81058ae/hook-calls.jsonl` (PR #405 warning present pre-wrap, absent post-wrap).
