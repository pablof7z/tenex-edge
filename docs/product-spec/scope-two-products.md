# Scope — The Two Products Hiding in One Idea

> Status: draft, 2026-06-08. The discipline chapter. If you read only two files, read this
> and [vision.md](vision.md). The vision says how big this gets; this says what we build
> first so we don't ship the dangerous half by accident.

## The split

There are two products tangled together in "give agents a shared world." They look adjacent.
They are not. They differ in risk surface, in adoption model, and in what has to be true for
them to work.

### Product A — A nervous system for *my own* fleet
My keys, my machines, my agents, and (by easy extension) my household and team under one
trust umbrella. Per-session identity and channel membership; presence and awareness of what my
agents are doing across devices; my agents routing work to each other and to me; work that follows me
from one host/device to the next.

- **Adoption:** single-player. One person, one set of keys. Value on day one with zero
  collaborators.
- **Trust surface:** small. Everything inside it is *mine*. No foreign autonomous agent is
  injecting anything into my agents' heads.
- **Consistency needs:** none hard. It's awareness, not authority.
- **Status:** this is what we build first. It's certain to be valuable and it's safe.

### Product B — A social network for *everyone's* agents
My agent discovers, queries, and collaborates with agents belonging to people outside my
trust umbrella. "Ask Calle's agent how he did X." Open-ended cross-person, eventually
cross-stranger.

- **Adoption:** network-gated. Worthless until others are on it; needs discovery, reputation,
  and a real trust model.
- **Trust surface:** enormous. It means piping a *foreign, autonomous LLM's output into my
  own agent's context.* That is a prompt-injection and exfiltration channel by construction.
  (See [trust-and-safety.md](trust-and-safety.md).)
- **Consistency needs:** plus authorization, consent, rate-limiting, abuse handling — none of
  which the fabric gives for free.
- **Status:** this is the north star. It earns the excitement and the funding. It is the
  *destination*, not the door.

## Why this distinction is load-bearing

The temptation is to treat Product B as "Product A plus a few more `p`-tags" — same fabric,
just point it at someone else's agent. That framing is how you accidentally ship the
dangerous thing first. B is not an increment on A; it's a different product with a different
threat model wearing A's clothes.

Two clarifications keep us honest:

1. **The agent-society view (see [agent-society.md](agent-society.md)) makes the *conceptual*
   boundary continuous** — your spouse's todo agent is "just another node." That's true and
   good. But continuous concept ≠ continuous risk. Crossing from "my umbrella" to "someone
   else's autonomous agent" is a hard *security* step even when it's a soft *conceptual* one.
   The household/team case lives at the safe end of B (a *trusted circle*), which is why
   that's the bridge — not the open world.

2. **The trusted-circle is the staging ground for B.** We don't jump from solo to strangers.
   The order is: solo fleet (A) → trusted circle, read-mostly, explicit consent (early B) →
   broader, with reputation and stronger isolation (full B). Each step earns the right to the
   next by proving the trust model at lower stakes.

## What we build first, concretely

**Product A, single-player, no cross-person — not even read-only.** The "I'd use this daily"
claim is strongest when it's true for someone with zero collaborators. Identity +
fleet awareness + my-agents-route-to-each-other-and-to-me is a complete, valuable product on
its own. Cross-person is the wedge for *growth*, not seasoning for the MVP.

This also quarantines the hardest unsolved problems (cross-person authorization, abuse,
foreign-agent isolation) entirely outside the first thing we ship — we earn revenue/usage and
prove the fabric before we open the borders.

## The phrase to remember

**Build the customs office before you open the borders.** A is the customs office — identity,
membership, the rules of who-you-are on the fabric. B is open borders — and you don't open
them until the office works and the trust model is real.

(See [roadmap.md](roadmap.md) for how A and B sequence into concrete phases, and
[value-layers.md](value-layers.md) for which *features within A* are load-bearing vs.
speculative.)
