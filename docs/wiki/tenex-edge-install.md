---
title: tenex-edge install
slug: tenex-edge-install
topic: installation
summary: `tenex-edge install` is the setup command for wiring tenex-edge into local agent harnesses
tags: []
volatility:
confidence: warm
created:
updated: 2026-06-17
verified:
compiled-from:
sources:
  - session:019ed45f-4c71-78e0-b8a9-e2effe0a80d8
---

# tenex-edge install

`tenex-edge install` is the setup command for wiring tenex-edge into local agent harnesses. Its job is to place the hook commands that call `tenex-edge hook --host <harness> --type <hook-type>` into each harness's native settings, so lifecycle events, context injection, statusline rendering, and idle transitions reach the daemon.

The install surface mirrors proactive-context's installer model: pick a harness explicitly with `--harness`, install every supported harness with `--all`, preview edits with `--dry-run`, inspect install state with `--status`, or remove installed tenex-edge entries with `--uninstall`.

The supported flag surface is:

- `--harness <name>`: install one harness adapter.
- `--all`: install every supported harness adapter.
- `--dry-run`: print the planned edits without writing them.
- `--status`: report whether each supported harness appears installed.
- `--uninstall`: remove tenex-edge-owned entries instead of adding them.

## Harnesses

A harness is the host runtime that executes an agent process and exposes lifecycle hook points. In install context, the harness is not the agent identity. For example, an agent named `haiku` can use the Claude Code harness by running the `claude` command.

Supported harnesses:

- Claude Code: JSON settings at `~/.claude/settings.json`.
- Codex: JSON hook settings at `~/.codex/hooks.json`.
- opencode: TypeScript plugin at `~/.config/opencode/plugin/tenex-edge.ts`.

Claude Code and Codex use the same conceptual hook set: session start, user prompt submit, post tool use, and stop/end. Claude Code also gets a `statusLine` command so its status bar can read the tenex-edge statusline RPC. opencode receives the same lifecycle behavior through its plugin, which maps opencode SDK events to the unified Rust `hook` entry point.

## Agent Environment

`TENEX_EDGE_AGENT` is the authoritative active agent variable set by tenex-edge tmux via `tmux -e`, so an explicitly chosen agent like `haiku` wins even if it uses `claude` as the underlying command. `TENEX_EDGE_AGENT_FALLBACK` is the harness settings fallback variable, used only when `TENEX_EDGE_AGENT` is unset or empty, intended for direct unspawned launches like running `claude` manually. Harness settings must use `TENEX_EDGE_AGENT_FALLBACK` instead of `TENEX_EDGE_AGENT` to avoid overriding an explicit tmux-spawned agent. The lookup order is:

1. `TENEX_EDGE_AGENT`
2. `TENEX_EDGE_AGENT_FALLBACK`
3. the hook host default, such as `claude`, `codex`, or `opencode`

All CLI paths that resolve the current agent — including session-start, inbox, reply, chat, propose, wait-for-mention, statusline, and mid-turn checks — follow this precedence rule. This lets `tenex-edge tmux` start `haiku` with a `claude` command and still register the session as `haiku`, while direct `claude` launches can fall back to `developer`.

<!-- citations: [^019ed-1] -->
## JSON Merge Strategy

Claude Code and Codex use a JSON merge install strategy. The installer reads the existing settings file, preserves unrelated user and tool configuration, removes any previous tenex-edge hook entries matching the same hook signature, and writes the desired tenex-edge entries back into the appropriate hook arrays.

Hook identity is based on the command signature `hook --host X --type Y`, not on the binary path. That matters because the binary can move between a cargo build path, `~/.local/bin/tenex-edge`, or another install location. A reinstall should replace the old tenex-edge hook for the same host/type instead of accumulating duplicates just because the path changed.

Signature deduplication deliberately ignores the executable prefix. These two commands are the same tenex-edge hook for install purposes because the harness and hook type are identical:

```text
/Users/pablofernandez/src/tenex-edge/target/release/tenex-edge hook --host codex --type user-prompt-submit
~/.local/bin/tenex-edge hook --host codex --type user-prompt-submit
```

They should collapse to one configured hook. A prefix-based rule would keep both after a reinstall from a different binary path and would inject duplicate context into every turn.

For Claude Code, the merge covers lifecycle hooks and the `statusLine` command. For Codex, the merge covers the supported hook events in `~/.codex/hooks.json`. Uninstall uses the same signature matching to remove tenex-edge hooks, and removes tenex-edge-owned `statusLine` configuration while leaving unrelated entries alone.

## File Drop Strategy

opencode uses a file drop install strategy. The installer writes the canonical plugin source from `integrations/opencode/tenex-edge.ts` into `~/.config/opencode/plugin/tenex-edge.ts`.

The plugin is the harness adapter for opencode: it calls the unified Rust hook entry point, forwards opencode's process id and native session id where needed, and injects hook stdout back into the model turn. opencode loads plugins at startup, so plugin changes take effect on the next opencode launch.

File drop is different from JSON merge because opencode's integration point is a plugin file, not a hook array inside an existing settings object. The installer therefore owns the single `tenex-edge.ts` plugin path and can create, overwrite, or remove that file while leaving unrelated opencode plugins alone.

## Dry Run

`--dry-run` reports what would change without writing files. For JSON merge harnesses, it should show the target settings path and the hook/statusline entries that would be added, replaced, or removed. For file drop harnesses, it should show the target plugin path and whether the file would be created or overwritten.

Dry-run output is also useful for verifying deduplication: if an existing hook points to a different tenex-edge binary path but has the same `hook --host X --type Y` signature, dry-run should report a replacement, not an additional hook.

Dry-run should be side-effect free: no settings files are rewritten, no plugin file is dropped, and no existing hook is removed. Its output is the operator-facing diff of the install plan.

## Uninstall

`--uninstall` removes previously installed tenex-edge integration entries. For Claude Code and Codex, it removes hook commands by signature. For Claude Code, it also removes tenex-edge-owned `statusLine` configuration. For opencode, it removes the dropped tenex-edge plugin file. It should preserve non-tenex settings and other tools' hooks.

Uninstall is the inverse of install, not a settings reset. JSON merge uninstall scans the existing settings, removes entries whose command signature matches tenex-edge's `hook --host X --type Y`, removes the tenex-edge `statusLine` command for Claude Code, and writes the remaining user configuration back. File drop uninstall removes only the owned opencode plugin path.

## proactive-context Model

proactive-context's install command is the model: a single CLI command owns harness-specific setup, supports dry-run/status/uninstall modes, preserves existing settings, and makes reinstall idempotent. tenex-edge follows the same shape but installs tenex-edge's native hook commands and opencode plugin instead of proactive-context's hook commands.

The design inheritance from proactive-context is operational rather than protocol-level: one setup command owns all supported host wiring, each harness gets an adapter strategy that matches its native configuration surface, and repeated installs converge to one correct configuration instead of appending duplicates.
