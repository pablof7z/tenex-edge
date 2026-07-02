# tenex-edge isolated agent containers

This harness uses Apple's `container` CLI to run tenex-edge tests and agent
harnesses with isolated state. The source checkout is mounted at `/workspace`;
all generated state lives under `.container-state/<profile>` on the host.

## Commands

```bash
bash containers/tenex-edge/run build-image
bash containers/tenex-edge/run doctor
bash containers/tenex-edge/run test-unit

bash containers/tenex-edge/run codex-login --device-auth
bash containers/tenex-edge/run codex
bash containers/tenex-edge/run codex-ollama

bash containers/tenex-edge/run opencode
bash containers/tenex-edge/run opencode-ollama ollama/qwen2.5-coder:7b
```

`codex-login` stores subscription auth only in `.container-state/codex`.
`codex-ollama` uses `.container-state/codex-ollama`, so local-provider testing
does not share Codex subscription state. OpenCode has the same split between
`opencode` and `opencode-ollama`.

`codex`, `codex-ollama`, `opencode`, and `opencode-ollama` build the current
checkout and run `tenex-edge install --harness <name>` inside the isolated home
before launching the harness. That means Codex hooks and the OpenCode plugin are
installed through the same code path users run on a real machine.

The runner defaults `OLLAMA_HOST` to `http://192.168.64.1:11434`, the Apple
container VM's gateway to the host on this machine. Override it with
`TENEX_EDGE_OLLAMA_HOST` if your setup changes.

The default mount is read-only, so Cargo uses `/state/target` and cannot write
to the checkout. Use `shell-rw` only when you intentionally want a writable repo
from inside the container.

## Isolation Boundary

| Purpose | Path |
| --- | --- |
| Home | `/state/home` |
| Codex config/auth | `/state/home/.codex` |
| OpenCode config | `/state/home/.config/opencode` |
| OpenCode data/cache | `/state/home/.local/share`, `/state/home/.cache` |
| Cargo registry/cache | `/state/cargo` |
| Cargo target | `/state/target` |
| tenex config | `/state/tenex/config.json` |
| tenex daemon/socket/db | `/state/tenex/edge` |

Host Codex/OpenCode config, sessions, plugins, and host tenex daemon state are
not mounted.
