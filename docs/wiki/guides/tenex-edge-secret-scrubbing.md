---
title: tenex-edge Secret Scrubbing
slug: tenex-edge-secret-scrubbing
topic: tenex-edge
summary: Secret scrubbing is the mechanism used to avoid leaking secrets in published events that contain user/agent data.
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-12
updated: 2026-06-12
verified: 2026-06-12
compiled-from: conversation
sources:
  - session:1f333238-0710-47f2-bae9-9d5f54b09634
---

# tenex-edge Secret Scrubbing

## Purpose

Secret scrubbing is the mechanism used to avoid leaking secrets in published events that contain user/agent data. <!-- [^1f333-1] -->

## Interception Point

Secret scrubbing intercepts between builder.build() and keys.sign_event() in publish_signed and publish_signed_checked in src/transport.rs. After scrubbing unsigned.content, unsigned.id must be reset to None so nostr-sdk recomputes the event ID over the scrubbed content at sign time. <!-- [^1f333-2] -->

## Scrubbed Patterns

The scrub_secrets function in src/transport.rs covers patterns for AWS access key IDs (AKIA/ASIA/AGPA...), GitHub tokens (ghp_, gho_, etc.), Slack tokens (xoxb-, xoxp-, etc.), Google API keys (AIza...), Anthropic keys (sk-ant-...), OpenAI/generic sk-... keys, Nostr secret keys (nsec1...), PEM private key blocks, and Ollama keys (32-hex.alphanum). The Ollama key pattern matches the format <32-hex>.<alphanum> with a bounded suffix to avoid false positives. <!-- [^1f333-3] -->

## Dependencies

The regex crate (regex = "1") is the only dependency added for scrubbing, using std::sync::OnceLock rather than once_cell. <!-- [^1f333-4] -->
