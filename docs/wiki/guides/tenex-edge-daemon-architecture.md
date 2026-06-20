---
title: tenex-edge Daemon Architecture
slug: tenex-edge-daemon-architecture
topic: tenex-edge
summary: The tenex-edged architecture is a single per-machine daemon that solely owns state.db and serves all CLI verbs and session engines over a Unix domain socket, el
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-07
updated: 2026-06-17
verified: 2026-06-07
compiled-from: conversation
sources:
  - session:8a3eb1b2-7bbf-4761-ad1a-411a0a1fa666
  - session:f3a730bf-9a3b-4952-b687-c93ade5fd7ec
  - session:162f9965-82ca-420b-aa24-99faa15cb59a
  - session:98f9939c-f42b-43dd-baba-d9a176d4b2d7
  - session:435ec383-d607-459b-a712-a00ed4decaa7
  - session:ab9998c4-6e65-410e-b298-122a2072171c
  - session:56f9fe89-5ff7-4e5b-b202-334cd7629d42
  - session:081ec521-c99b-42fb-9aa7-4a109519a62f
  - session:412e32c5-05f9-4e2a-86c6-e1c21e464553
  - session:52474db7-1e81-4011-a859-6343bfeae807
  - session:rollout-2026-06-09T15-35-48-019eac61-c1bb-7391-b237-7378101f099a
  - session:rollout-2026-06-12T11-18-49-019ebae9-8fa7-73f1-844d-bea23bfb0193
  - session:rollout-2026-06-16T12-40-33-019ecfcd-d47b-7992-998f-75432d8ac4cf
  - session:rollout-2026-06-16T14-11-38-019ed021-38a8-7472-bc5d-dc019a072086
  - session:rollout-2026-06-17T11-22-24-019ed4ac-a308-7250-b5ec-d95c8d18de3e
  - session:ses_13a5173feffeXR4Fi4UffHR88M
---

# tenex-edge Daemon Architecture

## Architecture Overview

The tenex-edged architecture is a single per-machine daemon that solely owns state.db and serves all CLI verbs and session engines over a Unix domain socket, eliminating multi-writer SQLite corruption; this single-owner design is the correct solution for multi-writer corruption, whereas WAL mode alone is only a stopgap (seatbelt, not redesign). The daemon uses a CQRS architecture where ALL reads come from a single unified local store (state.db) and fabric providers are write-side materializers that decode, ACL-admit, derive, and upsert canonical rows. The daemon spawns on first invocation via double-fork/setsid, binds a Unix domain socket under $TENEX_EDGE_HOME, uses flock for race-safe startup, and reclaims stale sockets. A newer binary meeting an older running daemon transparently tells it to exit and respawn (version-skew handshake), so upgrades are seamless. The daemon idle-exits when no sessions are alive, preventing it from lingering forever. The implementation uses Rust and nostr-sdk (not NMP), because NMP is a full cross-platform app kernel unsuitable for a headless CLI daemon; NMP remains a future codec swap-in option behind the transport seam. Only one Store::open call exists in the entire codebase (daemon/server.rs), proving the single-writer guarantee by construction. The channel adapter must NOT become a second state.db writer; it is a thin stream-consumer of the engine, never independently writing state.db. Debug-vs-release binaries or differing TENEX_EDGE_HOME paths can evade the flock/socket lock and spawn parallel daemons, which is a robustness gap that should be hardened. The daemon's planned subscribe --json streaming verb is the seam that will feed all three host adapters (Claude channels, Codex app-server, OpenCode prompt_async). The daemon design doc is captured in docs/daemon-design.md as the review checkpoint for the RPC surface, lifecycle/ownership, socket/lock/stale-reclaim/version-handshake mechanics, and engine relocation. The production daemon was cutover to the refactored binary: old daemon stopped, binary swapped, new daemon started; who works; propose and threads are live verbs; the real state.db was migrated and backfilled (40 projects, 15 members). Incoming relay events are deduplicated by event ID in handle_incoming using a 512-slot ring buffer in DaemonState.seen_events. The Config struct includes user_nsec: Option<String>, deserialized from the JSON field name 'userNsec' in ~/.tenex/config.json. The daemon must be reinstalled via 'just install' after code changes that add new RPCs, because the running daemon binary is the installed one at ~/.local/bin/tenex-edge and adding RPC methods is not detected as a protocol version change. NIP-29 groups are auto-created by the daemon via ensure_group_and_membership (called in rpc_session_start and reconcile_sessions), which publishes kind:9002 create-group then locks the group closed+public before any session presence events flow. The accept loop starts immediately after socket binding and local store initialization, without waiting for relay connection; relay connection and NIP-42 AUTH warmup fetch execute as a background task after the accept loop is already running. DaemonState holds transport and provider as Mutex<Option<Arc<...>>> fields, populated later when the relay connects; a relay_ready Notify fires once when the relay connects, allowing relay-dependent handlers to wait via async transport() and provider() methods. who and tmux_* RPCs read exclusively from SQLite and render immediately from last-known state without touching transport or provider. Relay-dependent operations (publish, subscribe, session start) wait for relay readiness via the async transport()/provider() methods. The `call()` method in `src/daemon/client.rs` loops past `item` progress frames before returning, so it no longer fails when `session_start` emits `item` frames before the terminal `ok`. The integration test harness in `tests/daemon_integration/harness.rs` removes both `TENEX_EDGE_AGENT` and `TENEX_EDGE_AGENT_FALLBACK` from the subprocess environment, preventing the live shell's `developer` slug from leaking into tests. Daemon session_start cancels older session rows for the same agent/project/host with the same watched host PID before spawning the new task, preserving true parallel processes from separate terminals. Daemon session_start is idempotent so reasserting a session on UserPromptSubmit does not spawn duplicate tasks. UserPromptSubmit defensively reasserts the session so an already-open Claude session recovers after a daemon restart. The daemon marks local outbound events as `published` in its SQLite database optimistically, regardless of whether the relay actually accepted the event. During session_start initialization, the daemon streams progress frames to the hook, which prints them to stderr while preserving stdout for harness protocol data. The NIP-29 provider reports fetch, create, lock, admin grant, and agent member steps with timing during initialization. Setting TENEX_EDGE_INIT_PROGRESS=0 suppresses the initialization progress lines. The daemon protocol version was bumped so old clients do not misread the new progress frames. Daemon clients determine readiness by completing a real hello/welcome handshake (and a ping probe for sync commands), not merely by detecting that the Unix socket file exists. All daemon-backed commands print a short stderr line when waiting for daemon readiness. The daemon runs session reconciliation in the background after it starts accepting RPCs, not before. Daemon handshake connect and read operations have timeouts so clients never wait silently forever. There is a CLI ↔ daemon dependency inversion where daemon/server.rs imports crate::cli::load_who_snapshot and crate::cli::assemble_turn_start_context, meaning the daemon depends on CLI code. Server extraction is partial: 9 submodules exist, but server.rs still holds 2692 lines of unextracted handlers.

