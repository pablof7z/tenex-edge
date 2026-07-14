---
title: Mosaico Daemon
slug: mosaico-daemon
topic: mosaico
summary: Daemon `cleanup()` does not delete the lock file, so the flock persists on the same inode until the old daemon process exits
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-29
updated: 2026-07-12
verified: 2026-06-29
compiled-from: conversation
sources:
  - session:b07a57a3-67a1-4c44-a8fc-58a1bb97860a
  - session:47f3cac2-1ad9-461c-8ac0-3ea341d0e962
  - session:38650a40-2fcc-452f-9b6a-9250a9c76c95
  - session:75f62bb9-f564-4633-8741-997dfea1d0e7
---

# Mosaico Daemon

## Lock File and Cleanup Behavior

Daemon `cleanup()` does not delete the lock file, so the flock persists on the same inode until the old daemon process exits. This prevents a two-daemon race over state.db.

The `daemon.inhibit` file is the `mosaico daemon stop` mechanism that prevents hooks from respawning a daemon the user explicitly killed. Any non-hook `mosaico` command clears the file at dispatch time so hooks resume working after a stop.

<!-- citations: [^b07a5-50304] [^38650-4ff91] [^38650-1f1f4] -->
## Command Surface and Output Modes

The `mosaico daemon` command is exposed as a visible subcommand and is the command used by the auto-spawner. It runs in the foreground and produces colorized output on stdout while simultaneously writing the same output to a daemon.log file. When stdout is not a terminal (detached daemon), a single plain-text layer is used instead of dual ANSI-stdout plus file output. <!-- [^47f3c-0c97b] -->

## Daemon Model and Concurrency

The mosaico daemon is a single per-machine process owning SQLite and the one relay connection, using flock'd startup and UDS RPC, explicitly designed to fix a multi-writer corruption class. <!-- [^75f62-c75e2] -->
