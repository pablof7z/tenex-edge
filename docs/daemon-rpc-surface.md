# mosaico daemon RPC surface

Companion to [daemon-design.md](daemon-design.md). This file owns the durable wire-method catalog for the per-machine daemon.

## 6. RPC surface (every method)

Coarse, lifecycle/intent-level — **not** fine-grained DB ops. The engine lives
inside the daemon, so there is no per-DB-op RPC chatter; the surface is
low-frequency lifecycle signals from hooks, CLI reads, and channel sends.

All params/results are JSON. Public selectors are full npub/hex pubkeys or a
session's current leased handle, never private runtime ids. Harness-owned IDs
are typed locators, not identity. The daemon resolves the caller from the explicit public identity
when present, then the PTY locator, harness-native locator, watched harness
process, and finally the cwd+agent scan where that fallback is safe. **The
resolution stays daemon-side** so every client path observes the same identity
rules.

### `session_start`
Spawns an in-daemon `SessionTask` (publishes profile and presence, declares its
NMP live-query demand, and routes mentions — today's `runtime::run_session`).
```jsonc
params: {"agent": "coder", "harness": "claude-code", "profile": "reviewer"|null, "harness_session": "native-id"|null, "cwd": "/path", "watch_pid": 12345|null}
result: {"pubkey": "hex"}
```
`harness_session` is a typed harness locator and never identity. A session is
addressed by its dashed public handle, such
as `@quill-codex`, backed by the session's own minted pubkey. The npub is its
permanent copy-paste resume value; the handle is a seven-day offline lease.
The provider opens the workspace root NIP-29 group through NMP, names it from the
workspace slug, and adds the session agent as a relay member before the engine
publishes presence.
The workspace and root channel are one entity with the public address `<workspace>`.
There is no local agent
allow/block file in the NIP-29 path.

### `session_end`
```jsonc
params: {"session": "npub1…"|"hex"|"handle", "cause": "manual"|"harness_hook"}
result: {"ended": true|false, "deferred": true|false}
```
The harness hook is an observation, not the owner of PTY exit classification.
For PTY-bound sessions it returns `deferred=true`; the supervisor subsequently
reports child status plus its attachment snapshot. A manual call stops the
runtime record but does not signal its process. CLI: `mosaico my session end
--self`; agents cannot target another session.

### `session_kill`
```jsonc
params: {"session": "npub1…"|"hex"|"handle", "forget": bool}
result: {"killed": true|false, "ended": true|false, "note": "pty=…"|"pid=…", "cleanup_confirmed": bool, "cleanup_failures": ["…"], "reason": "…"}
```
Process-kill, the counterpart to `session_end`. Stops the session's hosted
process (kills the owning PTY if one is tracked, else `SIGTERM`s the tracked
child pid), then stops that exact runtime generation. `killed` reflects whether process termination itself
succeeded; `reason` is populated on failure (including "no local session
matched" when `session` doesn't resolve). `mosaico sessions` sets
`forget`: the daemon atomically revokes local signer, locator, and route
authority before attempting relay cleanup. Unconfirmed NIP-29 removals remain
durable retry work. `mosaico my session kill --self` leaves that flag false,
resolves the caller from the PTY/session environment, and refuses a positional
target — an agent may only kill its own session. The CLI exits non-zero when
process termination or requested fabric cleanup is not confirmed.

### `session_pty_wrap`
```jsonc
params: {"session": "npub1…"|"hex"|"handle"}
result: {"wrapped": true, "pty_id": "…"}
       | {"wrapped": false, "refusal": "already_wrapped"|"working"|"not_resumable"|"not_found"|"kill_failed"|"resume_failed", "reason": "…"}
```
Re-homes a session started manually outside a daemon-owned PTY (no live
`pty_session` alias, so idle mentions silently black-hole — see
`turn_context::start`'s warning) into a fresh daemon PTY supervisor. Refuses
if the session already has a live `pty_session` alias (`already_wrapped`,
nothing to do), is mid-turn (`working`, to avoid losing in-flight work), or
carries no harness resume token (`not_resumable`). Otherwise kills the
manually-started process (via `session_kill`, stopping the old generation)
BEFORE resuming the SAME harness session inside a fresh PTY, so the two
steps cannot race a second caller across CLI round-trips. Only the harness's
own persisted session state survives the hop; terminal scrollback from the
killed process is lost. CLI: `mosaico my session pty-wrap-me --self`,
which resolves the caller from the PTY/session environment and refuses a
positional target — an agent may only re-home its own session. The CLI
exits non-zero unless the refusal is `already_wrapped`.

### `who`
```jsonc
params: {"workspace": "…"|null, "all_workspaces": bool, "cwd": "/path"}
result: {"now": u64, "fabric_human": "…"|null,
         "rows": [ {source, fresh, slug, channel, status, host,
                    pubkey, age_secs}, … ]}
```
Human/operator-only live fabric projection. It returns terminal-oriented
`fabric_human` text and never returns agent XML. Exact session anchors and loose
agent/group hints are rejected with guidance to use `my_session`.

### `my_session`
```jsonc
params: {"pty_session": "…"|null, "harness_session": "…"|null,
         "watch_pid": N|null, ...}
result: {"fabric": "<mosaico>…</mosaico>"}
```
Strict self-scoped agent briefing. It resolves the exact live caller and emits
`<self>`, global `<agents>` capabilities, all known workspaces, nested channels,
and typed member sessions. Every workspace joined by this exact session is
expanded; merely known workspaces stay compact. This is a pure read and does
not advance the hook-awareness cursor.

### `my_session_status`
```jsonc
params: {"title": "…", exact caller anchor fields...}
result: {"title": "…"}
```
Sets and immediately publishes the exact caller session's broadcast
status/title. CLI: `mosaico my session status <TITLE>`.

### `turn_start`
```jsonc
params: {"harness_session": "native-id", "transcript": "/path"|null, "json": bool, "cwd": "/path"}
result: {"context": "…"|null}    // the assembled injection text, or null
```
Daemon marks the turn, records the transcript path, claims pending directed
mentions from the inbox ledger, and returns the hook fabric context. A first
turn (`seen_cursor=0`) renders the relevant channel snapshot;
later turns render only rows changed since the session cursor. The cursor
advances after rendering. An absent harness locator yields `context: null`.

### `turn_check`
```jsonc
params: {"harness_session": "native-id"|null, "json": bool, "cwd": "/path"}
result: {"context": "…"|null}
```
Claims pending directed mentions once and uses a compare-and-swap cursor advance
for rate-limited fabric deltas. Hooks that lose the CAS emit no duplicate delta;
direct mentions still surface even when the delta window is closed.

### `turn_end`
```jsonc
params: {"harness_session": "native-id"}
result: {"ok": true}
```

### `doctor`
```jsonc
params: {}
result: {"relays": [...], "probe_pubkey": "hex", "publish": "OK …"|"ERR …",
         "readback": "N event(s) …"|"ERR …"}
```
The daemon's narrow direct edge performs the connectivity publish + read-back;
the client prints the existing multi-line report. Product group writes do not
use this diagnostic path.

### `tail` (streaming)
```jsonc
params: {"channel": "…"|null}
stream: {"item": {"line": "<rendered fabric line>"}}   // repeated
        … until client disconnects (Ctrl-C)
```
The daemon ensures NMP observation coverage for the requested channel, then
forwards structured events emitted by the materializer and daemon lifecycle.
Backfill comes from the canonical store; live events come from the daemon's
bounded tail broadcast. The client renders each streamed item.

### Channel messaging
The streaming read, send, reply, and blocking wait contracts live in
[daemon-rpc-messaging.md](daemon-rpc-messaging.md).

### `root_channels`
```jsonc
params: {}
result: {"channels": [ {slug, about}, … ]}
```
Returns all known workspace root channels from the daemon's cache.

### `channel_edit`
```jsonc
params: {"channel": "…", "about": "…"}
result: {"event_id": "hex", "channel": "channel-h", "about": "…", "confirmed": true}
```
Publishes an updated NIP-29 kind:39000 group metadata event for a channel.

### `channel_members`
```jsonc
params: {"channel": "…"}
result: {"members": [ {pubkey, slug, role}, … ]}
```
Returns the current membership list for the given NIP-29 group.

### `channel_add_member`
```jsonc
params: {"channel": "…", "pubkey": "hex"|null, "agent": "slug"|null}
result: {"ok": true}
```
Adds a pubkey or agent to a NIP-29 group.

### `channel_remove_member`
```jsonc
params: {"channel": "…", "pubkey": "hex"}
result: {"ok": true}
```
Removes a pubkey from a NIP-29 group.

### `channel_create`
```jsonc
params: {"name": "…", "about": "…", "parent": "…"|null,
         "parent_channel": "…"|null, "agents": [...], ...}
result: {"child_h": "…", "display_path": "…", "switched": bool,
         "orchestration_event_id": "hex"|""}
```
Creates a child channel under the caller's current channel, an explicit parent,
or the `<workspace>` root resolved from cwd.

### `channel_list`
```jsonc
params: {"channel": "…"}
result: {"channel": "…", "rooms": [ {child_h, name, about, depth}, … ]}
```
Lists the materialized child-channel tree under a channel.

### `channel_join` / `channel_leave` / `channel_switch` / `channel_archive`
```jsonc
params: {"channel": "…", "session": "npub1…"|"hex"|"handle"|null, ...}
result: {"channel": "channel-h", ...}
```
Mutates the caller session's channel membership or archives a channel.

### `statusline`
```jsonc
params: {"harness_session": "native-id"|null, "cwd": "/path", ...}
result: {"working": bool, "status": "…", "session_count": N, "member_count": N,
         "is_member": bool, "pending": N, "pending_chat": N}
```
Pure-read snapshot for the host statusline integration — no drain, no writes.

### `ping`
```jsonc
params: {}
result: {"pong": true}
```
Health-check / keep-alive.

### `pty_status`
Returns live portable PTY state with `pty_id`, `pubkey`, `npub`, and optional
public `handle`; private runtime ids are omitted.

### `operator_sessions`
Returns the canonical local control projection consumed by `mosaico
sessions`. It starts from `runtime_state='running'` rows in the daemon-owned `sessions` table,
but exposes only `pubkey`, `npub`, and the current public `handle`; the private
runtime row id never crosses this RPC boundary. Each row joins agent/harness
state, workspace-grouped joined channels, filesystem bindings, local host, and
an optional attach endpoint. Remote relay-only status rows are intentionally
excluded; they remain observable through `who` and cannot be killed by this
machine.

### `pty_send`
Sends keystrokes or text to a portable PTY session.

### `pty_spawn`
Spawns an agent through either its explicit bundle binding or an unambiguous
logical native/generic provider. This interactive boundary selects PTY launch
policy and atomically creates the canonical zero-argument bundle when none is
configured, optionally pre-loading a message. The RPC accepts no argv, command,
or bundle override.

### `pty_attach`
Accepts an npub, hex pubkey, or handle and returns the PTY target plus public identity.

### `pty_resume`
Reconstitutes a stopped harness session in PTY (re-opens the agent in its worktree).
```jsonc
params: {"session": "npub1…"}
result: {"pty_id": "…", "npub": "npub1…", "agent": "coder"}
```

### `pty_resumable`
Returns resumable rows with `pubkey`, `npub`, `runtime_state`, and an optional current `handle`.
Raw session ids are not exposed.

### Control / handshake (not user verbs)
- `hello` / `welcome` (§4)
- `please_exit` (version-skew re-exec, §4)
