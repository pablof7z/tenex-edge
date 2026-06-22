# CLAUDE.md

This file intentionally defers to [`AGENTS.md`](AGENTS.md), the canonical contributor guide for this repository. Every rule that applies to agents applies to Claude (and vice versa). Keeping both files in sync would violate the repository's own single-source-of-truth rule — so `AGENTS.md` is authoritative and this file is a pointer.

## TL;DR

- **File size** — soft limit 300 LOC, hard ceiling 500 LOC for hand-authored files. Split along domain boundaries; move inline tests into nested test modules; use narrow visibility (`pub(super)`/`pub(crate)`) for extracted surfaces. See [`AGENTS.md`](AGENTS.md#file-size).
- **Planning** — GitHub Issues is the only tactical queue. No scattered `TODO.md`/`ROADMAP.md`/plan files; correct docs in place; never commit code reviews. See [`AGENTS.md`](AGENTS.md#planning-discipline--github-queue-temporal-files-no-duplicate-plans).
