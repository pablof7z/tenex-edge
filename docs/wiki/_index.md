# Wiki Index

> Derived cache — do not hand-edit. Rebuilt by proactive-context after each capture.

Last updated: 2026-06-09

## general (1 guide)

| Slug | Title | Summary | Tags | Volatility | Verified | Topic |
|------|-------|---------|------|------------|----------|-------|
| [the-sessionstarthookspecificoutputwire-struct-contains-two-f](the-sessionstarthookspecificoutputwire-struct-contains-two-f.md) | the sessionstarthookspecificoutputwire struct contains two f | The SessionStartHookSpecificOutputWire struct contains two fields: suppressOutput and systemMessage. | capture | warm | 2026-06-09 | general |

## tenex-edge (13 guides)

| Slug | Title | Summary | Tags | Volatility | Verified | Topic |
|------|-------|---------|------|------------|----------|-------|
| [opencode-configuration](opencode-configuration.md) | OpenCode Configuration | The @opencode-ai/plugin dependency version must match the opencode binary version (1.16.2) in both ~/.config/opencode/package.json and ~/.opencode/package.json. | capture | warm | 2026-06-08 | tenex-edge |
| [tenex-edge](tenex-edge.md) | Tenex-Edge | tenex-edge grafts a shared Nostr-based nervous system onto agents that remain in their native hosts (Claude Code, Codex, OpenCode, mobile apps) rather than host | capture | warm | 2026-06-07 | tenex-edge |
| [tenex-edge-activity-distillation](tenex-edge-activity-distillation.md) | Tenex-Edge Activity Distillation | Activity distillation is LLM-based, with an optional heuristic gate to bound cost | capture | warm | 2026-06-08 | tenex-edge |
| [tenex-edge-awareness](tenex-edge-awareness.md) | Tenex-Edge Awareness | The awareness board's state model lives behind a transport interface, so that switching from local storage to network sync is a transport swap rather than a rew | capture | warm | 2026-06-08 | tenex-edge |
| [tenex-edge-configuration](tenex-edge-configuration.md) | Tenex-Edge Configuration | The project slug defaults to the current directory's git repository name (to unify worktrees), or the basename of $PWD if no git repo exists; it can be overridd | capture | warm | 2026-06-08 | tenex-edge |
| [tenex-edge-daemon](tenex-edge-daemon.md) | Tenex-Edge Daemon | tenex-edge uses a single machine-daemon that solely owns state.db, with all CLI calls and session engines acting as thin IPC clients over a Unix domain socket | capture | warm | 2026-06-09 | tenex-edge |
| [tenex-edge-data-persistence](tenex-edge-data-persistence.md) | Tenex-Edge Data Persistence | Local state is stored in SQLite | capture | warm | 2026-06-08 | tenex-edge |
| [tenex-edge-design-philosophy](tenex-edge-design-philosophy.md) | Tenex-Edge Design Philosophy | The design discussion operates at a higher, design-space level—what the thing is, what shape it should take, what is worth wanting, and where tensions and bets | capture | warm | 2026-06-07 | tenex-edge |
| [tenex-edge-host-adapter](tenex-edge-host-adapter.md) | Tenex-Edge Host Adapter | Host adapters must carry no identity logic or fabric logic and must never block the editor on the daemon being healthy (fail open) | capture | warm | 2026-06-09 | tenex-edge |
| [tenex-edge-identity](tenex-edge-identity.md) | Tenex-Edge Identity | Agent identity is a sovereign keypair, durable per-agent, anchored to a person | capture | warm | 2026-06-08 | tenex-edge |
| [tenex-edge-messaging](tenex-edge-messaging.md) | Tenex-Edge Messaging | Sending a message to another agent uses `tenex-edge send-message` accepting either an agent slug via `<agentSlug>@<projectSlug>` or a session ID via `--recipien | capture | warm | 2026-06-08 | tenex-edge |
| [tenex-edge-session-management](tenex-edge-session-management.md) | Tenex-Edge Session Management | MVP1 session start is invoked as `tenex-edge session-start --agent <agent-slug>`, which forks a background process and begins publishing a presence heartbeat | capture | warm | 2026-06-08 | tenex-edge |
| [tenex-edge-transport-codec](tenex-edge-transport-codec.md) | Tenex-Edge Transport Codec | Envelope encoding and decoding is modularized as a codec set providing per-event encode, decode, and subscribe operations | capture | warm | 2026-06-08 | tenex-edge |

