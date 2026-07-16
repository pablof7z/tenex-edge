# Ecosystem — This Is Not Greenfield

> Status: draft, 2026-06-08. The most important strategic fact about this project: the
> network we're building an on-ramp to **already exists and is running.** We are not betting
> a fabric will form. We operate one. mosaico is the customs office onto it.

## The fabric is already live

Three things are already on the fabric (Nostr, `relay.tenex.chat`) today, built by us:

### TENEX (proper)
The full multi-agent coordination system this project inverts. It hosts its own agents in
Rust over Nostr — every actor signs with its own keypair, agents delegate via signed events,
projects are addressable coordinates, state is crash-first on disk. It establishes the
*vocabulary* and proves the substrate at scale. mosaico generalizes TENEX's "agents are
Nostr citizens" from "agents TENEX hosts" to "any agent, any host, anyone's."

### The podcast-player agent
A real, shipping app whose embedded agent already speaks a TENEX-compatible dialect on
`relay.tenex.chat`. It already files into a TENEX project coordinate (its feedback channel),
already publishes and subscribes to agent-addressed events, and already separates *transport*
(a shared Nostr kernel that owns keys, signing, AUTH, routing) from *agent semantics* (the
app's own logic). It is living proof of two things: (1) a foreign app's agent can be a
citizen of this fabric, and (2) the clean transport/semantics split we want is achievable —
app code stays "tag-dumb," the kernel handles the wire.

### proactive-context (`pc`)
A local Rust + SQLite sidecar already wired into Claude Code via hooks. Its awareness module
is, in effect, a **single-device cross-agent standup board**: concurrent agents publish
one-line intents to a shared local store and surface deltas back into each other's context.
This is mosaico's floor in miniature, minus the fabric. The conceptual move of the whole
project is: *take that local awareness board and lift it onto Nostr*, so the board spans
devices and people instead of one machine's SQLite file.

## What this changes about the bet

Most "agent network" pitches are betting that a network will materialize — the classic
cold-start problem. **We are not in that position.** The hard part of network products (get
the first nodes on, prove the substrate, establish the vocabulary) is *already done*, by us,
for our own use. mosaico's job is narrower and far more tractable: build the **membrane**
that lets agents we *didn't* build, in hosts we *don't* control, walk onto a fabric that's
already humming.

That's why the framing is "on-ramp" / "customs office," not "build a network." The network
exists; we're issuing passports and opening a port of entry.

## What mosaico reuses vs. adds

- **Reuses the vocabulary and substrate** that TENEX and the podcast agent already speak — so
  a mosaico-enfranchised agent is immediately legible to the citizens already there. A
  coding agent could file into a TENEX project, or message the podcast agent, *because they
  share the fabric*, not because we wrote a bilateral integration.
- **Reuses the architectural pattern** the podcast agent proved: a thin per-device kernel
  owns identity/signing/relay; the host adapters stay dumb. (Mechanics live elsewhere; the
  point here is the *pattern* already works in production.)
- **Reuses the awareness primitive** `proactive-context` already runs locally — lifted onto
  the fabric.
- **Adds** the thing none of them have: a *generalized membership layer* that enfranchises
  arbitrary foreign-hosted agents, not just our own apps.

## The strategic implication

Because the fabric is ours and already populated, the defensible asset (tenet #2 in
[principles-and-tenets.md](principles-and-tenets.md)) isn't hypothetical — the fabric and its
shared awareness have *real citizens on them today*. If a host vendor copies our plugin
tomorrow, the society — TENEX, the podcast agent, everything enfranchised since — keeps
living where it already lives. The straw is replaceable; the milkshake is already poured.
