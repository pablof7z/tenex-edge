---
name: mosaico-dev
description: "Use for mosaico development live labs: run a local croissant relay, configure isolated PTY or ACP/app-server agent bundles with real host AI auth, launch Claude/Codex/OpenCode agents, and inspect sessions, logs, relay traffic, and Nostr events."
allowed-tools: Bash
---

# mosaico development live lab

Use this skill when validating mosaico changes in a real local environment:
a host croissant relay, isolated container backends, real host
Claude/Codex/OpenCode auth, PTY and ACP/app-server agents, relay-level traffic,
hook logs, and Nostr events inspected with `nak`.

This is the replacement mindset for the old scripted e2e harness. Prefer a
small live lab that can be inspected over a mocked shortcut. The objective is
not to prove model quality; it is to prove that the real agents, hooks, relay
traffic, launch paths, and injected context work together.

## Resource Map

Read these files from this skill directory as needed:

- `references/live-lab-workflow.md`: the complete start-to-finish lab procedure,
  including single-agent, multi-agent, direct harness, and `mosaico launch`
  flows.
- `references/container-backends.md`: how host credentials, container state,
  profile configs, model choices, and launch modes fit together.
- `references/acp-backends.md`: the two-file bundle contract, structured
  profiles, smoke/headless launch workflow, and ACP-specific troubleshooting.
- `references/observability.md`: how to inspect PTY sessions, croissant logs,
  Nostr events, hook logs, daemon logs, and what counts as evidence.
- `references/troubleshooting.md`: common failures and the concrete checks to
  run before escalating.
- `scripts/start-croissant-relay`: starts croissant as a host process, creates a
  per-run work directory, waits for NIP-11, and writes `lab.env`.
- `scripts/write-container-profiles`: writes disposable backend config plus the
  current `harnesses.json` and `agents/<slug>.json` launch contract.
- `scripts/launch-agent`: runs a raw provider CLI, an ACP/app-server smoke, or
  `mosaico launch`; structured launches are headless and PTY launches attach.
- `scripts/probe-lab`: captures relay NIP-11, relay logs, and selected Nostr
  event kinds into a probe directory.
- `scripts/cleanup-lab`: stops recorded agent containers and then stops the
  relay process.

## Non-Negotiables

- Use real host AI auth. The container runner defaults to
  `MOSAICO_CONTAINER_HOST_AUTH=1`; it mounts host auth directories read-only
  and projects credential files into isolated container state.
- Do not create fake provider files or fake logins unless the user explicitly
  asks for a non-agent smoke test.
- Do not print secrets. Never show auth files, provider files, generated `nsec`
  values, or raw private keys in the transcript or final report.
- Keep fabric state isolated. Generated state belongs under `.container-state/`
  or a temp live-lab work directory, not in the host `~/.mosaico`.
- Run croissant on the host at `/tmp/croissant-smallmap` when present, else
  `${HOME}/Work/croissant`; set `MOSAICO_DEV_CROISSANT_DIR` to override.
  Containers point at the host relay URL; croissant itself does not need a
  container.
- Use the cheapest useful model for each provider. The lab only needs enough
  model ability to run commands and surface UI/hook behavior.
- Use `mosaico launch` when validating reattach, injection, or hosted
  lifecycle behavior. Direct runs are foreground auth/plugin checks;
  `__acp-smoke` proves structured transport handshake, prompt, and resume.
- Interactive `mosaico launch` of an ACP/app-server agent offers attachable
  PTY bundles plus a final headless choice. Automated structured launches pass
  `--headless`; PTY launches may pass `--harness <bundle>` explicitly.
- Harness configuration is two-file state under `MOSAICO_HOME`:
  `harnesses.json` defines bundles and `agents/<slug>.json` selects a bundle via
  `harness`. The filename is plural; there is no `harness.json` surface.
- Use `mosaico my session` for agent identity/fabric inspection. Do not rely on obsolete
  identity commands.

## Standard Start

From the mosaico repo root:

```bash
git status -sb
bash containers/mosaico/run build-image
bash containers/mosaico/run doctor
```

`doctor` must verify every installed backend/transport tool, `nak`, and the
selected profile's provider auth plus hook/plugin installation. If auth checks
fail, stop and report the
missing host path; do not silently switch to new credentials.

Start a relay:

```bash
skills/mosaico-dev/scripts/start-croissant-relay
```

The command prints an `env=.../lab.env` path. Use that exact file for the rest
of the run:

```bash
LAB_ENV=/tmp/mosaico-live-lab-YYYYmmdd-HHMMSS/lab.env
skills/mosaico-dev/scripts/write-container-profiles "${LAB_ENV}" claude-acp
```

Prewarm and verify the exact profile before launching the agent UI:

```bash
bash containers/mosaico/run --profile claude-acp doctor
skills/mosaico-dev/scripts/launch-agent "${LAB_ENV}" smoke claude-acp
```

