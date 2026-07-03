---
title: Copy Generation Pipeline
slug: copy-generation-pipeline
topic: marketing
summary: The copy-generation pipeline is a two-stage marketing workflow
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

# Copy Generation Pipeline

## Pipeline Overview

The copy-generation pipeline is a two-stage marketing workflow. The first target of the effort is to rewrite README.md from scratch and land high-level copywriting for the landing page, anchored in uniqueness, USP, and best copywriting practices. Stage one runs a swarm of four parallel Sonnet research agents — code-reality, docs-vision, market-research, and audience-copy — each investigating a distinct facet of the product. Stage two commissions a Fable agent for high-level ideas, direction, and concepts; the Fable pass synthesizes across all four research agents' outputs rather than running concurrently with them. The Fable agent then reviews all copywriting — README, landing page, and CLI/Cargo taglines — against the ~/.agents/skills/copywriting skill. The product-spec docs are explicitly working drafts — positions to argue with, not shipped-feature claims — and copy must not treat them as such.

<!-- citations: [^75f62-2c069] [^75f62-debc4] [^75f62-be4f1] -->
