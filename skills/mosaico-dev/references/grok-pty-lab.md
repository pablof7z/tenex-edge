# Grok PTY Lab

Use this proof when testing native Grok hooks, launch ownership, or p-tagged
message injection. It deliberately separates launch-owned identity from hook
command strings.

## Prepare

Start a persistent fresh relay when the command runner reaps descendants:

```bash
MOSAICO_DEV_RELAY_FOREGROUND=1 \
  skills/mosaico-dev/scripts/start-croissant-relay
```

Keep that yielded process running. In another terminal, use its printed env
path and create the Grok profile:

```bash
LAB_ENV=/tmp/.../lab.env
skills/mosaico-dev/scripts/write-container-profiles "${LAB_ENV}" grok
bash containers/mosaico/run --profile grok doctor
skills/mosaico-dev/scripts/launch-agent "${LAB_ENV}" direct grok \
  -p "Respond with exactly OK."
bash containers/mosaico/run --profile grok mosaico channel init
```

Doctor must find Grok auth and `${GROK_HOME}/hooks/mosaico.json`. The direct
check must exit with `OK` before spending time on PTY delivery.

## Launch And Inject

Launch Grok attached in one terminal:

```bash
skills/mosaico-dev/scripts/launch-agent "${LAB_ENV}" launch grok
```

Record the printed handle and keep the TUI visible. In another terminal, read
the live session pubkey from profile state, then publish a kind:9 signed by the
disposable relay-owner identity. This is the generated human identity in the
profile whitelist; the backend key is deliberately not used. The secret is
consumed without printing it:

```bash
source "${LAB_ENV}"
GROK_SESSION_PUBKEY="$(sqlite3 .container-state/grok/mosaico/state.db \
  "select pubkey from sessions where runtime_state='running' and agent_slug='grok' limit 1")"
nak event -k 9 -h workspace -p "${GROK_SESSION_PUBKEY}" \
  --sec "$(<"${OWNER_SK_FILE}")" \
  -c "Reply with exactly LAB-GROK-INJECT." "${RELAY_WS}"
```

The Grok TUI must visibly receive a `<mosaico>` block containing the event. A
relay publish alone is not injection proof.

## Prove Ownership And Delivery

The installed hook surface must contain only Grok commands:

```bash
jq -r '.hooks[][]?.hooks[]?.command' \
  .container-state/grok/home/.grok/hooks/mosaico.json
jq -r 'select(.phase=="received") | .hook.host' \
  .container-state/grok/mosaico/sessions/_unscoped/hook-calls.jsonl \
  | sort | uniq -c
```

Every installed command and received hook host must be `grok`. Correlate the
hook env's `MOSAICO_PTY_SESSION`, `MOSAICO_PUBKEY`, and `GROK_SESSION_ID` with
the launch-owned session. Do not accept an imported Claude hook as evidence.

Correlate the exact event id with daemon delivery evidence. Also inspect the
inbox ledger when that delivery path records a row:

```bash
rg '<event-id-prefix>' .container-state/grok/mosaico/daemon.log
sqlite3 -header -column .container-state/grok/mosaico/state.db \
  "select event_id,target_pubkey,state,delivered_at from inbox \
   where event_id='<full-event-id>'"
```

An inbox row, when present, must be `injected` or `echo_consumed`. Do not use
row presence as the sole pass gate: the decisive injection proof is the same
event visibly arriving as a `<mosaico>` turn in the idle Grok TUI. If Grok
replies, the reply kind:9 must use its session pubkey, p-tag the sender, and
e-tag the injected event.

Capture the relay and event evidence, then clean up:

```bash
skills/mosaico-dev/scripts/probe-lab "${LAB_ENV}" <grok-handle>
skills/mosaico-dev/scripts/cleanup-lab "${LAB_ENV}"
```
