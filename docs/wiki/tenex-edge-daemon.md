---
title: Tenex-Edge Daemon
slug: tenex-edge-daemon
topic: tenex-edge
summary: tenex-edge uses a single machine-daemon that solely owns state.db, with all CLI calls and session engines acting as thin IPC clients over a Unix domain socket
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-09
updated: 2026-06-09
verified: 2026-06-09
compiled-from: conversation
sources:
  - session:162f9965-82ca-420b-aa24-99faa15cb59a
---

# Tenex-Edge Daemon

## Architecture

tenex-edge uses a single machine-daemon that solely owns state.db, with all CLI calls and session engines acting as thin IPC clients over a Unix domain socket. The single-daemon architecture also collapses per-session relay connections and becomes the clean seam that the channel/streaming-mention adapter consumes. <!-- [^162f9-1] -->

The daemon is spawn-if-absent: the first tenex-edge invocation that finds no daemon forks one to the background bound to a Unix domain socket under TENEX_EDGE_HOME. <!-- [^162f9-2] -->

Startup uses a lockfile plus socket to be race-safe: flock on daemon.lock, winner binds the socket, losers connect; stale-socket files are reclaimed. <!-- [^162f9-3] -->

The daemon idle-exits when no sessions are alive, tracked via session liveness/heartbeats. <!-- [^162f9-4] -->

## IPC Protocol

The IPC protocol is JSON-RPC or length-prefixed JSON lines over the Unix domain socket, with CLI verbs (who, inbox, send-message, turn-start/end) becoming RPCs. <!-- [^162f9-5] -->

## Database

WAL mode is enabled immediately (with busy_timeout and synchronous=NORMAL) as a stopgap while the daemon architecture is built. <!-- [^162f9-6] -->
