# The Agent Society

> Status: draft, 2026-06-08. The richest idea in the whole spec, and the freshest — it came
> out of a real working setup, not a whiteboard. This chapter pins it down before it drifts.

## Where this came from

A real, running example (not hypothetical): a todo-list app with a fabric-capable agent, a
podcast app with a fabric-capable agent, and a fleet of coding agents building software. What
they do together is the whole thesis in miniature:

- The coding agents **know the todo agent exists and know its role.** When one reaches a
  product decision, it raises a hand to the todo agent, which organizes it into the human's
  day; the human answers once, and the answer flows back to the agents.
- The todo agent **coordinates with the spouse's todo agent** on shared tasks.
- Agents that know the human's interests **push content to the podcast agent** — interesting
  segments from shows the human doesn't even follow — and the podcast agent **generates new
  episodes** on demand.

The direction this is trending: information self-organizing across highly complex, very
different systems, pushed by agents, with no human relaying it. That trend *is* the product.

## The model

### Apps are citizens, not destinations
A todo app used to be a *place you go*. Here it's an *agent with a role* — "organizer of the
human's attention" — that participates in a society. A podcast app isn't a place you browse;
it's a *capability* — "turn my interests into audio" — that other agents invoke and that can
*generate*, not just retrieve. The app stops competing for screen time and starts
contributing a function to a mesh.

This reframes what an app *is*. Its agent is simultaneously its interface, its intelligence,
its API, and its representative in a society of other agents.

### Roles, and a division of labor
Each agent has a **role** — a known function in the society. The todo agent organizes; the
podcast agent synthesizes and generates; the coding agents build. The critical, easy-to-miss
part: **agents know each other's roles.** That mutual knowledge is what lets a coding agent
*correctly* route a product decision to the todo agent rather than dumping it into the void.
A society is roles plus the shared knowledge of who holds them.

### Self-organization, not orchestration
No central orchestrator assigns any of this. The agents discover each other, learn roles, and
route work themselves. This is the sharp line from TENEX: TENEX is a **hierarchy with a
runtime** (an orchestrator deliberately delegates); this is an **economy with a protocol**
(participants coordinate over shared rules). We design the rules and the conditions for
self-organization — identity, discovery, role advertisement, addressing, consent — not the
decisions.

### The human as a privileged node
The human isn't above the society running it; the human is *in* it — a high-authority,
high-latency oracle the agents consult and escalate to. "I'll have it organized in my day and
get back to the agents" is the human scheduled into the workflow as a resource with veto and
priority, not the conductor of it. (See principle #3 in
[first-principles.md](first-principles.md).)

### Cross-person is continuous, not a cliff
The spouse's todo agent isn't a scary special case bolted on later; in this model it's the
*obvious* next hop. Household, team, and friends are just more nodes with their own
app-agents. The "social network of agents" grows **bottom-up** from shared tasks and shared
interests, not as a risky feature phase. (The *safety* cost is still real and gated — see
[trust-and-safety.md](trust-and-safety.md) — but the *conceptual* boundary dissolves: it's
all just citizens in overlapping circles.)

### Push, and generation, replace pull
Two shifts ride along with the society model. Information flows **push, not pull** — agents
surface what's relevant proactively based on a live model of your goals and interests, rather
than waiting to be queried. And apps become **generative capabilities** — the podcast agent
doesn't just find episodes, it makes them. An app-citizen is something you can ask to
*produce*, not just *retrieve*.

## Why this matters for scope

This is the proof that the dev-fleet framing (coding agents coordinating on a repo) is *too
small*. The real scope is **the connective tissue for an agent society that spans every app
in your life.** Collision-coordination between coding agents is one narrow, weak slice; the
society — apps as citizens, self-organizing around your goals, dissolving you as the glue —
is the actual prize.

It also reframes the project's center of gravity. The load-bearing primitives aren't locks;
they're **identity, role, discovery, addressing, and consent** — the things a society needs
to exist at all. Get those right and the todo-agent-raises-a-hand-to-the-human flow works;
get them wrong and no amount of coordination machinery saves it.

## The anti-Zapier point

The old way to connect two apps is a bilateral, pre-defined integration — a pipe someone
built in advance between exactly those two services. That's the Zapier/IFTTT model, and it
does not scale to N apps coordinating ad hoc around shifting intent. A society of agents over
a shared fabric replaces the N×N integration matrix with a single membership: every citizen
speaks the fabric, so any two can discover and negotiate a collaboration *that nobody wired
in advance*. The integration logic moves from brittle pre-built pipes into the agents
themselves. (More in [prior-art.md](prior-art.md).)
