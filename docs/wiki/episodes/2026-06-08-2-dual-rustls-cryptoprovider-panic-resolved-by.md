---
type: episode-card
date: 2026-06-08
session: f3a730bf-9a3b-4952-b687-c93ade5fd7ec
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/f3a730bf-9a3b-4952-b687-c93ade5fd7ec.jsonl
salience: root-cause
status: active
subjects:
  - rustls-crypto-provider
  - tls-initialization
supersedes: []
related_claims: []
source_lines:
  - 4783-4789
  - 4829-4860
captured_at: 2026-06-12T19:51:37Z
---

# Episode: Dual rustls CryptoProvider panic resolved by installing ring default

## Prior State

A single crypto provider (ring via nostr-sdk) was implicit; rustls auto-selected it without conflict.

## Trigger

Adding rig-core pulled reqwest → hyper-rustls → aws-lc-rs alongside the existing ring, causing rustls 0.23 to panic at any TLS handshake: 'Could not automatically determine the process-level CryptoProvider' (line 4783-4786).

## Decision

Install ring as the explicit default CryptoProvider at process startup in main() before any TLS connection: rustls::crypto::ring::default_provider().install_default(). Also added rustls = { version = '0.23', features = ['ring'] } to Cargo.toml.

## Consequences

- All TLS connections (relay wss://, OpenRouter HTTPS, ollama HTTPS) consistently use ring
- Any future dependency that brings another crypto provider won't cause a panic
- doctor and inbox commands (which open fresh TLS connections) work again

## Open Tail

*(none)*

## Evidence

- transcript lines 4783-4789
- transcript lines 4829-4860

