---
type: research-record
date: 2026-06-28
session: b9176726-a9a8-41a9-b806-c966e8c94ed7
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/b9176726-a9a8-41a9-b806-c966e8c94ed7.jsonl
source_lines: 279-429
agent_attribution: main
has_preregistered_criteria: true
has_method: true
has_structured_report: true
characterization: "Investigated #29er absence from local cache and traced owns_group flag misuse across three code paths; verdict: owns_group (local-creation marker) incorrectly gates management operations, but relay admin status is the true authority; proposed fix: check relay role instead"
captured_at: 2026-06-28T07:22:42Z
---

Investigated #29er absence from local cache and traced owns_group flag misuse across three code paths; verdict: owns_group (local-creation marker) incorrectly gates management operations, but relay admin status is the true authority; proposed fix: check relay role instead

---

Assistant: You're right on both counts. Let me stop defending the design and actually look at the facts — first, where is #29er in the cache, and what does the relay say about admin status.

User: [Tool result]: === owned_groups matching 29er ===
=== project_meta matching 29er ===
=== projects matching 29er ===
Error: in prepare, no such column: name
  SELECT * FROM projects WHERE name LIKE '%29er%' OR id LIKE '%29er%';
                 error here ---^
=== group_members matching 29er ===
=== any table: grep 29er via project_origins/project_paths ===
Error: in prepare, no such column: project
  SELECT * FROM project_origins WHERE project LIKE '%29er%';
                        error here ---^

Assistant: #29er is completely absent from `owned_groups`, `project_meta`, and `group_members` — yet the hook reported it as an active channel. Let me find where the hook is even seeing it, and dump the relay's actual admin list.

User: [Tool result]: === brute-force: dump every table row LIKE 29er ===

Assistant: #29er is genuinely nowhere in this `state.db`. So two separate things are true and I need to nail both down. Let me find (a) where the hook gets "#29er [2 members]" from, and (b) how the code decides "can I manage this group" today.

User: [Tool result]: chpwd:5: command not found: _iterm2_set_profile
=== hook / awareness source for 'active channels' ===
src/cli/who/awareness.rs:49:    write_section(&mut out, "Other active channels, last 10m:", &other);
src/cli/who/awareness.rs:103:    write_section(&mut out, "Other active channels, last 10m:", &other);
src/state/channels.rs:214:    /// channel + its rendered subtree). Drives the "N other active channels in