<!-- citations: [^52474-1] [^52474-2] [^98f99-1] [^98f99-2] [^98f99-6] [^8a3eb-11] [^f3a73-3] [^162f9-1] [^ab999-2] [^56f9f-1] [^412e3-1] [^rollo-26] [^rollo-41] [^rollo-76] [^rollo-114] [^ses_1-4] -->
## Transport & Codec

Envelope shapes are decoupled from business logic via a modularized codec so that a different transport can be added as a shape adapter with no issue. Heartbeat activity is part of the codec since it varies for different codecs. <!-- [^f3a73-4] -->

## Local State

Local state is SQLite (per-session processes, with LMDB single-writer compatibility as a known NMP-integration consideration). The `tenex-edge` messaging state and local database reside under `~/.tenex/edge`, not in the proactive-context logs. Local daemon state (sessions, turn state, inbox delivery, seen mentions, tmux endpoints, and local liveness) is daemon-owned and separate from relay-owned project/group state.

<!-- citations: [^f3a73-5] [^rollo-53] [^rollo-77] -->
## CLI

tenex-edge tail streams all messages with optional project-scoping via --project. <!-- [^f3a73-6] -->


tenex-edge project list fetches all kind:39000 events from the relay (no author filter, since the relay authors kind:39000 events in NIP-29), caches them locally, and renders them as a left-aligned table. <!-- [^98f99-4] -->

tenex-edge project edit --description <desc> publishes a kind:9002 (NIP-29 edit-metadata) event signed by userNsec, which the relay validates and re-publishes as kind:39000; the local cache is updated optimistically. <!-- [^98f99-5] -->

The `who` command groups agents by project and shows only the project name and metadata (one line per project) instead of listing each agent individually in the 'other projects' section. <!-- [^435ec-2] -->
## macOS Deployment Note

When reinstalling the tenex-edge binary over ~/.local/bin, macOS requires rm + cp + xattr -cr + codesign --force --sign - to avoid SIGKILL on the fork/re-exec path. The tenex-edge binary is symlinked to ~/.local/bin/tenex-edge on the remote machine, which is the fallback path server.ts uses when TENEX_EDGE_BIN isn't set.

<!-- citations: [^f3a73-7] [^081ec-1] -->
## WAL Mode Stopgap

WAL mode (journal_mode=WAL + busy_timeout=5000 + synchronous=NORMAL) is enabled as an immediate stopgap to reduce corruption risk while the daemon is built. <!-- [^162f9-2] -->

## Event Classification

Kind:1 events require an agent tag to be classified as a Mention; events with a p-tag but no agent tag are classified as Activity, preventing user OPs from being routed into the agent's inbox. <!-- [^98f99-3] -->
