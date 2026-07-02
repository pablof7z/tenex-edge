---
description: Use for tenex-edge development live labs: run a local croissant relay, configure one or more container backends with real host AI auth, launch Claude/Codex/OpenCode agents, and inspect tmux panes, hook logs, relay logs, and Nostr events with nak.
allowed-tools: Bash
---

# tenex-edge development live lab

Use this skill when testing tenex-edge behavior in a real local environment:
local croissant relay on the host, one or more isolated container backends, real
Claude/Codex/OpenCode credentials, observable tmux panes, and relay-level proof.

This replaces the old scripted `e2e/` harness mindset. Prefer live, inspectable
runs over mocked or hermetic shortcuts.

## Ground Rules

- Use real host AI auth. The container runner defaults to
  `TENEX_EDGE_CONTAINER_HOST_AUTH=1`, mounts host auth directories read-only, and
  symlinks credential files into isolated state. Do not run fake logins or stub
  provider files.
- Do not print secrets. Never `cat` auth files, provider files, `nsec` values, or
  generated private keys into the transcript or final report.
- Keep fabric state isolated. Generated state belongs under
  `.container-state/<profile>` or a temp run directory, not in `~/.tenex-edge`.
- Run croissant on the host, not in a container. Containers should point at the
  host relay URL.
- Use the cheapest useful model. This is for proving hooks, relay traffic, UI
  injection, and routing, not model quality.
- Prefer `tenex-edge launch` when testing launched-agent behavior. Wrap direct
  harness runs in host tmux when testing without `tenex-edge launch`.

## First Checks

From `/Users/pablofernandez/src/tenex-edge`:

```bash
git status -sb
bash containers/tenex-edge/run build-image
bash containers/tenex-edge/run doctor
```

`doctor` must show `claude`, `codex`, `opencode`, `nak`, and `tmux`, then install
Claude hooks, Codex hooks, and the OpenCode plugin inside container state.

If auth checks fail, stop and report the missing host auth path. Do not continue
with isolated credentials unless the user explicitly asks for a non-agent smoke
test.

## Croissant Relay

Croissant lives at `/Users/pablofernandez/Work/croissant`. It needs:

- `PORT`, default `9888`
- `HOST`, bind to the host bridge IP that containers can reach
- `DATAPATH`, a fresh per-run data directory
- `OWNER_PUBLIC_KEY`, a hex pubkey
- `DOMAIN`, empty for local plain `ws://`

For Apple containers, the host bridge is usually `192.168.64.1`. Verify cheaply:

```bash
ipconfig getifaddr bridge100 2>/dev/null || echo 192.168.64.1
```

Do not bind croissant to `0.0.0.0` for these tests; croissant uses `HOST` when
constructing its relay URL. Use the bridge IP so containers can connect and logs
show the correct local URL.

Typical relay launch:

```bash
RUN_ID="$(date +%Y%m%d-%H%M%S)"
RELAY_HOST="$(ipconfig getifaddr bridge100 2>/dev/null || echo 192.168.64.1)"
RELAY_PORT="${TENEX_EDGE_DEV_RELAY_PORT:-9888}"
RELAY_WS="ws://${RELAY_HOST}:${RELAY_PORT}"
RELAY_HTTP="http://${RELAY_HOST}:${RELAY_PORT}"
RELAY_DATA="${TMPDIR:-/tmp}/tenex-edge-croissant-${RUN_ID}"
RELAY_TMUX="te-relay-${RUN_ID}"

mkdir -p "${RELAY_DATA}"
OWNER_SK_FILE="${RELAY_DATA}/owner.nsec"
nak key generate >"${OWNER_SK_FILE}"
OWNER_PK="$(nak key public "$(cat "${OWNER_SK_FILE}")")"

(cd /Users/pablofernandez/Work/croissant && CGO_ENABLED=1 go build -o ./croissant)
tmux new-session -d -s "${RELAY_TMUX}" \
  "cd /Users/pablofernandez/Work/croissant && PORT=${RELAY_PORT} HOST=${RELAY_HOST} DATAPATH=${RELAY_DATA} OWNER_PUBLIC_KEY=${OWNER_PK} DOMAIN= ./croissant"
```

Check liveness and relay support:

```bash
curl -fsS -H 'Accept: application/nostr+json' "${RELAY_HTTP}" | jq .
nak req -k 39000 "${RELAY_WS}" | jq -c .
```

Croissant logs all traffic, including rejected event reasons. Keep its tmux pane
available:

```bash
tmux capture-pane -pt "${RELAY_TMUX}" -S -200 -e
```

