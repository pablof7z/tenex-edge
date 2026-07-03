---
title: Tenex-Edge Landing Page Design
slug: tenex-edge-landing-page-design
topic: marketing
summary: The landing page supports both light and dark themes via token-level custom properties with `prefers-color-scheme` and a `data-theme` override.
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-07-03
updated: 2026-07-03
verified: 2026-07-03
compiled-from: conversation
sources:
  - session:75f62bb9-f564-4633-8741-997dfea1d0e7
---

# Tenex-Edge Landing Page Design

## Theme System

The landing page supports both light and dark themes via token-level custom properties with `prefers-color-scheme` and a `data-theme` override. <!-- [^75f62-3c91e] -->

Terminal panes stay dark in both themes because terminals are inherently dark. <!-- [^75f62-eba3a] -->

## Color Palette

The design uses cool ink neutrals as the base with a warm amber accent serving as the 'live wire / presence signal' and a functional cyan to mark the other vendor's agent in transcripts. The warm-on-cool contrast meaningfully distinguishes two hosts. <!-- [^75f62-eba3a] -->

## Typography

Monospace is used for display and headlines because the headline is literally a terminal line. A humanist system sans is paired for running text. <!-- [^75f62-4bd62] -->

## Layout & Hero Motion

The page uses a single-column layout. The hero consists of two terminal panes showing an @mention traveling from Claude Code into Codex — this is the one orchestrated motion moment, and it is reduced-motion-safe. <!-- [^75f62-50f0c] -->
