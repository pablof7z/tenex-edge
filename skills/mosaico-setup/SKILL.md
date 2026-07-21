---
name: mosaico-setup
description: Install, configure, repair, verify, update, or uninstall Mosaico and its coding-agent integrations. Use when an agent is told to follow https://mosaico.f7z.io/SETUP.md, when preparing Mosaico on a new macOS or Linux machine, or when wiring Claude Code, Codex, OpenCode, Grok Build, Goose, or Hermes Agent into a shared Mosaico fabric.
---

# Set up Mosaico

Treat setup as an inspect, explain, install, verify workflow. Keep the user informed about commands, credentials, and file changes. Do not claim success until `mosaico doctor` passes.

## Understand the product

Mosaico gives the coding agents a person already runs a shared awareness fabric. Each live agent receives an identity and handle, can see peer presence and status, and can send an `@mention` that arrives as a real turn in another agent's native session. Mosaico does not replace the harness, run the model, or read every transcript to guess what agents are doing.

One local daemon owns Mosaico's SQLite state and relay connection. Thin harness integrations call the `mosaico` binary at lifecycle boundaries and inject relevant fabric context. They are fail-open: a Mosaico failure must not prevent the coding harness from continuing its own work.

Current harness support:

| Harness | Integration |
| --- | --- |
| Claude Code | Lifecycle hooks in Claude settings, plus the Mosaico runtime skill. |
| Codex | Lifecycle hooks in Codex's hook configuration, plus the shared runtime skill. |
| OpenCode | A TypeScript plugin that supplies lifecycle events and fabric context. |
| Grok Build | Lifecycle hooks in Grok's hook configuration. |
| Goose | Native ACP transport. Goose needs no Mosaico hook installation. |
| Hermes Agent | A Hermes user plugin for lifecycle integration, with native ACP and PTY transports. |

## Follow the safety contract

1. Work only on macOS or Linux. Stop on Windows instead of improvising a partial install.
2. Inspect the machine before changing it. Determine the OS, architecture, current `mosaico` binary, Rust/Cargo availability, Git availability, installed harnesses, and relevant home-directory overrides.
3. Explain the proposed commands and file changes before running them.
4. Do not install Rust, a package manager, or another harness without explicit user approval. If Cargo is absent, identify the correct platform-specific Rust installation path and ask first.
5. Do not invent operator pubkeys, relay URLs, or host labels. Use installer defaults only after showing them to the user; ask when identity or network choices are unclear.
6. Do not use `--all` by default. Let the interactive installer detect installed harnesses and let the user review the selected integrations.
7. Preserve unrelated hooks and configuration. Stop and report any merge conflict or malformed config instead of overwriting it.
8. Never delete `~/.mosaico` or `$MOSAICO_HOME` as part of normal uninstall. It contains identity and session state.

## Inspect the host

Run equivalent read-only checks for the current shell:

```bash
uname -s
uname -m
command -v git || true
command -v cargo || true
command -v mosaico || true
for harness in claude codex opencode grok goose hermes; do command -v "$harness" || true; done
```

Honor `MOSAICO_HOME`, `GROK_HOME`, and `HERMES_HOME` when they are already set. Do not set them merely to make setup pass.

If `mosaico` already exists, inspect `mosaico --help` and `mosaico install --status` before updating it. A checkout-local or stale binary can differ from the current repository contract.

## Obtain the binary

Use the current GitHub source installation until a versioned release channel is documented here:

```bash
cargo install --git https://github.com/pablof7z/mosaico --locked --force
```

This compiles Mosaico and installs the binary into Cargo's bin directory, normally `~/.cargo/bin`. Confirm that the exact installed binary is on `PATH`:

```bash
command -v mosaico
mosaico --help
```

If it is not on `PATH`, explain the required shell-path change and ask before editing a shell profile. Do not fall back to an arbitrary binary copied from another checkout or machine.

## Preview and install integrations

Preview the detected footprint first:

```bash
mosaico install --dry-run
```

Then run the installer in an interactive terminal:

```bash
mosaico install
```

Keep the Mosaico runtime skill selected. Review the detected harnesses with the user and apply only the integrations they want. If an interactive terminal is unavailable, use `--harness` with an explicit comma-separated list only after the user approves that list.

The installer may:

- create or update `~/.mosaico/config.json` (or `$MOSAICO_HOME/config.json`) with the relay, host label, operator allowlist, and a generated backend management key;
- install the packaged `mosaico` runtime skill under `~/.agents/skills/mosaico` and expose it to supported skill-aware harnesses;
- merge Mosaico-owned hook groups into Claude Code, Codex, or Grok configuration;
- write the OpenCode Mosaico plugin;
- install and enable the Hermes Mosaico user plugin.

Goose is detected for agent launch and uses native `goose acp`; it has no hook entry to install. Existing non-Mosaico hooks and settings must remain intact.

Mosaico stores local state under `~/.mosaico` by default and connects to the configured Nostr relay. Attachment sending can upload explicitly attached files to the relay's configured Blossom service. Setup does not configure third-party analytics; Mosaico does keep local hook and command diagnostics in its own state.

## Verify and repair

Restart any harness sessions that were already open so they load the new integration. Then inspect the installed footprint:

```bash
mosaico install --status
mosaico doctor --json
```

Use the structured report to distinguish errors from warnings. A detected harness that was not selected is an opt-in warning, not permission to install it. If the doctor reports repairable errors, summarize the exact proposed repairs and ask the user whether to apply them. Only after the user approves repair, run:

```bash
mosaico doctor --fix --json
mosaico doctor --json
```

Do not replace doctor with ad hoc config edits. If doctor still fails, preserve its exact diagnostic, inspect the named path or service, fix only the proven cause, and rerun doctor.

For the first useful loop, have the user start two supported agents in the same repository. In each restarted agent, load the installed Mosaico skill and inspect `mosaico my session`. Confirm that both sessions appear in the same workspace, then send one addressed handoff and wait for the reply. Do not call setup complete based only on files existing.

## Update

Reinstall from the current source, rerun the integration installer, and repair any detected drift:

```bash
cargo install --git https://github.com/pablof7z/mosaico --locked --force
mosaico install
mosaico doctor --json
```

Review installer selections again because the set of installed harnesses may have changed. If the final doctor report contains errors, follow the approval and repair loop above.

## Uninstall

Explain that normal uninstall removes Mosaico-owned integration entries and the packaged runtime skill but preserves local identity and session data. Then run:

```bash
mosaico install --uninstall
mosaico daemon stop
cargo uninstall mosaico
```

Restart previously open harness sessions after removing integrations. If the user also asks to purge data, resolve the exact active `MOSAICO_HOME` (default `~/.mosaico`), show it, explain that its identities and session history are not recoverable, and request a separate explicit confirmation before removing it.
