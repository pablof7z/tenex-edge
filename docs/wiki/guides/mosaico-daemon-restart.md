---
title: Mosaico Daemon Restart Safety
slug: mosaico-daemon-restart
topic: mosaico
summary: A daemon restart or binary swap must not kill live agent/PTY sessions
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-07-13
updated: 2026-07-13
verified: 2026-07-13
compiled-from: conversation
sources:
  - session:019f5a74-0a91-7340-8299-8ac3dccfa36d
---

# Mosaico Daemon Restart Safety

## Restart and Binary Swap Safety

A daemon restart or binary swap must not kill live agent/PTY sessions. The daemon and every detached PTY supervisor are the same binary, so killing by process name reaps the whole fleet. A daemon restart kills only the daemon process (`pkill -f 'mosaico daemon'`), never `pkill -x mosaico` or a process group/cgroup; the daemon re-adopts still-running supervisors on boot via `reconcile_sessions`. systemd units must set `KillMode=process` so stopping the unit signals only the main daemon PID, leaving detached supervisors alive across the restart. `scripts/reset.sh` is a full wipe (deletes `state.db` and the sessions dir) that deliberately reaps supervisors too; a restart is not modeled on it. <!-- [^019f5-927e4] -->
