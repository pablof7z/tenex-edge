# Wiki Index

> Derived cache — do not hand-edit. Rebuilt by proactive-context after each capture.

Last updated: 2026-06-19

## agent-configuration (1 guide)

| Slug | Title | Summary | Tags | Volatility | Verified | Topic |
|------|-------|---------|------|------------|----------|-------|
| [tenex-system-prompt](guides/tenex-system-prompt.md) | TENEX System Prompt | TENEX's system prompt is deterministic and serves as a cache anchor; all per-turn volatile material (reminders, RAG hits, todos) goes into the projected user me | capture | warm | 2026-06-07 | agent-configuration |

## android-app (2 guides)

| Slug | Title | Summary | Tags | Volatility | Verified | Topic |
|------|-------|---------|------|------------|----------|-------|
| [android-key-pair-generation](guides/android-key-pair-generation.md) | Android Generate Key Pair Feature | The Android 'Generate Key Pair' feature (IdentityActions.generate, first-launch auto-generate, stale-rev push guard, onSnapshotPull wiring) is preserved on back | capture | warm | 2026-06-13 | android-app |
| [android-release-builds](guides/android-release-builds.md) | Android Release Builds | Android release builds produce an unsigned APK (no signing config in the build type), so only debug APKs are directly installable. | capture | warm | 2026-06-13 | android-app |

## architecture (7 guides)