For multiple backends:

```bash
skills/mosaico-dev/scripts/write-container-profiles "${LAB_ENV}" \
  claude-acp codex-app-server opencode-acp
```

The writer generates disposable local Nostr keys, writes
`.container-state/<profile>/mosaico/harnesses.json` plus the selecting agent
file, and whitelists every generated pubkey in every profile. Model-provider
auth still comes from the host credential mounts.

To exercise a normal named Codex config profile through app-server, regenerate
the bundle with its profile name:

```bash
MOSAICO_DEV_CODEX_CONFIG_PROFILE=planner \
  skills/mosaico-dev/scripts/write-container-profiles "${LAB_ENV}" codex-app-server
```

The runner stages host `~/.codex/*.config.toml` files without exposing their
contents. The harness composes the selected file over base `config.toml` in an
isolated `CODEX_HOME`; project config and inline bundle overrides still win.

## Launch Patterns

Direct harness run in the foreground:

```bash
skills/mosaico-dev/scripts/launch-agent "${LAB_ENV}" direct claude --model haiku
```

Run through `mosaico launch` in portable PTY mode:

```bash
bash containers/mosaico/run --profile claude mosaico channel init
skills/mosaico-dev/scripts/launch-agent "${LAB_ENV}" launch claude --model haiku
```

Codex example:

```bash
skills/mosaico-dev/scripts/launch-agent "${LAB_ENV}" direct codex -m gpt-5.3-codex-spark
```

OpenCode example:

```bash
skills/mosaico-dev/scripts/launch-agent "${LAB_ENV}" direct opencode-ollama "${MOSAICO_OPENCODE_OLLAMA_MODEL:-ollama/deepseek-r1:8b}"
```

If a CLI rejects the model flag, record the rejection and fall back to the
cheapest configured model that works. Do not block the lab on an exact model
name unless the feature under test depends on that provider/model.

Structured launch examples:

```bash
bash containers/mosaico/run --profile claude-acp mosaico channel init
MOSAICO_DEV_PROMPT="Run mosaico my session and summarize the self header." \
  skills/mosaico-dev/scripts/launch-agent "${LAB_ENV}" launch claude-acp
```

The helper passes `--headless` for ACP/app-server profiles so an interactive
terminal does not stop at the launch-mode picker.

Use `codex-app-server` and `opencode-acp` the same way. Configure their model
through the profile-writer environment described in `container-backends.md`;
do not pass provider CLI flags to an ACP/app-server launch.

## Inspecting The Run

Inspect or attach to PTY launch sessions with the human session picker:

```bash
bash containers/mosaico/run --profile claude mosaico sessions
```

Drive an agent through its attached terminal or send it a channel mention; there
is no public PTY injection command.

```bash
bash containers/mosaico/run --profile claude mosaico channel send \
  --channel <channel> --message "@<session-handle> Run mosaico my session."
```

Probe everything into files:

```bash
skills/mosaico-dev/scripts/probe-lab "${LAB_ENV}" <agent-session>
```

Also inspect:

```bash
bash containers/mosaico/run --profile claude mosaico debug hook-tail
tail -n 200 .container-state/claude/mosaico/daemon.log
tail -n 200 .container-state/claude/mosaico/relay.log
```

Croissant logs all inbound/outbound traffic and rejected event reasons to the
relay log named in `lab.env`. Use those logs together with `nak` event probes
and transport-specific evidence. ACP/app-server launches print
`[mosaico acp] session: ...` and have no PTY to attach; prove them with the
smoke output, launch session id, daemon/delivery logs, and relay events.

## Evidence Standard

A useful report contains:

- relay URL, run id, profile names, and whether this was direct or launch mode
- exact agent commands and accepted model flags
- PTY attach output for PTY agents, or `__acp-smoke` plus headless session and
  delivery evidence for ACP/app-server agents
- croissant log excerpts showing traffic or rejection reasons
- `nak` evidence for the expected event kinds
- hook-tail or daemon log evidence when testing hook injection
- pass/fail tied to the feature under test, plus the next failing command if it
  did not pass

## Simple Agent Validation Prompt

Give this to a simple agent to validate that the skill works:

```text
Use the mosaico-dev skill. Start a fresh local croissant relay on the host
without forcing port 9888. Generate a `claude-acp` container profile with real
host Claude auth, disposable fabric keys, `harnesses.json`, and a selecting
agent file. Run profile doctor and the ACP smoke, initialize the workspace
channel, then launch the headless Claude ACP agent with an initial prompt asking
it to run `mosaico my session`. Collect the ACP session id, daemon/delivery and
croissant logs, hook evidence, and nak relay probes. Clean up with
scripts/cleanup-lab. Do not print secret or auth file contents. Report exact
commands/results; if it fails, write concise lessons to the skill.
```
