---
title: tenex-edge Project Add Command
slug: tenex-edge-project-add
topic: tenex-edge
summary: Running `tenex-edge project add` with no arguments resolves the project from the current directory and opens an interactive checkbox selector over locally-avail
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-17
updated: 2026-06-17
verified: 2026-06-17
compiled-from: conversation
sources:
  - session:rollout-2026-06-17T10-28-15-019ed47b-0f0a-7ce3-8928-091aa8f67f69
---

# tenex-edge Project Add Command

## `tenex-edge project add`

Running `tenex-edge project add` with no arguments resolves the project from the current directory and opens an interactive checkbox selector over locally-available agents. Running `tenex-edge project add <project>` opens the same interactive local-agent selector for an explicitly specified project. Running `tenex-edge project add <project> <pubkey>` performs a direct add (put-user) without opening the picker. <!-- [^rollo-94] -->

The interactive picker initializes its checkbox states (checked/unchecked) from the project's existing membership configuration read via a `project_members` RPC. It uses up/down to navigate, space to toggle selection, and enter to confirm, then publishes only the required `project_add` and `project_remove` membership events to reconcile the desired state. <!-- [^rollo-95] -->

Agent removal uses NIP-29 `kind:9001 remove-user` events via a `project_remove` daemon RPC. <!-- [^rollo-96] -->

The picker UI uses a crossterm checklist with stable keys as a local CLI submodule, avoiding new external prompt dependencies. <!-- [^rollo-97] -->
