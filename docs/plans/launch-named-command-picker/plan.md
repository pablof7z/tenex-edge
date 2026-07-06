# Launch Named Command Picker for Issue 258

## Summary

Plan a clean launch-command model for tenex-edge agents: `commands` is the only local launch-command schema, CLI-side selection handles interactive launches, missing commands get guided suggestions, and daemon paths stay noninteractive.

## Boundaries

```mermaid
flowchart LR
  JSON[Local agent JSON\ncommands array only] --> Identity[identity launch-command API]
  Identity --> CLI[tenex-edge launch\nTTY picker / command-name / override]
  Identity --> Registry[tmux registry\nnoninteractive default]
  CLI --> RPC[tmux_spawn RPC\nconcrete base_command argv]
  Registry --> RPC
  RPC --> Tmux[tmux session spawn]
```

## Detailed Plan

## Scope

Implement issue #258 for `tenex-edge launch <agent>` and the local agent config model. Do not close the issue from the planning PR. The implementation PR should close it after code, tests, and durable docs land.

## Data Model

Replace the old singular `command` field with one canonical named-command shape: `name: String` and `argv: Vec<String>`, stored in a `commands` array on agent JSON. Do not deserialize, normalize, or honor the old singular `command` field for launch resolution. An agent file with only `command` is treated as having zero configured commands, which routes through the guided missing-command flow.

New writes always write `commands`. If editing an existing file that contains `command`, remove that field when writing the file so the schema converges immediately. Do not add an automatic migration for untouched files.

Because `src/identity.rs` is already above the soft line target, put launch-command parsing, canonical writes, and suggestion helpers in a cohesive identity submodule or another domain-owned submodule with `pub(super)` or `pub(crate)` visibility.

## CLI Behavior

Keep `-c/--command` as the current full command override. Add a new noninteractive selector, preferably `--command-name <name>`, because `--command` is already taken. If the selector is present, resolve the named choice or fail with the available names. If multiple choices exist and no selector or override exists, require a TTY and show a `dialoguer` or `inquire` picker. If stdin/stdout is not a TTY, fail with a concrete message telling the caller to pass `--command-name` or `-c`.

If zero configured choices exist, prompt on TTY. Suggestions come from other agent files that already have `commands` entries. Do not read old singular `command` entries as suggestions. Adapt suggestions with two safe rules: replace a literal `{slug}` placeholder with the target slug, and replace exact source-slug argv tokens or path filename stems equal to the source slug. Do not perform broad substring replacement. If no local suggestions exist, show built-in harness defaults. Include a custom-command entry that shell-splits user input with the existing `shlex` dependency. Persist the selected or custom command atomically as a named command before spawning.

## Daemon And Tmux Boundaries

Do not make `tmux_spawn`, `spawn_agent`, or `resolve_spawn_entry` interactive. The CLI launch command should pass the selected argv through `base_command`. The existing daemon fallback remains deterministic for TUI or background callers: choose the first canonical `commands` entry, or built-in default if the slug matches `SPAWN_DEFS`; otherwise fail with an error telling the caller to configure `commands`. `spawnable_agents()` can display the default command plus an indication when multiple choices exist, but it should not require a prompt.

Resume should continue using the deterministic default command, because a prior session resume must not stop for a picker. A future enhancement can persist the command used by a session if exact resume-command parity becomes necessary.

## Validation

Add unit tests for canonical command parsing, strict non-use of singular `command`, command precedence, empty argv filtering or errors, unique-name handling, and canonical writes that remove `command`. Add pure helper tests for suggestion adaptation, including placeholder replacement, exact token replacement, path-stem replacement, and no broad substring mutation. Add CLI parse tests for `--command-name` alongside the existing `-c/--command` override. Add registry tests showing deterministic default resolution for multi-command configs and no fallback from legacy-only configs.

Run `cargo fmt --check`, targeted tests for identity/tmux CLI modules, and `cargo test --lib` if the change touches shared identity or spawn behavior. Avoid formatting churn outside touched files.

## Docs

After implementation, update the authoritative launch docs in `docs/wiki/guides/tenex-edge-launch.md` and the CLI surface docs in `docs/wiki/guides/tenex-edge-cli-commands.md`. If the config schema details become durable user-facing knowledge, add them to the existing agent identity or config guide instead of creating a new planning file.

## Rollout And Rollback

Rollout is intentionally strict: files with only old `command` will be treated as missing command configuration and will prompt on interactive launch. Rollback to an older binary may require manually restoring the old singular field if the file was rewritten to `commands`. That tradeoff is accepted to avoid maintaining duplicate schema behavior.

## Risks And Open Questions

The largest UX risk is confusion between the existing full command override and the new named selector; using `--command-name` avoids breaking the current `--command` contract. The largest correctness risk is unsafe suggestion adaptation; keep it conservative and covered by tests. The plan intentionally does not add last-used command memory, because that introduces hidden state and is not necessary for the requested picker.

## Rule And ADR Check

- No ADR files exist in this repository; governing constraints are AGENTS.md plus durable architecture/product docs.
- The GitHub issue remains the backlog source of truth; the generated docs/plans artifact is a temporary planning PR artifact and must be retired or collapsed after implementation.
- The implementation should respect the 300-line soft and 500-line hard file-size rule: identity.rs is already 382 lines, so launch-command parsing or suggestion logic should move into a cohesive submodule with narrow visibility.
- The product doctrine says agent identity persists across hosts; this plan keeps launch commands as local per-agent configuration attached to the existing local agent identity file, not to transient sessions.
- The daemon RPC stays noninteractive and receives concrete argv, preserving existing ownership boundaries between CLI UX, daemon provisioning, and tmux spawning.
- No compatibility layer is required for the old singular command field; strict replacement is an owner decision for this issue.

## Possible Rule Or ADR Loosening

- No permanent repository rule needs loosening.
- This intentionally accepts local config breakage for files that only use the old singular command field; that is a product/schema decision, not a repository-rule violation.

## Possible Rule Tightening

- Consider documenting that daemon RPC handlers and background spawn paths must never prompt on stdin; all interactive selection belongs in explicit CLI/TUI layers.
- Consider documenting that local config schema replacements should avoid read fallbacks and aliases unless the owner explicitly asks for compatibility.

## Alternatives Considered

- Keep legacy command as a read fallback: lower short-term friction, but rejected because the owner wants no legacy path and duplicate config state is not worth preserving.
- Array-of-tuples schema: compact, but weaker for validation and future fields than object entries with name and argv.
- Prompt inside resolve_spawn_entry or the daemon: fewer call sites, but unsafe for RPC, TUI, and background spawn callers because it can hang noninteractive paths.
- Always pick the first configured command silently: simplest, but fails the requested launch-time choice behavior and hides privileged command variants.
- Broad string substitution for cross-agent suggestions: convenient, but too likely to mutate unrelated paths or flags; explicit placeholders plus exact token/stem replacement is safer.

## Certainty

91 percent.

## Decision

ready

## Hosted Artifacts

- Plan page: Generated after publishing.

- TTS audio: https://blossom.primal.net/d1f883e49f85dd95a1126b52b4bc5c316657f037f097ecc98d0bbe2dde19c37c.mp3
