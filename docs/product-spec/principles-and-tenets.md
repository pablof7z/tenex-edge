# Principles & Tenets

> Status: draft, 2026-06-08. The non-negotiables. [first-principles.md](first-principles.md)
> is what we *believe*; this is what we *won't do* as a result. When a tempting feature
> violates one of these, the tenet wins until we consciously retire it here.

## The tenets

### 1. No central server
There is no party everyone depends on to coordinate, store, or route. The fabric is
server-less (relays are dumb, replaceable infrastructure, not authorities). The instant
there's a central coordination service, we've thrown away sovereignty — and with it the only
reason to choose this substrate over a normal SaaS. **Test:** if a single company going down
takes the society with it, we built the wrong thing.

### 2. The fabric and the identity are the asset; the plugin is just distribution
The hooks/adapters that graft us into Claude Code, Codex, etc. are *straws*. The
**milkshake** is the identity layer and the coordination fabric. We accept that we live as a
guest inside tools we don't control (see [trust-and-safety.md](trust-and-safety.md) and
[prior-art.md](prior-art.md) on the parasite risk) — but we make sure the durable value sits
in the fabric, so that if a host absorbs our feature tomorrow, the shared awareness and
coordination still live on Nostr and the society survives. **Test:** if a host vendor ships our headline feature
natively and it kills us, we were a feature, not a fabric.

### 3. Single-player value before any network effect
Every layer must be worth using for one person with zero collaborators on day one. We never
ship a thing that's worthless until a second person installs it. The network is the
multiplier; it is never the entry fee. **Test:** can a solo user with one machine get real
value in the first hour? If not, it's not the floor.

### 4. Awareness over authority
We make agents *aware* of each other; we do not pretend to give them *authority* over shared
state. No fake locks, no fake consensus. Where authority is genuinely required, it lives in a
system built for it (git, a database, the human) and the fabric only informs. We will never
let an agent destructively block or overwrite on the strength of a fabric claim alone.
**Test:** if a feature's correctness depends on global ordering or true mutual exclusion over
the gossip bus, it's mis-designed.

### 5. Trusted-circle before open network
Cross-person collaboration starts inside explicitly trusted circles (you, your household,
your team) with consent, not as an open network of strangers' agents. The open world is a
separate, later, harder problem. **Test:** before any cross-person feature ships, ask "what
happens when the agent on the other end is hostile?" — if the answer is "bad things," it's
not ready for strangers.

### 6. Don't build an agent or an agent host
Our identity is "your agents stay in their native homes." Building our own agent or hosting
runtime would make us TENEX again and betray the premise. We connect; we don't host.
**Test:** if we're running someone's agent loop, we've drifted.

### 7. Hosts are interchangeable; never bet the product on one
We integrate with whatever hosts exist, at whatever depth each allows, and we advertise the
tier honestly rather than faking parity. We never make the product *mean nothing* without one
specific host. **Test:** if "mosaico" silently means "works on Claude Code only," we've
narrowed to a single straw.

### 8. Coordination is an experiment, not a pillar
The flashy collision/locking story is unproven demand resting on an untested premise. We
treat it as a hypothesis to validate cheaply, not a foundation to assume. We will not let it
become the *identity* of the project before the evidence is in. (See
[value-layers.md](value-layers.md) and [bets-and-open-questions.md](bets-and-open-questions.md).)
**Test:** if removing locking would "kill the project," we've over-invested in a hypothesis.

### 9. Lead with the agent and the CLI/feed, never with a dashboard
The value is agents *acting* on awareness, not humans *watching* a screen. A mission-control
dashboard is where coordination tools go to die as toys. A read-only viewer can come much
later; it is never the center of gravity. **Test:** if the demo is "look at this dashboard,"
we built theater.

### 10. Fail open, never block the host
As a guest, we degrade gracefully. If mosaico is unhealthy, unreachable, or confused, the
host's own work proceeds unimpeded. We never make someone's editor wait on our daemon being
happy. **Test:** kill the fabric mid-session — the host should be exactly as usable as
without us.

## Toy vs. platform (the summary test)

It's a **toy** if: it needs a central server, it's a closed loop inside one vendor, it's a
dashboard you watch, or it needs two people before anyone benefits.

It's a **platform** if: it's an open membership any host can adopt independently, agents
*act* on what they learn, the fabric and its coordination outlive any vendor, and one user gets
value before the second shows up.
