---
title: Tenex-Edge Hook XML
slug: tenex-edge-hook-xml
topic: tenex-edge
summary: The hook XML agents see always emits canonical absolute channel paths, never relative refs, so agents can safely copy them into commands from any workspace.
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-07-14
updated: 2026-07-14
verified: 2026-07-14
compiled-from: conversation
sources:
  - session:019f606f-1d25-76d1-95da-a7bb383cbe6d
---

# Tenex-Edge Hook XML

## Channel Paths

The hook XML agents see always emits canonical absolute channel paths, never relative refs, so agents can safely copy them into commands from any workspace. <!-- [^019f6-a1664] -->

Nested channels are represented as nested `<channel>` elements with an `id` attribute carrying the absolute path (e.g. `id="/tenex-edge/dev/api"`) and a `name` attribute carrying the short channel name (e.g. `name="#api"`). <!-- [^019f6-a722a] -->

## Workspace Element

The workspace element in hook XML omits the redundant `name` attribute and uses only the `channel` attribute to identify the workspace scope. <!-- [^019f6-20094] -->

## Presence

Hook XML uses `<presence>` containing `<session>` elements for presence display, replacing the former `<recent-presence>` wrapper with `<status>` elements. Presence `<session>` elements use `name` for the agent ref and `status` for the status text, replacing the former `ref` and `text` attributes. <!-- [^019f6-0aa78] -->

## Removed Attributes

The `seen` attribute is removed entirely from presence and member rows in hook XML, including the suppression of any "just now" value. <!-- [^019f6-e4a61] -->

## Retained Attributes

Message `age` is retained in hook XML because it still conveys useful chronology. <!-- [^019f6-cbada] -->