| Slug | Title | Summary | Tags | Volatility | Verified | Topic |
|------|-------|---------|------|------------|----------|-------|
| [tenex-cli-acl-removal](guides/tenex-cli-acl-removal.md) | TENEX CLI ACL Removal | ACL command, routing, storage, and tail variants remain removed | capture | warm | 2026-06-14 | architecture |
| [tenex-cli-interface](guides/tenex-cli-interface.md) | TENEX CLI Interface | The message sending and reading commands are unified under the `inbox` command (`inbox send`, `inbox reply`, `inbox`), replacing the previous `send-message` com | capture | warm | 2026-06-09 | architecture |
| [tenex-edge-daemon-rebuild](guides/tenex-edge-daemon-rebuild.md) | tenex-edge Daemon Rebuild and Restart | After pushing source changes to origin/master, the locally installed daemon binary is rebuilt and restarted so live hooks/RPCs run the new code | capture | warm | 2026-06-14 | architecture |
| [tenex-process-isolation](guides/tenex-process-isolation.md) | TENEX Process Isolation | TENEX uses process-per-project isolation rather than running all projects in a single process as the TypeScript predecessor did. | capture | warm | 2026-06-07 | architecture |
| [tenex-role-separation](guides/tenex-role-separation.md) | TENEX Role Separation | TENEX enforces three strictly separated roles: Subscribe (relay), Orchestrate (runtime, never calls LLMs), and Execute (tenex-agent, one-shot per turn, never op | capture | warm | 2026-06-07 | architecture |
| [tenex-rust-migration](guides/tenex-rust-migration.md) | TENEX Rust Migration | TENEX has been fully migrated from TypeScript/Bun to 100% Rust, with the TypeScript runtime removed and a read-only reference preserved at a specific path | capture | warm | 2026-06-07 | architecture |
| [tenex-self-update](guides/tenex-self-update.md) | TENEX Self-Update Mechanism | On macOS, when reinstalling a binary that gets SIGKILLed due to a stale code-signature, the fix is to remove the old binary and recopy it rather than overwritin | capture | warm | 2026-06-09 | architecture |

## ci-cd (5 guides)

| Slug | Title | Summary | Tags | Volatility | Verified | Topic |
|------|-------|---------|------|------------|----------|-------|
| [branch-protection-rules](guides/branch-protection-rules.md) | Branch Protection Rules | Branch protection on `main` requires six cloud-based status checks to pass before a PR can merge: Rust workspace build gate, Swift bridge codegen drift gate, An | capture | warm | 2026-06-13 | ci-cd |
| [ci-self-hosted-runner](guides/ci-self-hosted-runner.md) | Self-Hosted CI Runner Saturation | The repository uses a single self-hosted GitHub Actions runner (Pablos-MacBook-Pro-3-podcast) that processes one job at a time, causing saturation when multiple | capture | warm | 2026-06-13 | ci-cd |
| [ci-test-flake-fixes](guides/ci-test-flake-fixes.md) | CI Test Flake Fixes | The run_tests.sh script includes a pre-build chmod -R u+w on the secp256k1 DerivedData plugin cache, self-healing the read-only flake caused by interrupted buil | capture | warm | 2026-06-13 | ci-cd |
| [ci-testflight-deploy](guides/ci-testflight-deploy.md) | TestFlight Deploy Workflow | The TestFlight deploy workflow triggers only when the iOS app version number (CFBundleShortVersionString in App/Resources/Info.plist) increases, or on manual di | capture | warm | 2026-06-13 | ci-cd |
| [ci-workflow-concurrency](guides/ci-workflow-concurrency.md) | CI Workflow Concurrency | The Test workflow uses per-branch/per-PR concurrency with cancel-in-progress: true, so a newer push cancels the older in-flight run for that ref instead of stac | capture | warm | 2026-06-13 | ci-cd |

## data-persistence (1 guide)

| Slug | Title | Summary | Tags | Volatility | Verified | Topic |
|------|-------|---------|------|------------|----------|-------|
| [tenex-edge-state-db-persistence](guides/tenex-edge-state-db-persistence.md) | tenex-edge state.db Persistence Architecture | SQLite multi-writer corruption is a risk because approximately 16 per-session engines plus CLI invocations all share one state.db | capture | warm | 2026-06-09 | data-persistence |

## documentation (2 guides)

| Slug | Title | Summary | Tags | Volatility | Verified | Topic |
|------|-------|---------|------|------------|----------|-------|
| [codex-refactor-review](guides/codex-refactor-review.md) | Codex Refactor Review | The review of the a41bd570 refactor is written to docs/review-codex.md with findings ordered by severity and line numbers referenced to the a41bd570 tree. | capture | warm | 2026-06-16 | documentation |
| [docs-wiki-directory](guides/docs-wiki-directory.md) | Docs Wiki Directory | The docs/wiki directory (~400â460 files) is submitted as PR #434 for review on whether the generated/authored docs belong in the repo | capture | warm | 2026-06-13 | documentation |

## engineering-standards (2 guides)

| Slug | Title | Summary | Tags | Volatility | Verified | Topic |
|------|-------|---------|------|------------|----------|-------|
| [agents-md-contributor-policy](guides/agents-md-contributor-policy.md) | AGENTS.md Contributor Policy | AGENTS.md must include a contributor policy that sets a soft limit of 300 LOC and a hard limit of 500 LOC for code files | capture | warm | 2026-06-10 | engineering-standards |
| [tenex-engineering-standards](guides/tenex-engineering-standards.md) | TENEX Engineering Standards | TENEX requires no temporary solutions, no backward-compatibility shims, no TODO/FIXME/workaround comments, and no wrapper classes â every change must be the r | capture | warm | 2026-06-07 | engineering-standards |

## git-workflow (2 guides)

| Slug | Title | Summary | Tags | Volatility | Verified | Topic |
|------|-------|---------|------|------------|----------|-------|
| [git-branch-merge-strategy](guides/git-branch-merge-strategy.md) | Git Branch Merge Strategy | Diverged branches must be resolved via proper merge, not force-push, to preserve all existing work on both sides | capture | warm | 2026-06-13 | git-workflow |
| [merged-pr-review](guides/merged-pr-review.md) | Merged PR Review | PR #432 (iOS launch library hydration flash fix by the codex agent) was reviewed and merged. | capture | warm | 2026-06-13 | git-workflow |

## ios-app (3 guides)

| Slug | Title | Summary | Tags | Volatility | Verified | Topic |
|------|-------|---------|------|------------|----------|-------|
| [ios-app-state-store](guides/ios-app-state-store.md) | iOS AppStateStore Dictionary Safety | AppStateStore.applyDownloadOverlay uses Dictionary(_:uniquingKeysWith:) instead of Dictionary(uniqueKeysWithValues:) for its active-downloads overlay, preventin | capture | warm | 2026-06-13 | ios-app |
| [ios-swift-concurrency-fixes](guides/ios-swift-concurrency-fixes.md) | iOS Swift Concurrency Fixes | The test method calling `nostrConversationFromDTO` was annotated `@MainActor` to fix a compile error from calling a main-actor-isolated static method in a synch | capture | warm | 2026-06-13 | ios-app |
| [ios-ui-test-selectors](guides/ios-ui-test-selectors.md) | iOS UI Test Selectors | The UITest agent button selector is 'agent.open' (replacing the stale 'sparkles' image name after the UI redesign) | capture | warm | 2026-06-13 | ios-app |

## opencode-integration (1 guide)

| Slug | Title | Summary | Tags | Volatility | Verified | Topic |
|------|-------|---------|------|------------|----------|-------|
| [opencode-plugin-setup](guides/opencode-plugin-setup.md) | OpenCode Plugin Setup | The opencode plugin dependency @opencode-ai/plugin must match the installed opencode version (1.16.2) to prevent the plugin from failing to load and opencode fr | capture | warm | 2026-06-08 | opencode-integration |

## security (1 guide)

| Slug | Title | Summary | Tags | Volatility | Verified | Topic |
|------|-------|---------|------|------------|----------|-------|
| [tenex-edge-key-security](guides/tenex-edge-key-security.md) | tenex-edge Key Security Incident | The owner key (09d48a1aâ¦) was leaked to Google during blind adb sign-in automation (typed into the emulator's search box) and must be rotated immediately | capture | warm | 2026-06-09 | security |

## social-media-agent (1 guide)

| Slug | Title | Summary | Tags | Volatility | Verified | Topic |
|------|-------|---------|------|------------|----------|-------|
| [social-media-writer-agent](guides/social-media-writer-agent.md) | Social Media Writer Agent | The social media writer agent drafts and publishes tweets on behalf of the user, using work done in the cut-tracker repository as the source of content rather t | capture | warm | 2026-06-17 | social-media-agent |

## syncthing-directory-sync (1 guide)

| Slug | Title | Summary | Tags | Volatility | Verified | Topic |
|------|-------|---------|------|------------|----------|-------|
| [syncthing-directory-sync](guides/syncthing-directory-sync.md) | Syncthing Directory Sync | The Syncthing directory syncs only markdown (.md) documents and excludes all other file types (no git, code, or build artifacts) | capture | warm | 2026-06-09 | syncthing-directory-sync |

## tenex (1 guide)

| Slug | Title | Summary | Tags | Volatility | Verified | Topic |
|------|-------|---------|------|------------|----------|-------|
| [tenex-overview](guides/tenex-overview.md) | TENEX Overview | TENEX is a multi-agent AI coordination system for software development built on the Nostr protocol, treating AI context as the primary artifact, with agent-to-a | capture | warm | 2026-06-07 | tenex |

## tenex-edge (53 guides)

| Slug | Title | Summary | Tags | Volatility | Verified | Topic |
|------|-------|---------|------|------------|----------|-------|
| [tenex-edge-advisory-locks](guides/tenex-edge-advisory-locks.md) | tenex-edge Advisory Locks | The advisory lock algorithm uses a mandatory settle window (~1500ms relay propagation RTT), TTL-based leases (~120s), and deterministic tie-breaking by (created | capture | warm | 2026-06-07 | tenex-edge |
| [tenex-edge-agent-identity-store](guides/tenex-edge-agent-identity-store.md) | tenex-edge Agent Identity Store | The spawnable agents list is sourced from the tenex-edge agent identity store (~/.tenex/edge/agents/ JSON files), not from PATH `which` checks for binaries or f | capture | warm | 2026-06-14 | tenex-edge |
| [tenex-edge-agent-society-vision](guides/tenex-edge-agent-society-vision.md) | tenex-edge Agent Society Vision | The higher-level abstraction revealed by the todo-app/podcast-app example is that the operating system is being turned inside out: apps become citizens with rol | capture | warm | 2026-06-07 | tenex-edge |
| [tenex-edge-awareness-output](guides/tenex-edge-awareness-output.md) | tenex-edge Awareness Output | PostToolUse awareness output must be delta-gated, project-scoped, and self-excluded â only emitting sibling session changes in the current project since the l | capture | warm | 2026-06-15 | tenex-edge |
| [tenex-edge-beachhead-user](guides/tenex-edge-beachhead-user.md) | tenex-edge Beachhead User | The beachhead user for tenex-edge is the solo agent-power-user running two or more agents (specifically Claude Code + Codex or Cursor) on the same repo, because | capture | warm | 2026-06-07 | tenex-edge |
| [tenex-edge-chat-commands](guides/tenex-edge-chat-commands.md) | tenex-edge Chat Commands | The CLI supports `tenex-edge chat write` to send chat messages in the NIP-29 codec and `tenex-edge chat read` with optional `--since <relative-time>`, `--limit` | capture | warm | 2026-06-16 | tenex-edge |
| [tenex-edge-codex-hook-integration](guides/tenex-edge-codex-hook-integration.md) | tenex-edge Codex Hook Integration | The claude/codex binary hooks use the tenex-edge binary found in $PATH rather than a hardcoded absolute path. | capture | warm | 2026-06-19 | tenex-edge |
| [tenex-edge-core-thesis](guides/tenex-edge-core-thesis.md) | tenex-edge Core Thesis | The core thesis of tenex-edge is the inversion of TENEX: instead of hosting agents, it connects agents that already run in their native homes, grafting a shared | capture | warm | 2026-06-07 | tenex-edge |
| [tenex-edge-cross-agent-collaboration](guides/tenex-edge-cross-agent-collaboration.md) | tenex-edge Cross-Agent Collaboration | tenex-edge enables cross-agent collaboration where, for example, a Codex session and a Claude Code session encountering the same bug can coordinate on fixing it | capture | warm | 2026-06-07 | tenex-edge |
| [tenex-edge-cross-person-collaboration](guides/tenex-edge-cross-person-collaboration.md) | tenex-edge Cross-Person Collaboration | Tenex Edge enables cross-person collaboration where one user's agent can query another person's agent about how they handled something in one of their projects | capture | warm | 2026-06-07 | tenex-edge |
| [tenex-edge-daemon-architecture](guides/tenex-edge-daemon-architecture.md) | tenex-edge Daemon Architecture | The tenex-edged architecture is a single per-machine daemon that solely owns state.db and serves all CLI verbs and session engines over a Unix domain socket, el | capture | warm | 2026-06-07 | tenex-edge |
| [tenex-edge-debug-hook-tail](guides/tenex-edge-debug-hook-tail.md) | tenex-edge Debug Hook-Tail Command | The `tenex-edge debug hook-tail` command shows what was or will be injected in a session and by which hook, as well as what tenex-edge commands each session is | capture | warm | 2026-06-17 | tenex-edge |
| [tenex-edge-design-altitude](guides/tenex-edge-design-altitude.md) | tenex-edge Design Altitude | The design conversation operates at a design-space levelâfocusing on what the thing is, what shape it should take, what's worth wanting, and where the tension | capture | warm | 2026-06-07 | tenex-edge |
| [tenex-edge-distillation](guides/tenex-edge-distillation.md) | tenex-edge Distillation | Distillation is automatic (auto-distill), not manual; agents are not relied on to call it themselves | capture | warm | 2026-06-08 | tenex-edge |
| [tenex-edge-durable-value](guides/tenex-edge-durable-value.md) | tenex-edge Durable Value | The two durable, defensible values that survive even if a host vendor ships native coordination tomorrow are: (1) vendor-independent agent identity where reputa | capture | warm | 2026-06-07 | tenex-edge |
| [tenex-edge-existing-nostr-fabric](guides/tenex-edge-existing-nostr-fabric.md) | tenex-edge Existing Nostr Fabric | A working Nostr agent fabric already exists on this machine, as demonstrated by the podcast-player app (Pod0) | capture | warm | 2026-06-07 | tenex-edge |
| [tenex-edge-fabric-architecture](guides/tenex-edge-fabric-architecture.md) | tenex-edge Fabric Architecture | The domain speaks in two concern-planes: Project-State (open_project, roster, presence, status, project_meta) and Communications (send, inbox, threads, thread_m | capture | warm | 2026-06-09 | tenex-edge |
| [tenex-edge-host-integration-mcp-hooks](guides/tenex-edge-host-integration-mcp-hooks.md) | tenex-edge Host Integration (MCP & Hooks) | MCP is the lowest-common-denominator substrate that every supported host speaks; Claude Code hooks are the premium tier adding blocking capability and lifecycle | capture | warm | 2026-06-07 | tenex-edge |
| [tenex-edge-identity-and-presence](guides/tenex-edge-identity-and-presence.md) | tenex-edge Identity and Presence | tenex-edge owns identity and awareness as its own substrate, independent of any host adapter like pc | capture | warm | 2026-06-07 | tenex-edge |
| [tenex-edge-inbox-envelope-format](guides/tenex-edge-inbox-envelope-format.md) | tenex-edge Inbox Envelope Format | Inbox messages are displayed in an email-like envelope format with From, Date, Branch, ID, a '--' separator, and the message body | capture | warm | 2026-06-12 | tenex-edge |
| [tenex-edge-indexer-relay](guides/tenex-edge-indexer-relay.md) | tenex-edge Indexer Relay | tenex-edge publishes kind:0 events to a configurable indexer relay, defaulting to wss://purplepag.es | capture | warm | 2026-06-14 | tenex-edge |
| [tenex-edge-install-subcommand](guides/tenex-edge-install-subcommand.md) | tenex-edge Install Subcommand | tenex-edge provides an install subcommand that sets up hooks in the different harnesses, mirroring the proactive-context install command's interface. | capture | warm | 2026-06-16 | tenex-edge |
| [tenex-edge-mvp-scope](guides/tenex-edge-mvp-scope.md) | tenex-edge MVP Scope | The MVP for tenex-edge is advisory lock (collision avoidance) plus shared-bug deduplication, strictly solo, across Claude Code and Codex on one machine â no c | capture | warm | 2026-06-07 | tenex-edge |
| [tenex-edge-nip29-materializer](guides/tenex-edge-nip29-materializer.md) | tenex-edge NIP-29 Materializer | NIP-29 39000/39002 events hydrate state exclusively through Nip29Materializer into store-level materializer methods | capture | warm | 2026-06-16 | tenex-edge |
| [tenex-edge-nostr-guarantees](guides/tenex-edge-nostr-guarantees.md) | tenex-edge Nostr Guarantees | Nostr is an AP (available, partition-tolerant) system; relays are an eventually-consistent gossip bus with no compare-and-swap, no broadcast guarantee, and no e | capture | warm | 2026-06-07 | tenex-edge |
| [tenex-edge-phasing-and-vision](guides/tenex-edge-phasing-and-vision.md) | tenex-edge Phasing and Vision | There are two categorically different products here: (A) a nervous system for your own fleet (single-player, your keys, your machines, safe, immediately valuabl | capture | warm | 2026-06-07 | tenex-edge |
| [tenex-edge-platform-bet](guides/tenex-edge-platform-bet.md) | tenex-edge Platform Bet | The platform bet for tenex-edge is a thin open adapter on Nostr where the protocol is the product; any host emits and consumes signed events, and tenex-edge own | capture | warm | 2026-06-07 | tenex-edge |
| [tenex-edge-product-spec](guides/tenex-edge-product-spec.md) | tenex-edge Product Spec | The product spec has been written as 13 chapters in docs/product-spec/, all at design-space altitude with no mechanics, preserving live tensions and disagreemen | capture | warm | 2026-06-07 | tenex-edge |
| [tenex-edge-project-add](guides/tenex-edge-project-add.md) | tenex-edge Project Add Command | Running `tenex-edge project add` with no arguments resolves the project from the current directory and opens an interactive checkbox selector over locally-avail | capture | warm | 2026-06-17 | tenex-edge |
| [tenex-edge-project-slug-resolution](guides/tenex-edge-project-slug-resolution.md) | tenex-edge Project Slug Resolution | Project slug resolution uses `git rev-parse --git-common-dir` (instead of `--show-toplevel`) to extract the shared repository directory's basename, ensuring git | capture | warm | 2026-06-09 | tenex-edge |
| [tenex-edge-propose](guides/tenex-edge-propose.md) | tenex-edge Propose Command | tenex-edge propose publishes a kind:30023 event signed by the agent's identity | capture | warm | 2026-06-09 | tenex-edge |
| [tenex-edge-provider-seam](guides/tenex-edge-provider-seam.md) | tenex-edge Provider Seam | The full inventory of wire-shape leaks must be moved behind the provider layer | capture | warm | 2026-06-12 | tenex-edge |
| [tenex-edge-recipient-resolution](guides/tenex-edge-recipient-resolution.md) | tenex-edge Recipient Resolution | When the `--to` argument contains an `@`, `resolve_recipient` parses it as a slug and project qualifier, then calls `store.resolve_agent_pubkey(slug, Some(proj) | capture | warm | 2026-06-16 | tenex-edge |
| [tenex-edge-red-team-analysis](guides/tenex-edge-red-team-analysis.md) | tenex-edge Red Team Analysis | The red-team analysis identifies the most kill-likely risk as the load-bearing superpower (advisory locking) being mostly redundant with git â collisions betw | capture | warm | 2026-06-07 | tenex-edge |
| [tenex-edge-relay-strategy](guides/tenex-edge-relay-strategy.md) | tenex-edge Relay Strategy | The recommended relay strategy is a personal relay per operator (single propagation domain, small settle window), supplemented by shared collaboration relays fo | capture | warm | 2026-06-07 | tenex-edge |
| [tenex-edge-scope-exclusions](guides/tenex-edge-scope-exclusions.md) | tenex-edge Scope Exclusions | Tenex-edge must not build a hosted central server, its own agent or agent host, a UI-first dashboard or mission control, open-ended agent chat, or require mutua | capture | warm | 2026-06-07 | tenex-edge |
| [tenex-edge-secret-scrubbing](guides/tenex-edge-secret-scrubbing.md) | tenex-edge Secret Scrubbing | Secret scrubbing is the mechanism used to avoid leaking secrets in published events that contain user/agent data. | capture | warm | 2026-06-12 | tenex-edge |
| [tenex-edge-session-forensics-logging](guides/tenex-edge-session-forensics-logging.md) | tenex-edge Session Forensics Logging | JSONL logs are written per-session under ~/.tenex/edge/sessions/<session-id>/ with separate hook-calls.jsonl and command-calls.jsonl files, instead of a single | capture | warm | 2026-06-17 | tenex-edge |
| [tenex-edge-session-identity](guides/tenex-edge-session-identity.md) | tenex-edge Session Identity | Session IDs use a hash-based short code (`session_short_code`) rather than UUID prefix truncation (`short_id`), rendering as a canonical 6-character short code | capture | warm | 2026-06-09 | tenex-edge |
| [tenex-edge-session-label](guides/tenex-edge-session-label.md) | tenex-edge Session Label | The session label is split into a persistent title (`Status.text`) and a separate `active`/`idle` boolean, rather than using a single status field that is wiped | capture | warm | 2026-06-14 | tenex-edge |
| [tenex-edge-session-resume](guides/tenex-edge-session-resume.md) | tenex-edge Session Resume | Session resume is local-only: resuming a remote machine's session requires SSH and is out of scope | capture | warm | 2026-06-14 | tenex-edge |
| [tenex-edge-statusline](guides/tenex-edge-statusline.md) | tenex-edge Statusline | The statusline format is: `claude@host [session-id] â¬¡{member_count} â{session_count} {activity} {distill_error} {inbox_segment}`, where â¬¡ is count of NIP- | capture | warm | 2026-06-12 | tenex-edge |
| [tenex-edge-strategic-posture](guides/tenex-edge-strategic-posture.md) | tenex-edge Strategic Posture | The Tenex Edge distribution mechanism is strictly an adapter that external systems depend on, never the reverseâthe word 'plugin' must not leak into tenex-edg | capture | warm | 2026-06-07 | tenex-edge |
| [tenex-edge-tail-v2](guides/tenex-edge-tail-v2.md) | tenex-edge Tail v2 Stream | Tail v2 was implemented as a structured TailEvent stream with 10 variants, join/leave derivation from heartbeat suppression, 4 tiers, backfill, --json output, a | capture | warm | 2026-06-09 | tenex-edge |
| [tenex-edge-technical-debt](guides/tenex-edge-technical-debt.md) | tenex-edge Technical Debt | Technical debt identification is scoped to identification only, with no changes to be made | capture | warm | 2026-06-14 | tenex-edge |
| [tenex-edge-thread-protocol](guides/tenex-edge-thread-protocol.md) | tenex-edge Thread Protocol | When an agent finishes producing text (stop hook), it must publish a kind:1 TurnReply with its own key, e-tagging the root event and the prompt that triggered t | capture | warm | 2026-06-10 | tenex-edge |
| [tenex-edge-tmux-sidebar](guides/tenex-edge-tmux-sidebar.md) | tenex-edge Tmux Sidebar | The `[no tmux]` tag has been removed from non-attachable live rows, and help text updated to `[âµ] attach/spawn` | capture | warm | 2026-06-15 | tenex-edge |
| [tenex-edge-trust-model](guides/tenex-edge-trust-model.md) | tenex-edge Trust Model | The trust model authorizes events by signer pubkey plus NIP-29 relay-authoritative group membership; NIP-29 relay state is the single source of truth for projec | capture | warm | 2026-06-07 | tenex-edge |
| [tenex-edge-tui-forensics-reader](guides/tenex-edge-tui-forensics-reader.md) | tenex-edge TUI Forensics Reader | Log files are read using a tail-read helper that seeks to file_size minus 2MB and skips the partial first line, capping reads at ~2MB per refresh cycle | capture | warm | 2026-06-17 | tenex-edge |
| [tenex-edge-wait-for-mention](guides/tenex-edge-wait-for-mention.md) | tenex-edge Wait-for-Mention Command | The `tenex-edge wait-for-mention` command has been removed from the codebase | capture | warm | 2026-06-09 | tenex-edge |
| [tenex-edge-who-command-implementation](guides/tenex-edge-who-command-implementation.md) | tenex-edge `who` Command Implementation | The `src/cli/who.rs`, `src/cli/who/render.rs`, and `src/cli/who/tests.rs` files (821 lines total) are dead code â they are never declared with `mod who;` in ` | capture | warm | 2026-06-14 | tenex-edge |
| [tenex-off-tui-client](guides/tenex-off-tui-client.md) | tenex-off TUI Client | The tenex-off codex/ratatui-tui-client worktree (374 lines of TUI markdown table rendering) was committed and merged into master via --no-ff. | capture | warm | 2026-06-13 | tenex-edge |
| [tmux-session-management](guides/tmux-session-management.md) | Tmux Session Management | Inside tmux, switching to a pane in another window uses `switch-client -t <pane_id>` (not `select-pane`, which only works within the current window) | capture | warm | 2026-06-14 | tenex-edge |

## Research Records (2 records)

| Record | Date | Finding | Agent |
|--------|------|---------|-------|
| [2026-06-18-1-experiment-comparing-session-title-generation-with](research/2026-06-18-1-experiment-comparing-session-title-generation-with.md) | 2026-06-18 | Experiment comparing session title generation with vs without tool_use in distillation context, finding that stripping tool_use makes titles anchor on user intent instead of agent actions | main |
| [2026-06-18-1-verification-and-severity-triaging-of-codex](research/2026-06-18-1-verification-and-severity-triaging-of-codex.md) | 2026-06-18 | Verification and severity-triaging of codex review findings on session-state rearchitecture, confirming critical harness-id vs canonical-id alias mismatch and heartbeat expiration bugs | main |

## Episode Cards (97 cards)

| Card | Date | Title | Salience | Status |
|------|------|-------|----------|--------|
| [2026-06-07-1-product-center-of-gravity-shifted-from](episodes/2026-06-07-1-product-center-of-gravity-shifted-from.md) | 2026-06-07 | Product center-of-gravity shifted from coordination to agent citizenship | reversal | active |
| [2026-06-07-2-not-greenfield-tenex-edge-is-the](episodes/2026-06-07-2-not-greenfield-tenex-edge-is-the.md) | 2026-06-07 | Not greenfield — tenex-edge is the on-ramp to an existing fabric | root-cause | active |
| [2026-06-08-1-agent-status-distilled-from-conversation-transcript](episodes/2026-06-08-1-agent-status-distilled-from-conversation-transcript.md) | 2026-06-08 | Agent status distilled from conversation transcript, not tool names | product | superseded |
| [2026-06-08-1-opencode-plugin-version-must-match-binary](episodes/2026-06-08-1-opencode-plugin-version-must-match-binary.md) | 2026-06-08 | opencode plugin version must match binary version | root-cause | active |
| [2026-06-08-2-distillation-config-moved-from-python-script](episodes/2026-06-08-2-distillation-config-moved-from-python-script.md) | 2026-06-08 | Distillation config moved from Python script + env var to ~/.tenex/ with native rig | architecture | active |
| [2026-06-09-1-abandon-python-wrapper-for-direct-binary](episodes/2026-06-09-1-abandon-python-wrapper-for-direct-binary.md) | 2026-06-09 | Abandon Python wrapper for direct binary hook invocation | reversal | active |
| [2026-06-09-1-activity-distillation-replaced-tool-driven-turn](episodes/2026-06-09-1-activity-distillation-replaced-tool-driven-turn.md) | 2026-06-09 | Activity distillation replaced: tool-driven → turn-driven, transcript-only | reversal | superseded |
| [2026-06-09-1-agent-identifier-in-who-output-slug](episodes/2026-06-09-1-agent-identifier-in-who-output-slug.md) | 2026-06-09 | Agent identifier in who output: slug@project → slug@hostname (slugified) | product | active |
| [2026-06-09-1-agent-mention-reactivity-via-wait-for](episodes/2026-06-09-1-agent-mention-reactivity-via-wait-for.md) | 2026-06-09 | Agent mention reactivity via wait-for-mention command | product | superseded |
| [2026-06-09-1-channel-adapter-replaces-wait-for-mention](episodes/2026-06-09-1-channel-adapter-replaces-wait-for-mention.md) | 2026-06-09 | Channel adapter replaces wait-for-mention for async work injection | reversal | active |
| [2026-06-09-1-cli-lifecycle-verbs-removed-hook-becomes](episodes/2026-06-09-1-cli-lifecycle-verbs-removed-hook-becomes.md) | 2026-06-09 | CLI lifecycle verbs removed; hook becomes sole harness entry point | architecture | active |
| [2026-06-09-1-codex-sessionstart-hook-must-emit-json](episodes/2026-06-09-1-codex-sessionstart-hook-must-emit-json.md) | 2026-06-09 | Codex SessionStart hook must emit JSON, not plain text | root-cause | active |
| [2026-06-09-1-daemon-actively-owns-nip-29-group](episodes/2026-06-09-1-daemon-actively-owns-nip-29-group.md) | 2026-06-09 | Daemon actively owns NIP-29 group per project (closed+public, userNsec-signed) | product | active |
| [2026-06-09-1-package-claude-code-adapter-as-a](episodes/2026-06-09-1-package-claude-code-adapter-as-a.md) | 2026-06-09 | Package Claude Code adapter as a plugin, binary stays separate | architecture | active |
| [2026-06-09-1-read-model-is-the-contract-provider](episodes/2026-06-09-1-read-model-is-the-contract-provider.md) | 2026-06-09 | Read model is the contract; provider is write-side materializer | architecture | superseded |
| [2026-06-09-1-replace-codec-seam-with-fabric-provider](episodes/2026-06-09-1-replace-codec-seam-with-fabric-provider.md) | 2026-06-09 | Replace Codec seam with Fabric Provider architecture | architecture | superseded |
| [2026-06-09-1-syncthing-sync-policy-narrowed-to-markdown](episodes/2026-06-09-1-syncthing-sync-policy-narrowed-to-markdown.md) | 2026-06-09 | Syncthing sync policy narrowed to markdown-only | product | active |
| [2026-06-09-1-user-prompt-publish-hook-with-cross](episodes/2026-06-09-1-user-prompt-publish-hook-with-cross.md) | 2026-06-09 | User prompt publish hook with cross-key signing | product | active |
| [2026-06-09-1-worktree-project-slug-resolves-to-wrong](episodes/2026-06-09-1-worktree-project-slug-resolves-to-wrong.md) | 2026-06-09 | Worktree project slug resolves to wrong project via git --show-toplevel | root-cause | active |
| [2026-06-09-2-add-fabric-relay-to-app-default](episodes/2026-06-09-2-add-fabric-relay-to-app-default.md) | 2026-06-09 | Add fabric relay to app default relay set | product | active |
| [2026-06-09-2-cache-poisoning-from-silent-relay-rejection](episodes/2026-06-09-2-cache-poisoning-from-silent-relay-rejection.md) | 2026-06-09 | Cache-poisoning from silent relay rejection — publish_signed_checked gates cache writes | root-cause | superseded |
| [2026-06-09-2-codec-disambiguates-mentions-from-user-ops](episodes/2026-06-09-2-codec-disambiguates-mentions-from-user-ops.md) | 2026-06-09 | Codec disambiguates Mentions from user OPs by requiring agent tag | product | active |
| [2026-06-09-2-context-injection-moves-from-python-scripts](episodes/2026-06-09-2-context-injection-moves-from-python-scripts.md) | 2026-06-09 | Context injection moves from Python scripts into Rust binary; scripts eliminated | architecture | superseded |
| [2026-06-09-2-mentions-must-carry-sender-session-as](episodes/2026-06-09-2-mentions-must-carry-sender-session-as.md) | 2026-06-09 | Mentions must carry sender session as return envelope | product | active |
| [2026-06-09-2-migrate-default-relay-from-relay-tenex](episodes/2026-06-09-2-migrate-default-relay-from-relay-tenex.md) | 2026-06-09 | Migrate default relay from relay.tenex.chat to nip29.f7z.io | reversal | superseded |
| [2026-06-09-2-single-daemon-architecture-eliminates-state-db](episodes/2026-06-09-2-single-daemon-architecture-eliminates-state-db.md) | 2026-06-09 | Single-daemon architecture eliminates state.db multi-writer corruption | architecture | active |
| [2026-06-09-2-sqlite-multi-writer-corruption-is-a](episodes/2026-06-09-2-sqlite-multi-writer-corruption-is-a.md) | 2026-06-09 | SQLite multi-writer corruption is a confirmed failure mode | root-cause | superseded |
| [2026-06-09-2-who-command-defaults-to-current-project](episodes/2026-06-09-2-who-command-defaults-to-current-project.md) | 2026-06-09 | who command defaults to current project scope with other-projects footer | product | active |
| [2026-06-09-2-who-command-shows-project-summaries-instead](episodes/2026-06-09-2-who-command-shows-project-summaries-instead.md) | 2026-06-09 | who command shows project summaries instead of individual agents in other-projects | product | active |
| [2026-06-09-3-nip-29-uses-two-distinct-relays](episodes/2026-06-09-3-nip-29-uses-two-distinct-relays.md) | 2026-06-09 | NIP-29 uses two distinct relays — presence on auth-gated relay, group management on NIP-29 relay | architecture | active |
| [2026-06-09-3-relay29-authorizes-group-writes-by-event](episodes/2026-06-09-3-relay29-authorizes-group-writes-by-event.md) | 2026-06-09 | Relay29 authorizes group writes by event author, not connection AUTH identity | root-cause | active |
| [2026-06-09-3-session-aware-routing-local-delivery-per](episodes/2026-06-09-3-session-aware-routing-local-delivery-per.md) | 2026-06-09 | Session-aware routing: local delivery, per-session dedup, agent-scoped resolution | root-cause | active |
| [2026-06-09-3-session-short-ids-changed-from-uuid](episodes/2026-06-09-3-session-short-ids-changed-from-uuid.md) | 2026-06-09 | Session short IDs changed from UUID prefixes to hash-based codes to avoid confusion | product | active |
| [2026-06-09-4-cwd-who-8e-correct-implementation-replaces](episodes/2026-06-09-4-cwd-who-8e-correct-implementation-replaces.md) | 2026-06-09 | cwd/who §8e: correct implementation replaces buggy partial | product | active |
| [2026-06-10-1-subscription-fanout-causes-duplicate-events-in](episodes/2026-06-10-1-subscription-fanout-causes-duplicate-events-in.md) | 2026-06-10 | Subscription fanout causes duplicate events in tail | root-cause | superseded |
| [2026-06-10-2-sessionid-newtype-enforces-correct-display-formatting](episodes/2026-06-10-2-sessionid-newtype-enforces-correct-display-formatting.md) | 2026-06-10 | SessionId newtype enforces correct display formatting by construction | architecture | active |
| [2026-06-10-3-restore-accidentally-deleted-propose-command-without](episodes/2026-06-10-3-restore-accidentally-deleted-propose-command-without.md) | 2026-06-10 | Restore accidentally-deleted propose command without agent tags or session requirement | reversal | active |
| [2026-06-12-1-add-project-add-cli-command-for](episodes/2026-06-12-1-add-project-add-cli-command-for.md) | 2026-06-12 | Add `project add` CLI command for NIP-29 group membership | product | active |
| [2026-06-12-1-fabric-provider-seam-closure-no-wire](episodes/2026-06-12-1-fabric-provider-seam-closure-no-wire.md) | 2026-06-12 | Fabric provider seam closure: no wire shape above the provider | architecture | active |
| [2026-06-12-1-inbox-messages-redesigned-as-email-like](episodes/2026-06-12-1-inbox-messages-redesigned-as-email-like.md) | 2026-06-12 | Inbox messages redesigned as email-like envelopes with unified command surface | product | active |
| [2026-06-12-1-secret-scrubbing-layer-inserted-into-nostr](episodes/2026-06-12-1-secret-scrubbing-layer-inserted-into-nostr.md) | 2026-06-12 | Secret-scrubbing layer inserted into Nostr event publishing | product | active |
| [2026-06-12-1-statusline-renders-fabric-citizenship-not-generic](episodes/2026-06-12-1-statusline-renders-fabric-citizenship-not-generic.md) | 2026-06-12 | Statusline renders fabric citizenship, not generic host data | product | active |
| [2026-06-12-1-who-command-show-hostname-instead-of](episodes/2026-06-12-1-who-command-show-hostname-instead-of.md) | 2026-06-12 | who command: show hostname instead of generic (remote) tag | product | superseded |
| [2026-06-12-2-imperative-nip-29-membership-warning-on](episodes/2026-06-12-2-imperative-nip-29-membership-warning-on.md) | 2026-06-12 | Imperative NIP-29 membership warning on first agent turn | product | active |
| [2026-06-12-2-statusline-rpc-is-pure-read-no](episodes/2026-06-12-2-statusline-rpc-is-pure-read-no.md) | 2026-06-12 | Statusline RPC is pure-read, no-spawn, fail-open | architecture | active |
| [2026-06-12-2-tail-stream-deduplication-self-authored-events](episodes/2026-06-12-2-tail-stream-deduplication-self-authored-events.md) | 2026-06-12 | Tail stream deduplication: self-authored events suppressed, canonical thread attribution | product | active |
| [2026-06-13-1-testflight-deploy-now-gates-on-version](episodes/2026-06-13-1-testflight-deploy-now-gates-on-version.md) | 2026-06-13 | TestFlight deploy now gates on version bump (not every push) + unit-only release criterion | architecture | active |
| [2026-06-13-2-branch-protection-on-main-6-cloud](episodes/2026-06-13-2-branch-protection-on-main-6-cloud.md) | 2026-06-13 | Branch protection on main: 6 cloud checks required before merge | architecture | active |
| [2026-06-13-3-test-workflow-cancel-in-progress-per](episodes/2026-06-13-3-test-workflow-cancel-in-progress-per.md) | 2026-06-13 | Test workflow: cancel-in-progress per branch/PR | architecture | active |
| [2026-06-14-1-add-dedicated-indexer-relay-for-kind](episodes/2026-06-14-1-add-dedicated-indexer-relay-for-kind.md) | 2026-06-14 | Add dedicated indexer relay for kind:0 profile publishing and lookup | architecture | active |
| [2026-06-14-1-decouple-session-title-from-active-idle](episodes/2026-06-14-1-decouple-session-title-from-active-idle.md) | 2026-06-14 | Decouple session title from active/idle status | product | superseded |
| [2026-06-14-1-exited-sessions-tui-filter-changed-from](episodes/2026-06-14-1-exited-sessions-tui-filter-changed-from.md) | 2026-06-14 | Exited sessions TUI filter changed from boolean toggle to configurable hours window | product | active |
| [2026-06-14-1-opencode-tenex-edge-plugin-was-stale](episodes/2026-06-14-1-opencode-tenex-edge-plugin-was-stale.md) | 2026-06-14 | opencode tenex-edge plugin was stale and silently broken — updated to unified hook interface | root-cause | active |
| [2026-06-14-1-publish-ack-false-positive-relay-rejection](episodes/2026-06-14-1-publish-ack-false-positive-relay-rejection.md) | 2026-06-14 | Publish-ack false positive: relay rejection surfaced as success | root-cause | active |
| [2026-06-14-1-session-distillation-restructured-single-prompt-title](episodes/2026-06-14-1-session-distillation-restructured-single-prompt-title.md) | 2026-06-14 | Session distillation restructured: single-prompt TITLE+NOW replaces dual prompts | product | active |
| [2026-06-14-1-session-resume-for-any-local-harness](episodes/2026-06-14-1-session-resume-for-any-local-harness.md) | 2026-06-14 | Session resume for any local harness session | product | active |
| [2026-06-14-1-spawn-prompt-replaced-actual-mention-message](episodes/2026-06-14-1-spawn-prompt-replaced-actual-mention-message.md) | 2026-06-14 | Spawn prompt replaced: actual mention message instead of generic 'tenex-edge inbox' | product | active |
| [2026-06-14-1-spawnable-agents-source-of-truth-identity](episodes/2026-06-14-1-spawnable-agents-source-of-truth-identity.md) | 2026-06-14 | Spawnable agents source of truth: identity store replaces PATH | architecture | superseded |
| [2026-06-14-1-tui-sessions-grouped-by-project-with](episodes/2026-06-14-1-tui-sessions-grouped-by-project-with.md) | 2026-06-14 | TUI sessions grouped by project with prioritized tabs and fuzzy search | product | active |
| [2026-06-14-1-who-always-shows-host-including-same](episodes/2026-06-14-1-who-always-shows-host-including-same.md) | 2026-06-14 | who always shows host, including same-machine agents | product | active |
| [2026-06-14-2-claude-code-session-id-env-leak](episodes/2026-06-14-2-claude-code-session-id-env-leak.md) | 2026-06-14 | CLAUDE_CODE_SESSION_ID env leak corrupts all spawned claude processes | root-cause | active |
| [2026-06-14-2-exited-sessions-hidden-by-default-in](episodes/2026-06-14-2-exited-sessions-hidden-by-default-in.md) | 2026-06-14 | Exited sessions hidden by default in TUI | product | superseded |
| [2026-06-14-2-non-attachable-sessions-tui-marks-and](episodes/2026-06-14-2-non-attachable-sessions-tui-marks-and.md) | 2026-06-14 | Non-attachable sessions: TUI marks and blocks unattachable sessions | product | active |
| [2026-06-14-2-session-distillation-engine-immediate-title-seeding](episodes/2026-06-14-2-session-distillation-engine-immediate-title-seeding.md) | 2026-06-14 | Session distillation engine: immediate title seeding, async with timeout, retry on failure | root-cause | superseded |
| [2026-06-14-2-spawned-agent-identity-lost-tenex-edge](episodes/2026-06-14-2-spawned-agent-identity-lost-tenex-edge.md) | 2026-06-14 | Spawned agent identity lost: TENEX_EDGE_AGENT not propagated to tmux pane | root-cause | active |
| [2026-06-14-3-per-agent-independent-tmux-sessions-replace](episodes/2026-06-14-3-per-agent-independent-tmux-sessions-replace.md) | 2026-06-14 | Per-agent independent tmux sessions replace shared session | architecture | active |
| [2026-06-14-3-tui-label-renames-spawnable-agents-spawnable](episodes/2026-06-14-3-tui-label-renames-spawnable-agents-spawnable.md) | 2026-06-14 | TUI label renames: Spawnable→Agents, spawnable via claude→claude | product | active |
| [2026-06-14-4-tui-inline-attach-with-return-to](episodes/2026-06-14-4-tui-inline-attach-with-return-to.md) | 2026-06-14 | TUI inline attach with return-to-list replaces exit-and-exec | product | active |
| [2026-06-15-1-replace-posttooluse-firehose-with-delta-gated](episodes/2026-06-15-1-replace-posttooluse-firehose-with-delta-gated.md) | 2026-06-15 | Replace PostToolUse firehose with delta-gated sibling awareness | product | active |
| [2026-06-15-1-tmux-spawn-uses-selected-project-tab](episodes/2026-06-15-1-tmux-spawn-uses-selected-project-tab.md) | 2026-06-15 | Tmux spawn uses selected project tab instead of process cwd | root-cause | active |
| [2026-06-15-1-tui-rendering-migrated-from-crossterm-full](episodes/2026-06-15-1-tui-rendering-migrated-from-crossterm-full.md) | 2026-06-15 | TUI rendering migrated from crossterm full-clear to ratatui | architecture | active |
| [2026-06-15-1-tui-session-interaction-simplified-now-that](episodes/2026-06-15-1-tui-session-interaction-simplified-now-that.md) | 2026-06-15 | TUI session interaction simplified now that all sessions are resumable | product | active |
| [2026-06-15-2-always-visible-session-sidebar-popup-quick](episodes/2026-06-15-2-always-visible-session-sidebar-popup-quick.md) | 2026-06-15 | Always-visible session sidebar + popup quick-switcher for tmux sessions | product | active |
| [2026-06-15-2-manual-tui-spawns-no-longer-auto](episodes/2026-06-15-2-manual-tui-spawns-no-longer-auto.md) | 2026-06-15 | Manual TUI spawns no longer auto-inject 'tenex-edge inbox' prompt | architecture | active |
| [2026-06-16-1-daemon-accept-loop-starts-before-relay](episodes/2026-06-16-1-daemon-accept-loop-starts-before-relay.md) | 2026-06-16 | Daemon accept loop starts before relay connection | architecture | active |
| [2026-06-16-1-local-agent-keystore-management-cli](episodes/2026-06-16-1-local-agent-keystore-management-cli.md) | 2026-06-16 | Local agent keystore management CLI | product | active |
| [2026-06-16-1-opencode-plugin-becomes-a-dumb-pipe](episodes/2026-06-16-1-opencode-plugin-becomes-a-dumb-pipe.md) | 2026-06-16 | opencode plugin becomes a dumb pipe — inject hook stdout instead of rebuilding context in TS | architecture | active |
| [2026-06-16-1-session-id-tag-corrupted-by-stale](episodes/2026-06-16-1-session-id-tag-corrupted-by-stale.md) | 2026-06-16 | Session-id tag corrupted by stale opencode plugin after JSON output change | root-cause | active |
| [2026-06-16-1-single-owner-turn-transitions-via-canonical](episodes/2026-06-16-1-single-owner-turn-transitions-via-canonical.md) | 2026-06-16 | Single-owner turn transitions via canonical session ID | architecture | active |
| [2026-06-16-1-stale-tmux-pane-attach-now-falls](episodes/2026-06-16-1-stale-tmux-pane-attach-now-falls.md) | 2026-06-16 | Stale tmux pane attach now falls back to transparent resume | product | active |
| [2026-06-16-1-surface-distillation-llm-errors-via-log](episodes/2026-06-16-1-surface-distillation-llm-errors-via-log.md) | 2026-06-16 | Surface distillation LLM errors via log file and statusline flash | product | active |
| [2026-06-16-1-tenex-edge-install-command-with-signature](episodes/2026-06-16-1-tenex-edge-install-command-with-signature.md) | 2026-06-16 | tenex-edge install command with signature-based hook dedup | product | active |
| [2026-06-16-1-who-output-split-into-dual-renderers](episodes/2026-06-16-1-who-output-split-into-dual-renderers.md) | 2026-06-16 | who output split into dual renderers (human vs agent) | product | active |
| [2026-06-16-2-heartbeat-must-re-arm-relay-expiration](episodes/2026-06-16-2-heartbeat-must-re-arm-relay-expiration.md) | 2026-06-16 | Heartbeat must re-arm relay expiration, not just update last_seen | architecture | active |
| [2026-06-16-2-inbox-new-session-replaces-tmux-spawn](episodes/2026-06-16-2-inbox-new-session-replaces-tmux-spawn.md) | 2026-06-16 | inbox new-session replaces tmux spawn as CLI surface | product | active |
| [2026-06-16-2-kind-0-profile-publishing-on-agent](episodes/2026-06-16-2-kind-0-profile-publishing-on-agent.md) | 2026-06-16 | Kind:0 profile publishing on agent creation | architecture | active |
| [2026-06-16-3-self-exclude-viewer-s-own-session](episodes/2026-06-16-3-self-exclude-viewer-s-own-session.md) | 2026-06-16 | Self-exclude viewer's own session from turn-start deltas | product | active |
| [2026-06-17-1-agent-spawn-uses-inline-agent-definitions](episodes/2026-06-17-1-agent-spawn-uses-inline-agent-definitions.md) | 2026-06-17 | Agent spawn uses inline agent definitions with per-harness translation | architecture | active |
| [2026-06-17-1-fabric-delta-block-deduplicates-local-peer](episodes/2026-06-17-1-fabric-delta-block-deduplicates-local-peer.md) | 2026-06-17 | Fabric delta block deduplicates local/peer session echo | root-cause | active |
| [2026-06-17-1-forensics-logs-rearchitected-from-monolithic-to](episodes/2026-06-17-1-forensics-logs-rearchitected-from-monolithic-to.md) | 2026-06-17 | Forensics logs rearchitected from monolithic to per-session layout | architecture | active |
| [2026-06-17-1-growth-lane-narrowed-to-ai-agents](episodes/2026-06-17-1-growth-lane-narrowed-to-ai-agents.md) | 2026-06-17 | Growth lane narrowed to AI agents/infra | reversal | active |
| [2026-06-17-1-macos-trust-cache-stale-inode-causes](episodes/2026-06-17-1-macos-trust-cache-stale-inode-causes.md) | 2026-06-17 | macOS trust-cache stale inode causes SIGKILL on identical binary | root-cause | active |
| [2026-06-17-1-strip-tool-use-from-distillation-transcript](episodes/2026-06-17-1-strip-tool-use-from-distillation-transcript.md) | 2026-06-17 | Strip tool_use from distillation transcript to anchor titles on user intent | product | active |
| [2026-06-17-2-daemon-client-call-must-skip-item](episodes/2026-06-17-2-daemon-client-call-must-skip-item.md) | 2026-06-17 | Daemon client call() must skip item progress frames | architecture | active |
| [2026-06-17-2-hook-tail-tui-ux-overhaul-identifiable](episodes/2026-06-17-2-hook-tail-tui-ux-overhaul-identifiable.md) | 2026-06-17 | Hook-tail TUI UX overhaul: identifiable panes, smart event timeline, detail overlay | product | active |
| [2026-06-17-2-strategy-reversal-replies-first-corpus-first](episodes/2026-06-17-2-strategy-reversal-replies-first-corpus-first.md) | 2026-06-17 | Strategy reversal: replies-first → corpus-first | reversal | active |
| [2026-06-19-1-hook-binary-resolution-switched-from-hardcoded](episodes/2026-06-19-1-hook-binary-resolution-switched-from-hardcoded.md) | 2026-06-19 | Hook binary resolution switched from hardcoded path to PATH lookup | architecture | active |

