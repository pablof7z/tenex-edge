# Nouns

Back to the [wiki index](../_index.md).

## Nouns (63 entities)

| Noun | Name | Origin | Definition |
|------|------|--------|------------|
| [activity](nouns/activity.md) | Activity | extracted | Used for social Activity notes (kind:1 without p tag) |
| [activity-distillation](nouns/activity-distillation.md) | Activity distillation | extracted | the process of distilling the agent's recent conversation transcript into a one-line intent that becomes its Activity note and live Status; it is LLM-only with no heuristic fallback and intent is not recoverable from tool calls by rule |
| [agent-identity-pubkey](nouns/agent-identity-pubkey.md) | agent identity / pubkey | extracted | durable, ordinal-keyed (agent, ordinal); base agent at ordinal 0, higher ordinals for concurrent instances of same agent; reused deterministically across rooms |
| [backend-orchestration](nouns/backend-orchestration.md) | backend orchestration | extracted | kind:9 subscription p-tagged to the backend's identity, independent of any project, maintaining one global subscription per backend |
| [channel](nouns/channel.md) | channel | extracted | in this project, a channel and a project are the same abstraction—a NIP-29 group that may or may not have a parent; the only distinction is whether parent_hint is set |
| [channel-readiness-gate](nouns/channel-readiness-gate.md) | channel readiness gate | extracted | idempotent `ensure_channel_ready(ctx: ChannelCtx)` method on `Nip29Provider` in `src/fabric/nip29/readiness.rs` that all three domain publish methods (`publish`, `publish_checked`, `set_status`) converge on; uses TTL-cached fast path, per-channel single-flight mutex, local SQLite read-model checks, and recursive parent ensures before provisioning a channel |
| [channel-traffic](nouns/channel-traffic.md) | channel traffic | extracted | signed by session keys but received via #h tag, regardless of signer |
| [chatmessage](nouns/chatmessage.md) | ChatMessage | extracted | scoped to the project group by its `h` tag. It is ambient project context; live sessions see it going forward only. Chat fans out to every alive project session — routing is by pubkey + current channel, no session IDs on the wire. |
| [distillation](nouns/distillation.md) | distillation | extracted | LLM-driven process triggered on each new user message (turn-start) that generates a session title and activity line |
| [domain-publish](nouns/domain-publish.md) | domain publish | extracted | publish above the codec seam: DomainEvent publishes that converge on three methods (publish, publish_checked, set_status) on Nip29Provider and encode via the wire codec |
| [domainevent](nouns/domainevent.md) | DomainEvent | extracted | The closed set of things that travel on the fabric. A codec encodes each of these to a wire envelope and decodes wire envelopes back into these. |
| [ensure-channel-ready](nouns/ensure-channel-ready.md) | ensure_channel_ready | extracted | the unified NIP-29 channel-provisioning primitive: recursively ensures parent groups exist, creates/confirms target channels, propagates admin roles downward, and adds agents as members with roster confirmation retry—used for per-session rooms, orchestration, explicit channels, and project provisioning |
| [explicit-channel-scope](nouns/explicit-channel-scope.md) | explicit channel scope | extracted | a session where project != work_root, created with the channel as a subgroup of the root project |
| [h-tag](nouns/h-tag.md) | h tag | extracted | the NIP-29 group identifier, derived inside the codec from the DomainEvent's project field, which is always SessionRecord::route_scope() — either the channel (if set) or the per-session room |
| [identity](nouns/identity.md) | Identity | extracted | (agent, machine) — the same slug on another machine is a different key |
| [live-session](nouns/live-session.md) | live session | extracted | at most one per (ordinal-pubkey, channel); binds to native harness session id via (pubkey, channel) → native_id resume key |
| [members](nouns/members.md) | members | extracted | the kind:39002 p-tag set, keyed by #d == group |
| [ms-server-1](nouns/ms-server-1.md) | MS-server-1 | extracted | extraction of 5 project RPC handlers (list, edit, members, add, remove) from server.rs into server/rpc/project.rs |
| [ms-server-2](nouns/ms-server-2.md) | MS-server-2 | extracted | extraction of demux functionality (handle_incoming, derive_and_emit_tail_events) from server.rs into server/demux.rs |
| [ms-server-3](nouns/ms-server-3.md) | MS-server-3 | extracted | extraction of background spawn tasks (spawn_pruner, spawn_idle_watcher) from server.rs into server/background/ |
| [ms-state-1](nouns/ms-state-1.md) | MS-state-1 | extracted | extraction of quarantine functionality (quarantine_inbound, replay_quarantine, clear_quarantine) from state.rs into state/quarantine.rs |
| [ms-state-2](nouns/ms-state-2.md) | MS-state-2 | extracted | extraction of status_outbox publish-queue methods from state.rs into state/outbox.rs |
| [ms-state-3](nouns/ms-state-3.md) | MS-state-3 | extracted | extraction of channels.rs from state.rs |
| [ms-state-4](nouns/ms-state-4.md) | MS-state-4 | extracted | extraction of canonical membership (admit_member, revoke_member, is_member_at) from state.rs into state/membership.rs |
| [ms-state-4b](nouns/ms-state-4b.md) | MS-state-4b | extracted | extraction of legacy group_members roster methods from state.rs into state/group_members.rs, deferred as follow-up to MS-state-4 |
| [nip-29-channel](nouns/nip-29-channel.md) | NIP-29 channel | extracted | the NIP-29 group a session publishes to: for a subgroup task room, the child h supplied via TENEX_EDGE_CHANNEL; otherwise, the working-directory project |
| [normalized-session-observation](nouns/normalized-session-observation.md) | normalized session observation | extracted | a hook observation describing the harness, harness-owned external ID, resume token, tmux pane, watched PID, and CWD, reported to the daemon which resolves the canonical session ID — minting new, reattaching via alias, or superseding stale |
| [nostrdelivery](nouns/nostrdelivery.md) | NostrDelivery | extracted | raw Nostr delivery that subscribes to relay streams for a given Scope |
| [ordinal-durable-pubkeys](nouns/ordinal-durable-pubkeys.md) | ordinal-durable pubkeys | extracted | durable keys keyed by (agent, ordinal); allocated as lowest ordinal not already live for agent in room; same pubkey reused across rooms |
| [parent](nouns/parent.md) | parent | extracted | parent group id in project_meta from relay-authored kind:39000 'parent' tag, empty for top-level projects |
| [parent-hint](nouns/parent-hint.md) | parent_hint | extracted | a parameter to the channel-provisioning primitive: None creates a top-level project (root channel), Some(parent) creates a subgroup under that parent |
| [per-session-room](nouns/per-session-room.md) | per-session room | extracted | a NIP-29 subgroup room_h parented under the work-root project that a human-initiated session lives in; minted idempotently per session |
| [persessionrooms](nouns/persessionrooms.md) | perSessionRooms | extracted | a boolean configuration field in ~/.tenex-edge/config.json controlling whether sessions mint per-session rooms (true) or use the project channel (false, the default) |
| [profile-identity-resolution](nouns/profile-identity-resolution.md) | profile/identity resolution | extracted | replaceable lookup data that should be performed on-demand via fetch and cache rather than maintained as a long-lived subscription |
| [project-channel](nouns/project-channel.md) | project channel | extracted | the bare work-root project group where sessions land their fabric events when per-session rooms are disabled (the default behavior); no subgroup is minted |
| [project-session-context](nouns/project-session-context.md) | project (session context) | extracted | For a subgroup task room, the child h supplied via TENEX_EDGE_CHANNEL; otherwise, the working-directory project |
| [project-skill](nouns/project-skill.md) | project skill | extracted | a verified, committed record of what works to launch the app: exact package commands, environment variables, patches, and drivers |
| [pubkey](nouns/pubkey.md) | pubkey | extracted | durable identifier created only when a second agent is added to a channel — no longer transient, not exploded per-session |
| [rejection-log-format](nouns/rejection-log-format.md) | rejection log format | extracted | includes event context prefix `kind:N  id=<12-hex>  h=<group>  ` when event is available, enabling correlation with relay server logs |
| [relay-connection](nouns/relay-connection.md) | relay connection | extracted | one per daemon; single nostr-sdk Client/pool that all agent identities' subscriptions multiplex onto |
| [relay-log](nouns/relay-log.md) | relay log | extracted | persistent log of every outgoing relay event and every relay rejection, appended to ~/.tenex-edge/relay.log |
| [roomdecision](nouns/roomdecision.md) | RoomDecision | extracted | an enum determining where a newly-born session's fabric events land: Mint creates a per-session NIP-29 subgroup under a parent, UseExisting routes to an existing group (via orchestration override or the default project) |
| [route-scope](nouns/route-scope.md) | route_scope | extracted | The NIP-29 group id this session currently routes under — its channel when set, else its per-session room (`project`). All fabric publishing (chat/mentions/proposals), local chat routing, `who`/statusline scoping, and turn-context deltas key on this so `channels switch` actually moves the session to a different room without restarting. `project` alone is stale the moment `channel` is set. |
| [routing-model](nouns/routing-model.md) | routing model | extracted | pubkey-based routing where p-tags carry the receiver's durable pubkey; no session-derived keys or session-specific wire tags |
| [routing-scope](nouns/routing-scope.md) | routing scope | extracted | The NIP-29 group id this session currently routes under — its channel when set (a `channels switch` moved it to a subgroup), else its per-session room (`project`) |
| [running](nouns/running.md) | Running | extracted | launching and interacting with the app as a user would (CLI at its command, server at its socket, GUI at its window), not just executing code |
| [scope](nouns/scope.md) | Scope | extracted | subscription scope that Delivery implementations convert into wire-level |
| [status](nouns/status.md) | Status | extracted | addressed by `(author pubkey, group id)` |
| [statusline](nouns/statusline.md) | statusline | extracted | renders the awareness floor for a host status bar, displaying agent name, project name, session identifier, channel title, and live activity |
| [subgroup-session](nouns/subgroup-session.md) | subgroup session | extracted | a session stored under its child group id (h), not the working-directory project |
| [subscription-ceiling](nouns/subscription-ceiling.md) | subscription ceiling | extracted | the number of concurrent subscriptions (REQs) a relay allows per connection; acts as bottleneck limiting relay scalability |
| [subscription-redesigned](nouns/subscription-redesigned.md) | subscription (redesigned) | extracted | three entity-keyed aggregate REQs: #h[channels] kinds[9,30315,30023], #p[pubkeys] kinds[9,30023], #d[groups] kinds[39000,39001,39002]; narrow add-REQs for new entities; compacted at daemon start |
| [subscriptionid](nouns/subscriptionid.md) | SubscriptionId | extracted | deterministically derived from the filter's content; re-subscribing the same filter replaces the existing relay subscription instead of opening a new one |
| [te-session](nouns/te-session.md) | @te_session | extracted | a tmux user option that stores the session identifier for tenex-edge statusline |
| [tenex-edge-channel](nouns/tenex-edge-channel.md) | TENEX_EDGE_CHANNEL | extracted | present only for sessions launched into a subgroup task |
| [tenex-edge-statusline](nouns/tenex-edge-statusline.md) | tenex-edge statusline | extracted | the one-line status bar rendering showing identity, project, session ID, channel title, and live activity; reads harness statusline JSON from stdin and fails open (exits 0 with empty output when daemon is down) |
| [tmux](nouns/tmux.md) | $TMUX | extracted | environment variable containing three comma-separated values: socket_path,server_pid,session_id |
| [tmux-pane](nouns/tmux-pane.md) | TMUX_PANE | extracted | a stable tmux pane ID from the $TMUX_PANE environment variable (e.g. "%5"), present only when the hook fires inside a tmux session |
| [tmuxstatuscommand](nouns/tmuxstatuscommand.md) | tmuxStatusCommand | extracted | custom tmux status-format string for agent sessions that overrides the default tenex-edge statusline command, using tmux format variables #{q:@te_session}, #{@te_agent}, and #{q:@te_cwd} to reference session identity |
| [transient-signer](nouns/transient-signer.md) | transient signer | extracted | a second-personality cryptographic key for concurrent same-agent sessions in the same scope; when two sessions of the same agent share a channel, the second gets a transient key to maintain distinct identities |
| [transport](nouns/transport.md) | Transport | extracted | private implementation detail; fabric-layer boundary |
| [work-root](nouns/work-root.md) | work_root | extracted | canonical root project in a hierarchy, resolved by walking up parent links in project_meta |
| [working-directory-project](nouns/working-directory-project.md) | working-directory project | extracted | the repo this harness runs in (resolved from the current working directory) |
