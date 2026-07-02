# tenex-edge isolated agent containers

This harness uses Apple's `container` CLI to run tenex-edge tests and agent
harnesses with isolated state. The source checkout is mounted at `/workspace`;
all generated state lives under `.container-state/<profile>` on the host.

## Commands

```bash
bash containers/tenex-edge/run build-image
bash containers/tenex-edge/run doctor
bash containers/tenex-edge/run test-unit

bash containers/tenex-edge/run claude
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

Live agent testing uses the host's real model credentials/config by default.
This is intentional: Claude, Codex, OpenCode, and tenex-edge distillation should
use the same subscriptions and provider settings as the host while keeping
fabric runtime state isolated per container profile. The runner mounts host auth
directories read-only, symlinks credential files into the isolated container
home, and keeps writable hook config in container state. Set
`TENEX_EDGE_CONTAINER_HOST_AUTH=0` only for non-agent smoke tests.

Claude auth is staged into `/state/home/.claude` because Claude Code may keep
the fresh OAuth credential in the macOS `Claude Code-credentials` Keychain item
while the host JSON file is stale. The runner also sanitizes Claude settings so
container hooks use `/state/target/debug/tenex-edge`, not a host binary path.

`claude`, `codex`, `codex-ollama`, `opencode`, and `opencode-ollama` build the current
checkout and run `tenex-edge install --harness <name>` inside the isolated home
before launching the harness. That means Claude hooks, Codex hooks, and the
OpenCode plugin are installed through the same code path users run on a real
machine.

Tmux-launched live labs can set `TENEX_EDGE_CONTAINER_NAME` and
`TENEX_EDGE_CONTAINER_CIDFILE` so a cleanup script can stop the exact Apple
container if the host tmux pane is killed before the agent exits.

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
| Claude auth/config | host credentials projected into `/state/home/.claude`, hook config copied and sanitized |
| Codex config/auth | host credentials symlinked into `/state/home/.codex` |
| OpenCode config | host credentials symlinked into `/state/home/.config/opencode` |
| OpenCode data/cache | `/state/home/.local/share`, `/state/home/.cache` |
| Cargo registry/cache | `/state/cargo` |
| Cargo target | `/state/target` |
| tenex config | `/state/tenex/config.json` |
| tenex daemon/socket/db | `/state/tenex/edge` |

Host Codex/OpenCode config, sessions, plugins, and host tenex daemon state are
not mounted.
