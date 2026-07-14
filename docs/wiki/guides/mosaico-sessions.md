---
title: Mosaico Sessions Command
slug: mosaico-sessions
topic: mosaico
summary: The unified session management surface is the top-level command `mosaico sessions`
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-07-14
updated: 2026-07-14
verified: 2026-07-14
compiled-from: conversation
sources:
  - session:019f5fec-4248-78b1-8d8f-8aa1238afb9c
---

# Mosaico Sessions Command

## Command Surface

The unified session management surface is the human-only top-level command `mosaico sessions`; agent help hides it.

The `mosaico sessions` command opens an interactive fuzzy picker that shows all alive local sessions and allows selecting one to attach or kill.

Inject and resize are internal runtime mechanisms rather than public session-picker commands. <!-- [^019f5-2e38c] -->

## Picker Navigation

The session picker uses single-cursor navigation with the following controls:

- **Shift+K** — immediately kill the highlighted session with no confirmation.
- **Enter** — attach to the highlighted session when a live PTY exists.
- **Typing** — fuzzy filter the session list.
- **Arrow keys** — move the cursor.
- **Esc** — exit the picker. <!-- [^019f5-3d644] -->

## Session Row Layout

Each session row in the picker is a fixed two-line item: identity/scope on the first line, long title and activity on the second. The picker expands beyond its current half-terminal height so the two-line layout still shows enough sessions. <!-- [^019f5-81b01] -->

## State Badge

The session state badge is a color-only dot indicator with no text labels such as `working`, `idle`, or `PTY`. The badge represents only loop state: busy or idle.

PTY is transport metadata used solely to determine whether Enter can attach; it is never rendered as a session state. <!-- [^019f5-68ecd] -->

## Channel Display

Channels in the session picker are grouped by workspace as `workspace: #channel #channel`, with each workspace grouped under its own deterministic color. Active and passively joined channels are displayed equally without additional markers or labels. <!-- [^019f5-cec0a] -->

## Explicit Operator Kill

Explicit operator Kill (Shift+K) immediately stops the process, marks the session dead, revokes it from every active or joined channel, clears visible status, and reconciles subscriptions, bypassing the ordinary ten-minute cleanup grace period.

If relay revocation fails for one channel after the process is dead, the row is removed, a red warning names the affected channel, and removal is retried automatically until confirmed. <!-- [^019f5-b630c] -->

## Unbound Supervisor Endpoints

Live unbound supervisor endpoints that lack DB enrichment appear in the session picker as minimally enriched attachable rows. Direct endpoint kill is allowed for those rows but reports that fabric cleanup cannot be confirmed without a current identity record. <!-- [^019f5-65fcc] -->