=== where 39001 (admins) is read/used ===
src/cli.rs:22:mod admin;
src/cli.rs:35:pub use admin::render_fabric;
src/cli.rs:37:use admin::{parse_since, render_tail_event};
src/cli.rs:109:            admin::tail(admin::TailOpts {
src/cli.rs:148:        Cmd::Project { action } => admin::project(action).await,
src/cli.rs:149:        Cmd::Channels { action } => admin::channels(action).await,
src/cli.rs:150:        Cmd::Agent { action } => admin::agent(action).await,
src/cli.rs:151:        Cmd::Doctor => admin::doctor().await,
src/cli/turn.rs:97:                 has relay admin access (e.

Assistant: The relay's 39001 admin list *is* already materialized (`materialize_admins_snapshot` → `group_members` with role=`admin`). So the source of truth exists. The question is where `owns_group` gates management instead of consulting it. Let me trace both.

User: [Tool result]: chpwd:5: command not found: _iterm2_set_profile
=== consumers of is_group_owned / owns_group / mark_group_owned ===
src/runtime.rs:251:                                        && s.is_group_owned(&p.project).unwrap_or(false)
src/cli/turn.rs:88:            let locally_owned = s.is_group_owned(&scope).unwrap_or(false);
src/state/core.rs:44:            "ALTER TABLE owned_groups ADD COLUMN owns_group INTEGER NOT NULL DEFAULT 1",
src/state/groups.rs:130:    pub fn mark_group_owned(&self, project: &str, ts: u64) -> Result<()> {
src/state/groups.rs:132:            "INSERT INTO owned_groups (project, created_at, owns_group) VALUES (?1, ?2, 1)
src/state/groups.rs:133:             ON CONFLICT(project) DO UPDATE SET owns_group=1",
src/state/groups.rs:139:    pub fn is_group_owned(&self, project: &str) -> Result<bool> {
src/state/groups.rs:141:            "SELECT COUNT(*) FROM owned_groups WHERE project=?1 AND owns_group=1",
src/state/schema.rs:215:    -- rows may exist with owns_group=0 when a no-

User: [Tool result]: chpwd:5: command not found: _iterm2_set_profile
4:    pub fn upsert_project_meta(&self, project: &str, about: &str, ts: u64) -> Result<()> {
13:    pub fn get_project_meta(&self, project: &str) -> Result<Option<String>> {
28:    pub fn group_display_name(&self, project: &str) -> Result<String> {
45:    pub fn latest_session_title_for_project(&self, project: &str) -> Result<Option<String>> {
61:    pub fn upsert_group_metadata(
82:    pub fn list_group_metadata(&self) -> Result<Vec<(String, String, String, String)>> {
103:    pub fn group_parent(&self, group: &str) -> Result<Option<String>> {
117:    pub fn list_project_meta(&self) -> Result<Vec<(String, String)>> {
128:    // ── NIP-29 owned groups + membership ─────────────────────────────────
130:    pub fn mark_group_owned(&self, project: &str, ts: u64) -> Result<()> {
132:            "INSERT INTO owned_groups (project, created_at, owns_group) VALUES (?1, ?2, 1)
1

User: [Tool result]: 78	    let mut blocks: Vec<String> = Vec::new();
79	
80	    if first_turn {
81	        // Warn only when this daemon is not the local owner for the group. If it
82	        // owns the group, session-start/room-minting is responsible for signing
83	        // the member-add itself; a cache miss here is a transient local state,
84	        // not a user action item.
85	        let should_warn_not_member = {
86	            let s = store.lock().expect("store mutex poisoned");
87	            let not_member = !s.is_group_member(&scope, &rec.agent_pubkey).unwrap_or(true);
88	            let locally_owned = s.is_group_owned(&scope).unwrap_or(false);
89	            not_member && !locally_owned
90	        };
91	        if should_warn_not_member {
92	            blocks.push(format!(
93	                "[tenex-edge] WARNING: this agent ({slug}) \
94	                 is not a member of the NIP-29 group for project \"{project}\". \
95	                 Messages published by this session may be rejecte

User: [Tool result]: 40	            return ChannelGate::Degraded;
41	        };
42	        let mgmt_pubkey = mgmt_keys.public_key().to_hex();
43	
44	        let parent_admins: Vec<String> = if let Some(parent) = ctx.parent_hint {
45	            let grandparent = provider.with_store(|s| s.group_parent(parent).unwrap_or(None));
46	            let parent_ctx = ChannelCtx {
47	                channel: parent,
48	                expect_member: &mgmt_pubkey,
49	                parent_hint: grandparent.as_deref(),
50	            };
51	            let parent_gate = ensure_channel_ready_inner(provider, parent_ctx, depth + 1).await;
52	            if matches!(parent_gate, ChannelGate::Degraded) {
53	                eprintln!(
54	                    "[daemon] ensure_channel_ready: parent {:?} is degraded; aborting for {:?}",
55	                    parent, ctx.channel
56	                );
57	                return ChannelGate::Degraded;
58	            }
59	            provider.with_store(|s| {
60	                s.li

User: [Tool result]: 240	                                // Issue #6: rename THIS session's room to the new
241	                                // distilled title (kind:9002 edit-metadata, admin-
242	                                // signed by the provider's operator key; the relay
243	                                // re-emits kind:39000). Gated to per-session rooms so
244	                                // a shared task room is never renamed by one member.
245	                                // Only publishes on an actual title change (above), and
246	                                // only after the async session-room create is confirmed
247	                                // on the relay. If distill wins the race, the room-mint
248	                                // path catches up from session_state after create/lock.
249	                                let is_owned_room = st!(|s: &Store| {
250	                                    s.is_session_room(&p.project).unwrap_or(false)
251	                   

User: [Tool result]: chpwd:5: command not found: _iterm2_set_profile
=== is there a relay-admin-role check helper? ===
src/state/groups.rs:203:    pub fn is_group_member(&self, project: &str, pubkey: &str) -> Result<bool> {
src/state/groups.rs:340:            "DELETE FROM group_members WHERE project=?1 AND role = 'admin'",

=== fetch_group_state / fetch_group_roles defs ===
src/fabric/provider.rs:170:    pub async fn fetch_group_state(
src/fabric/provider.rs:231:    pub async fn fetch_group_roles(&self, group: &str) -> HashMap<String, String> {

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
