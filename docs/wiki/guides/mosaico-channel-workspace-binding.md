---
title: Mosaico Channel Workspace Binding
slug: mosaico-channel-workspace-binding
topic: mosaico
summary: A project is a channel that owns a workspace binding; "project" is the role a node plays when it carries a workspace, not a separate concept or tree position
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-07-03
updated: 2026-07-03
verified: 2026-07-03
compiled-from: conversation
sources:
  - session:bdb6c341-4dd4-48e7-9764-e80242beb005
---

# Mosaico Channel Workspace Binding

## Workspace Binding Model

A project is a channel that owns a workspace binding; "project" is the role a node plays when it carries a workspace, not a separate concept or tree position. The channel node type has no subtype or enum (no `kind: project | channel | session`); the workspace binding is one optional field on the node, not a subtype, a different table, or a different code path. Workspace binding resolves by nearest ancestor: an agent in a descendant channel walks up the tree to the nearest workspace-bearing ancestor to find its repo, with root binds and descendants inheriting. Arbitrary nesting depth is a capability the model supports, but the product does not promote deep nesting as a feature; two levels (root plus task rooms) covers typical use. <!-- [^bdb6c-ce0e4] -->

## Relay Contract and Migration Scope

The relay contract is unchanged in this refactor: every channel remains a NIP-29 group with a parent hint, and the migration is purely local daemon, CLI, and hook code replacing branches on "is this a project or a channel" with "does this node have a workspace binding" or "is parent empty." The refactor does not include a big-bang rename of `relay_*` tables; those are relay-sourced projections and are left as-is, making this a local-state and rendering refactor. <!-- [^bdb6c-21416] -->

## Tracking

The channel→project architecture refactor is tracked as GitHub issue #201 in the mosaico repo, labeled `refactor:architecture`, `needs-human-policy`, and `risk:high`. <!-- [^bdb6c-b8501] -->
