---
title: Tenex-Edge Who Output
slug: tenex-edge-who-output
topic: tenex-edge
summary: "The `tenex-edge who` output includes a `Project: <name>` header."
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-16
updated: 2026-06-16
verified: 2026-06-16
compiled-from: conversation
sources:
  - session:a88513d3-754f-4369-b440-72c8d29331e2
---

# Tenex-Edge Who Output

## Project Header

The `tenex-edge who` output includes a `Project: <name>` header.

<!-- citations: [^a8851-1] [^a8851-7] -->
## Sessions

Active sessions are displayed under a `# Sessions` heading with the instruction: 'Active sessions — message one with `tenex-edge inbox send --to <agent@project|session-id> --subject "..." --message "..."`.' Sessions are rendered as a markdown table with columns: Agent, Session, Host, Title, Status. In the agent-facing rendering, an empty session title is shown as an em-dash (—).

<!-- citations: [^a8851-2] [^a8851-8] -->
## Agents

Available agents for new sessions are listed under a `# Agents (for new sessions)` heading with the instruction: 'Start a new session with `tenex-edge inbox new-session --agent <slug>`.' The `[spawnable via …]` suffix is removed from agent listings; only the slug@host format is shown, hiding the underlying command/harness distinction.

<!-- citations: [^a8851-3] [^a8851-9] [^a8851-14] -->
## Other Projects

Other projects are listed under a `# Other projects` heading, showing only project names. The project `about`/description rendered in the old format is dropped.

<!-- citations: [^a8851-4] [^a8851-10] [^a8851-15] -->
## Section Order

The section order in the who output is: Sessions, then Agents, then Other projects. <!-- [^a8851-5] -->

## Open Questions

- Whether to keep ANSI/colors inside the markdown table cells or go plain markdown is pending confirmation.

<!-- citations: [^a8851-6] [^a8851-12] -->
## Renderers

The `who` command produces two distinct renderings: a human-facing format (colorized, compact, aligned columns) and an agent-facing format (no ANSI, markdown structure with headings, tables, and embedded command instructions). Renderer selection is automatic based on TTY: terminal output uses the human renderer, while piped or captured output uses the agent markdown renderer. The turn-start fabric injection block uses the agent-facing markdown renderer with a one-line lead-in. <!-- [^a8851-11] -->
