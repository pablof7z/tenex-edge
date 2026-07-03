---
title: Croissant Test Relay
slug: croissant-test-relay
topic: tenex-edge
summary: Croissant is a local NIP-29 relay binary used for fully isolated, writable test relay environments
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-07-03
updated: 2026-07-03
verified: 2026-07-03
compiled-from: conversation
sources:
  - session:abce9e9f-8f3e-4561-9dd3-684afd59be80
  - session:a685f611-39bd-4a18-a6b7-ea4e38334b82
---

# Croissant Test Relay

## Overview

Croissant is a local NIP-29 relay binary used for fully isolated, writable test relay environments. The deployed instance is `nip29.f7z.io`, running on host `pablo@157.180.102.242` (host `kind2`) at WorkingDir `/opt/nip29-f7z-io` on port 3336.

During the daemon startup burst, the croissant relay never received or rejected the daemon's publish events. Its logs contain no reject, write-action, disabled, or OK-false lines, and no incoming EVENT traffic from the daemon's bursts appears at all.

<!-- citations: [^abce9-769ed] [^a685f-4648a] -->
