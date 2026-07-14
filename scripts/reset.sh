#!/usr/bin/env bash
# Wipe all runtime state (local + remote relay) without touching configs.
# Usage:
#   ./scripts/reset.sh --local-only
#   ./scripts/reset.sh --yes-i-know-this-wipes-the-relay

set -euo pipefail

RELAY_HOST="pablo@157.180.102.242"
MOSAICO_HOME_DIR="${MOSAICO_HOME:-$HOME/.mosaico}"

LOCAL_ONLY=false
case "${1:-}" in
  --local-only)
    LOCAL_ONLY=true
    ;;
  --yes-i-know-this-wipes-the-relay)
    ;;
  *)
    cat >&2 <<EOF
Refusing to reset without explicit confirmation.

This deletes local runtime state under:
  $MOSAICO_HOME_DIR

Options:
  $0 --local-only
      Wipe local state only (db, sessions, sockets). Relay untouched.

  $0 --yes-i-know-this-wipes-the-relay
      Wipe local state AND SSHes to $RELAY_HOST to wipe nip29.f7z.io relay data.
EOF
    exit 2
    ;;
esac

if [[ ! -d "$MOSAICO_HOME_DIR" ]]; then
  echo "MOSAICO_HOME_DIR does not exist: $MOSAICO_HOME_DIR" >&2
  exit 1
fi

# reset.sh is a WIPE tool: it deletes state.db AND the sessions dir, so any
# surviving PTY supervisor would be orphaned against a wiped DB — hence a reset
# reaps the supervisors too. Do NOT copy this kill into a plain daemon *restart*:
# the daemon and every detached PTY supervisor are the SAME binary (`mosaico`),
# so a bare `pkill -x mosaico` reaps live agent sessions along with the daemon.
# A restart must kill ONLY the daemon (`pkill -f 'mosaico daemon'`); the daemon
# then re-adopts the still-running supervisors on boot (reconcile_sessions), and a
# systemd unit must use `KillMode=process`. Target argv explicitly here instead of
# the shared binary name so we never reap an unrelated `mosaico` on the box.
echo "==> Killing local mosaico daemon..."
pkill -9 -f 'mosaico daemon' 2>/dev/null || true
echo "==> Killing local mosaico PTY supervisors (state is being wiped)..."
pkill -9 -f 'mosaico __pty-supervisor' 2>/dev/null || true
sleep 0.5

echo "==> Wiping local state..."
rm -f "$MOSAICO_HOME_DIR/state.db" "$MOSAICO_HOME_DIR/state.db-shm" "$MOSAICO_HOME_DIR/state.db-wal"
rm -f "$MOSAICO_HOME_DIR/daemon.sock" "$MOSAICO_HOME_DIR/daemon.lock" "$MOSAICO_HOME_DIR/daemon.log"
rm -rf "$MOSAICO_HOME_DIR/sessions"
echo "    kept:"
find "$MOSAICO_HOME_DIR" -mindepth 1 -maxdepth 1 -print | sed 's/^/      /'

if [[ "$LOCAL_ONLY" == true ]]; then
  echo "==> Done (local only). Run: mosaico daemon start"
  exit 0
fi

echo "==> Wiping remote NIP-29 relay (nip29.f7z.io) on $RELAY_HOST..."
ssh "$RELAY_HOST" bash <<'REMOTE'
set -euo pipefail
sudo systemctl stop nip29-f7z-io
sudo rm -rf /opt/nip29-f7z-io/data/main \
            /opt/nip29-f7z-io/data/mmmm \
            /opt/nip29-f7z-io/data/global-search \
            /opt/nip29-f7z-io/data/events \
            /opt/nip29-f7z-io/data/mmmm.lock
sudo systemctl start nip29-f7z-io
sleep 1
systemctl is-active nip29-f7z-io
echo "    nip29-f7z-io restarted"
REMOTE

echo "==> Done. Run: mosaico daemon start"
