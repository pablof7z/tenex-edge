---
title: tenex-edge Provider Seam
slug: tenex-edge-provider-seam
topic: tenex-edge
summary: The full inventory of wire-shape leaks must be moved behind the provider layer
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-12
updated: 2026-06-14
verified: 2026-06-12
compiled-from: conversation
sources:
  - session:0bc06206-1f30-4e35-8373-f31d0f5c1dcc
  - session:ses_13a5173feffeXR4Fi4UffHR88M
  - session:ses_13a5107b0ffeS3nHRuWFcAx21V
---

# tenex-edge Provider Seam

## Provider Seam Closure

The full inventory of wire-shape leaks must be moved behind the provider layer. Duplicate conflicting RPC implementations exist for `rpc_user_prompt`, `rpc_turn_end`, `rpc_propose`, and `rpc_project_edit` in both `daemon/server.rs` and extracted submodules (`inbox.rs`, `messaging.rs`, `admin.rs`), where the submodule versions bypass the provider/codec seam by building raw Nostr events inline, and the `server.rs` copies that go through the provider are dead code. `connection.rs` dispatch routes to the submodule versions in all four cases, confirming the provider-based `server.rs` implementations as dead code. `rpc_user_prompt` in `src/daemon/server/inbox.rs` (lines 219-284) builds raw `EventBuilder::new(Kind::from(1u16))` with inline `h`, `p`, `session-id`, and `e` tags, bypassing the codec/provider layer. `rpc_turn_end` in `src/daemon/server/inbox.rs` (lines 130-205) encodes via `state.codec.encode()` then publishes via `state.transport.publish_signed()` directly, bypassing the provider's publish path (which includes dual-write). `rpc_propose` in `src/daemon/server/messaging.rs` (lines 309-381) builds `EventBuilder::new(Kind::from(30023u16))` with inline `d`, `title`, `h` tags, hardcoding the kind:30023 Proposal wire shape outside the codec/provider. `rpc_project_edit` in `src/daemon/server/admin.rs` (lines 148-188) builds `EventBuilder::new(Kind::from(9002u16))` inline instead of using the existing `fabric::nip29::lifecycle::group_edit_metadata()` function (note: the `server.rs` version has this inverted — it builds the kind:9002 event inline, while `admin.rs` actually calls `group_edit_metadata()`). The doctor probe in `src/daemon/server/admin.rs` (lines 78-107) and `src/fabric/provider.rs` (lines 229-256) builds a kind:1 event inline with an `h` tag, encoding wire-shape knowledge outside the provider. Six sites build raw `EventBuilder`/`Kind`/`Tag` above the fabric/provider seam: inbox.rs:219 (kind:1 with h/p tags), inbox.rs:130 (`Kind1Codec` direct publish bypassing provider), messaging.rs:309 (kind:30023 inline), admin.rs:148 (kind:9002 inline where lifecycle function exists but unused), admin.rs:78 (kind:1 doctor probe), and runtime.rs (`Presence`/`Status`/`Activity` via `Kind1Codec` directly). The `Codec` trait in `src/codec/mod.rs` fuses three architecturally distinct concerns: wire mapping (encode/decode), subscription model (`SubScope` + `filters()` in `kind1/filters.rs`), and access control (NIP-29 group functions in `kind1/groups.rs`), yet `filters()` and the group functions are not part of the `Codec` trait itself, meaning a new codec can only ever be another Nostr codec. Two parallel codec traits exist for the same concern: `codec::Codec` with `Kind1Codec` (old) and `fabric::WireCodec` with `Kind1WireCodec` (new), where `Kind1WireCodec` is a thin proxy that delegates to `Kind1Codec`, adding no value. The `FabricProvider` trait in `fabric/provider.rs` (lines 90-99) has `#[allow(dead_code)]` and is a documentation shell — all methods are inherent on `Kind1Nip29Provider`. `provider.publish(ev, keys)` is the single wire-publish entry point above the seam. `Proposal` is a first-class domain event with codec arms. `project edit` uses the `nip29` lifecycle module. The only remaining wire construction above the fabric layer is a test oracle. Thread attribution for inbound messages uses the actual thread id from materialization, with the previous heuristic deleted.

<!-- citations: [^0bc06-8] [^ses_1-5] [^ses_1-14] -->
## Task Execution Sequence

The task execution sequence must be: finish the mechanical behavior-preserving rebase, then a single dedicated seam-closing commit for all six leaks, then the three tail bug fixes, then full e2e verification. <!-- [^0bc06-9] -->