## Container Backend Config

Each backend profile gets isolated fabric state:

```text
.container-state/<profile>/tenex/config.json
.container-state/<profile>/tenex/edge/
.container-state/<profile>/home/
```

For a local relay lab, write a profile config that points only at croissant and
uses test fabric keys. The model-provider credentials still come from the host
auth mounts; these Nostr keys are only for the disposable local fabric.

For multiple profiles, generate all backend keys first, compute their pubkeys
with `nak key public`, and put every backend pubkey in every profile's
`whitelistedPubkeys`.

Config shape:

```json
{
  "whitelistedPubkeys": ["<pubkey-a>", "<pubkey-b>"],
  "relays": ["ws://192.168.64.1:9888"],
  "indexerRelay": "ws://192.168.64.1:9888",
  "backendName": "edge-a",
  "userNsec": "<generated-test-nsec>",
  "tenexPrivateKey": "<generated-test-nsec>"
}
```

Never report the `nsec` values. Report only profile names, pubkey prefixes, and
paths.

## Launching Agents

Use the runner commands for direct harness runs:

```bash
bash containers/tenex-edge/run claude --model haiku
bash containers/tenex-edge/run codex -m gpt-5.3-codex-spark
bash containers/tenex-edge/run opencode-ollama "${TENEX_EDGE_OPENCODE_OLLAMA_MODEL:-ollama/deepseek-r1:8b}"
```

If a CLI rejects a model flag, record the rejection and fall back to the cheapest
configured model for that provider. For OpenCode, prefer a DeepSeek/Ollama Cloud
flash-class model when available, but do not block the test on an exact model
name. The test goal is "agent can act", not model benchmarking.

When testing `tenex-edge launch`, run it inside a host tmux session so the outer
terminal remains inspectable:

```bash
tmux new-session -d -s "te-claude-${RUN_ID}" \
  "cd /Users/pablofernandez/src/tenex-edge && bash containers/tenex-edge/run --profile claude tenex-edge launch claude -- --model haiku"
```

When testing a harness directly, still use host tmux:

```bash
tmux new-session -d -s "te-direct-claude-${RUN_ID}" \
  "cd /Users/pablofernandez/src/tenex-edge && bash containers/tenex-edge/run claude --model haiku"
```

Use `tmux send-keys` to drive prompts and `tmux capture-pane -e` to inspect the
exact UI:

```bash
tmux capture-pane -pt "te-direct-claude-${RUN_ID}" -S -200 -e
tmux send-keys -t "te-direct-claude-${RUN_ID}" "Run tenex-edge who and summarize the self header." C-m
```

## Observability

Use all of these when validating behavior:

- Agent UI: `tmux capture-pane -pt <session> -S -200 -e`
- Relay wire log: capture the croissant tmux pane
- Relay facts: `nak req -k 39000`, `nak req -k 39001`, `nak req -k 39002`,
  `nak req -k 30315`, and `nak req -k 9`
- tenex-edge relay log:
  `.container-state/<profile>/tenex/edge/relay.log`
- tenex-edge daemon log:
  `.container-state/<profile>/tenex/edge/daemon.log`
- Hook telemetry:
  `bash containers/tenex-edge/run --profile <profile> tenex-edge debug hook-tail`

Useful relay probes:

```bash
nak req -k 39000 "${RELAY_WS}" | jq -c '{kind,pubkey,tags,content}'
nak req -k 30315 "${RELAY_WS}" | jq -c '{kind,pubkey,tags,content}'
nak req -k 9 "${RELAY_WS}" | jq -c '{kind,pubkey,tags,content}'
```

## What To Prove

A good live-lab report includes:

- the relay URL, run id, and profile names
- which agents launched and which model flags were used
- tmux captures showing the agent UI and any injected tenex-edge context
- croissant log excerpts showing traffic or rejection reasons
- `nak` evidence for expected Nostr event kinds
- hook-tail or daemon log evidence for hook installation/injection
- a clear pass/fail call tied to the feature being tested

## Self-Test Prompt

To validate this skill with a simple agent, give it this task:

```text
Use the tenex-edge-dev skill. Start a fresh local croissant relay on the host,
configure one claude container profile against it using real host Claude auth and
test fabric keys, run the container doctor, launch Claude in a host tmux session
with the cheapest Haiku-class model available, ask it to run or describe
tenex-edge who, then capture the tmux pane, croissant logs, hook-tail output,
and nak relay probes. Do not print any secret or auth file contents. Report
whether the skill worked and include the exact evidence commands/results.
```
