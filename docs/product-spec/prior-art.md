# Prior Art & Positioning

> Status: draft, 2026-06-08. Where this sits against everything adjacent, and what the moat
> actually is. This chapter will be sharpened by the SOTA + X/Twitter research now in flight;
> treat the specifics as provisional and the *framing* as the durable part.

## The honest summary

The pieces this idea is made of mostly exist. Agent-to-tool calling, agent-to-agent
protocols, distributed locks, presence systems, multiplayer state, and "who's editing what"
are all solved or being solved by someone. What does *not* exist is the specific
**combination**: cross-*person* (not just cross-org), server-less, key-owned, vendor-
independent agent identity that lets agents you didn't build, in hosts you don't control,
become citizens of a shared society. The moat is narrow but real, and we should claim exactly
that and not more.

## The landscape

### MCP (Model Context Protocol)
Owns agent↔*tool*. It's how an agent reaches capabilities. It is *not* agent↔agent, not
identity, not cross-person, not a society. Complementary, not competitive — an enfranchised
agent might still use MCP to reach its tools. MCP is plumbing *below* us.

### Google A2A (and the agent-interop standards crowd)
The closest competitor in spirit: agent↔agent interop, with signed identity, now under
foundation governance and real enterprise adoption. **What it gives that overlaps us:**
cryptographic agent identity and cross-agent messaging. **What it structurally can't serve:**
it assumes *organizations*, discovery infrastructure, and registries — the enterprise frame.
It is not built for *two individuals with no shared org and no central registry* coordinating
their personal agents over a substrate they each own. That individual, sovereign,
no-central-anything case is precisely Nostr's home turf and precisely our niche. We don't
out-feature A2A; we serve the case its assumptions exclude.

### "Zapier / IFTTT for agents"
The bilateral-integration model: pre-built pipes between specific services. It does not scale
to N apps coordinating ad hoc around shifting intent (the N×N problem). Our answer is the
anti-Zapier: a single *membership* replaces the integration matrix — any two citizens
discover and negotiate a collaboration nobody wired in advance (see
[agent-society.md](agent-society.md)). If "Zapier for agents" wins, it wins the *predefined*
workflows; we win the *emergent* ones.

### CRDTs / multiplayer (Yjs, Liveblocks, et al.)
Far better than a gossip bus at *real-time shared mutable state*. But that's a different
problem — co-editing a document, not enfranchising heterogeneous agents with durable
identity. If we ever needed authoritative real-time shared state, we'd reach for these; we
mostly don't, because we're awareness-over-authority (principle #4).

### Distributed locks / consensus (ZooKeeper, etcd, Redis locks)
The right tools when you genuinely need *authority* over shared state. We deliberately don't
compete here — we defer authority to systems built for it (git, databases, the human) and
keep the fabric advisory. Invoking these is an admission you've left our problem space.

### git
The quiet incumbent for the collision/locking scenario. Branches isolate, merge detects,
conflict markers force resolution — git is the *authoritative* "who changed what" layer. Any
collision feature we build is *pre-collision UX* on top of git's *at-merge truth*, and has to
justify itself against git already doing the hard part. This is a major reason coordination is
a ceiling-feature-on-probation, not a pillar (see [value-layers.md](value-layers.md)).

### TENEX
Our own predecessor and the thing we invert. Not competition — substrate and proof. (See
[ecosystem.md](ecosystem.md).)

## Why hasn't "this" been done?

Because the obvious adjacent versions get built for the obvious markets — A2A for
enterprises, MCP for tools, Zapier for predefined automation — and the specific niche
(individuals' personal agents, sovereign identity, no central anything, cross-person) is
small, weird, and only reachable if you already believe in a server-less cryptographic
substrate. We're unusually positioned to serve it because **we already operate the fabric and
already have citizens on it** (ecosystem), which is the part everyone else would have to
cold-start.

## The moat, stated as one sentence

**Cross-person, server-less, key-owned, vendor-independent agent citizenship** — the durable
identity and shared society that A2A's org-assumptions, MCP's tool-scope, Zapier's predefined
pipes, and git's at-merge model each structurally can't provide.

## The risks this framing must respect

- **A2A momentum.** If the interop standard becomes ubiquitous, our differentiation collapses
  to "the individual/sovereign case" — which we must own hard rather than pretending to a
  broader win. (Bridging *to* A2A may be smart, not fighting it.)
- **Host absorption.** A host vendor shipping native cross-session/cross-device coordination
  eats our floor's flashiest part. Mitigation is tenet #2: own the fabric/identity, not the
  feature.
- **"Is Nostr right or a constraint dressed as a feature?"** A real question to keep
  answering. The defense: the sovereignty/no-central-anything property *is* the product, and
  Nostr is the substrate that gives it for free. If that property stops being the point, the
  substrate choice should be revisited.

*(The research in flight will populate the specific products, quotes, and frontier/gap
analysis behind this framing.)*
