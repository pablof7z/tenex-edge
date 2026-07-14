---
title: Mosaico Skills
slug: mosaico-skills
topic: agent-skills
summary: This guide governs the `mosaico` agent skill written to
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-29
updated: 2026-07-14
verified: 2026-06-29
compiled-from: conversation
sources:
  - session:019f12f9-8a0b-7012-ad2f-f4d0cb035d2b
  - session:019f5fec-4248-78b1-8d8f-8aa1238afb9c
---

# Mosaico Skills

## Overview

This guide governs the `mosaico` agent skill written to
`./skills/mosaico/`, with user-level symlinks pointing to the repo-local
skill directory. Agent self-identity and full awareness live under `my session`;
human `who` is an operator-only terminal view. Targeting uses public session
handles such as `@codex-quill-peak-369`.

<!-- citations: [^019f1-7100b] [^019f1-106a9] -->
## Skill Set

The `mosaico` skill is the sole usage authority for `mosaico`: it teaches day-to-day agent fabric usage — `my session`, channel read/send/wait, dispatch, membership, and channel navigation — and carries the rule to verify live help before trusting docs. Codex global memory must not retain `mosaico` usage or operational guidance; it should hold only codebase-internal engineering context.

The `mosaico` skill exposes two identity scopes: a stable agent identity (e.g. `codex`) and a session identity (e.g. `@nova-codex`), so independent skills can select the appropriate scope without coupling. Skills that manage agent-scoped state should prefer the stable agent identity and avoid using a session identity when an agent identity is available.

The `mosaico-dev` skill teaches hook wiring and debugging for Codex, Claude Code, and OpenCode, including how to prove agent registration with `my session`.

The `mosaico-verification` skill teaches local gates and test tiers: `just fmt-check`, `just loc-check`, `just lint`, `just test-unit`, when to use `cargo test`, when relay/croissant/nak are required, and how to run e2e safely. Because command and docs drift is already visible, this skill includes small scripts/resources such as a command that prints current help, LOC offenders, and known test prerequisites.

The `mosaico-docs-queue` skill teaches repository discipline: GitHub Issues are the tactical queue, no new planning files, correct docs in place, classify generated wiki output, and retire stale planning material.

The `mosaico` skill itself is resource-free: no `reference/` files or scripts are included, only the mental-model guidance and a small mechanics appendix.

<!-- citations: [^019f1-9c16f] [^019f1-f17ce] [^019f5-33e51] -->
## Agent Safety

Some open issues are marked high-risk or agent-unsafe. Skills must teach agents when to proceed autonomously and when to stop for architecture or owner review. <!-- [^019f1-0029b] -->

## Skill File Discipline

Each skill keeps `SKILL.md` short and puts volatile details in `reference/` files. <!-- [^019f1-ad217] -->
