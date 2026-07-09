# Roadmap — Phasing at Altitude

> Status: draft, 2026-06-08. Phasing as *direction*, not a build plan — no mechanics, no
> dates, no tasks. The rule that orders everything: **each rung must deliver real value
> before the next one is needed.** Single-player before network; awareness before authority;
> customs office before open borders.

## The shape

Four rungs, each a complete, valuable resting point. We could stop at any one and still have
shipped something worth using. That property is the whole discipline — it's how we avoid
betting the project on the unproven parts.

```
  Rung 0 ── Rung 1 ──────── Rung 2 ──────── Rung 3 ──────────── (north star)
  prove     make the        cross-agent     cross-person
  the floor fabric real     collaboration   society
  locally   across devices  (own fleet)     (trusted circle → wider)
```

## Rung 0 — Prove the floor, single-player, locally

The thinnest thing that demonstrates the core loop with **zero network risk**: per-session
agent identity + awareness of your own agents on *one device*. This is essentially lifting what
`proactive-context` already does (a local cross-agent awareness board) and giving each session
a signed identity. No relay required, no trust model, no consensus.

- **Delivers:** agents that can see what the others are doing on one machine, each session
  addressable under its own handle.
- **Proves:** the identity + awareness primitives are right before any distributed complexity.
- **Gate to next:** the local loop feels valuable on its own.

*(Optionally, Rung 0 is also where the Q1 collision-frequency experiment runs — passive
logging, no coordination logic — to decide whether Rung 2's coordination features are even
worth building. See [bets-and-open-questions.md](bets-and-open-questions.md).)*

## Rung 1 — Make the fabric real across devices

Lift the same state onto the shared fabric (Nostr) so it spans *your* devices. Work and
context follow *you*, not the machine. Your phone can see what your laptop's agents are doing.
This is where "the host is just a body" becomes tangible — the awareness follows the shared
fabric, not the vessel or the device.

- **Delivers:** cross-device presence and identity; the end of re-explaining yourself to each
  new session/machine; work that follows you.
- **Proves:** the fabric (which already exists — see [ecosystem.md](ecosystem.md)) carries the
  floor reliably across devices.
- **Still:** single-operator. No other person involved. No hard consistency needed.

## Rung 2 — Cross-agent collaboration within your own fleet

Agents knowing each other's *roles* and routing work — to each other and to you. The
coding-agent-raises-a-hand-to-the-todo-agent-which-escalates-to-the-human flow. This is the
first emotionally "alive" moment (the society in miniature) and it's still entirely inside
your trust umbrella.

- **Delivers:** the self-organizing-society feeling, single-operator. Roles, routing, the
  human as the privileged node who gets escalated to.
- **Conditionally includes coordination/locking** — *only if Q1 proved collisions are real*,
  and *only advisory* (git stays authoritative).
- **Proves:** roles + routing work, and the "dissolve the human-as-glue" thesis delivers
  inside one operator's world.

## Rung 3 — Cross-person society (the north star)

Open the borders — carefully, and in order. Trusted circle first (household, team, named
collaborators) with scoped, revocable, default-deny consent and the "peer input is data, not
instructions" boundary enforced and adversarially tested. The spouse's-todo-agent and
ask-Calle's-agent moments. Then, much later and much harder, wider.

- **Delivers:** the full vision — your agents, your circle's agents, and app-citizens, as one
  coordinated mesh.
- **Gated by:** everything in [trust-and-safety.md](trust-and-safety.md). This rung does not
  start until the trust model is real. It is the destination, not the door.
- **Sequenced internally:** solo (Rungs 0–2) → trusted circle, read-mostly → broader, with
  reputation and stronger isolation.

## What each rung deliberately defers

- Rung 0 defers: the network entirely.
- Rung 1 defers: other people entirely.
- Rung 2 defers: cross-person trust entirely; defers coordination *unless* the experiment
  justified it.
- Rung 3 defers: strangers (it starts with trusted circles) and may never fully reach the
  open world.

## The one-line ordering rule

**Customs office before open borders, awareness before authority, single-player before
network — and never let the unproven ceiling delay the certain floor.**
