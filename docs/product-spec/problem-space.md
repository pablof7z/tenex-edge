# Problem Space

> Status: draft, 2026-06-08. The pains we're addressing, in real terms. Each entry notes
> whether it's **acute** (people feel it daily, today) or **aspirational** (a problem that
> only becomes visible once you imagine the better world). We build down the acute list
> first; the aspirational ones are the pull of the vision.

## The root pain: the human is the integration layer

Everything below is a symptom of one disease. Software is siloed, and *you* are the bridge
between the silos. You carry context, decisions, and information from one tool to the next by
hand. You are a slow, lossy, expensive message bus — and you don't even get to be a good one,
because you forget, you mistype, and you're asleep half the day.

The product is, at root, the automation of *this job* — the job of being your own
integration runtime.

## The pains

### 1. The human-as-glue tax — **acute**
You copy-paste output from one AI tool into another. You hand-carry a product decision from a
coding session into your todo list. You relay "what's in flight" between your own tools.
Every cross-tool workflow is paid for in your attention and your transcription. This is the
headline pain and the one the vision most directly removes.

### 2. Amnesiac, non-portable agents — **acute**
Agents forget everything between sessions. Context is trapped in one vendor's tool. Open a
new agent and you re-explain your project, your preferences, what's in flight — again. Nothing
carries the shared working context from one session or host to the next, so every session
starts cold and you are the only thread of continuity.

### 3. App silos in the agent era — **acute, and worsening**
Every app is shipping its own walled-garden assistant. Your todo app's AI can't talk to your
podcast app's AI can't talk to your coding agent. We are re-building the silo problem one
layer up — except now the silos are *intelligent* and *still can't cooperate*. The agent era
is making the integration problem worse, not better, absent a shared fabric.

### 4. No cross-vendor / cross-app interop — **acute for power-users, aspirational for most**
There is no common way for an agent in one app, built by one vendor, to discover, address,
and collaborate with an agent in another. Today's "integration" is bilateral, brittle, and
pre-defined (the Zapier model) — which doesn't scale to N apps × N apps × ad-hoc intent.
People who run multiple agents already feel this; everyone else will once they have more than
one agent worth connecting.

### 5. Multiple agents, zero shared awareness — **acute for the dev power-user, narrow**
Running Claude Code *and* Codex *and* Cursor, you have no shared picture of what they're
each doing, across machines. Sometimes they duplicate work or step on the same files.
*(Honest caveat: this is the pain the dev-fleet framing over-indexes on. It's real but
narrow, and partly already handled by git. See [value-layers.md](value-layers.md) and the
collision-frequency question in [bets-and-open-questions.md](bets-and-open-questions.md)
before treating it as central.)*

### 6. Pull, never push — **aspirational**
You only get value from a tool when you go to it and ask. Nothing reaches across your systems
to surface what's relevant *before* you think to look — the interesting podcast segment from
a show you don't follow, the decision that's now unblocked, the task your collaborator just
touched. The better world is ambient and anticipatory; today's is strictly on-demand.

### 7. No cross-person / household / team agent coordination — **aspirational, high-value**
Your agent can't safely talk to your wife's agent about a shared task, or your teammate's
agent about a shared project. The natural unit of a lot of life is a small trusted circle,
and there is no fabric for those circles' agents to cooperate over. (This is the most
exciting pain and the one with the biggest safety cost — see
[trust-and-safety.md](trust-and-safety.md).)

### 8. No provenance or trust for agent work — **aspirational, becomes acute fast**
As agents produce more of everything, "which agent, under whose authority, produced this,
and can I trust it?" goes unanswered. There's no signed, durable record. Today this is a
shrug; in a year, with agents writing most code and most messages, it's a crisis.

## What we are *not* solving

- We are not making agents smarter. Capability is the hosts' job; we make agents *connected*.
- We are not solving authoritative shared-state consistency. Where that's needed, defer to
  systems built for it (see principle #6 in [first-principles.md](first-principles.md)).
- We are not, first, solving open-world trust between strangers' agents. We solve the
  trusted-circle case; the open world is a later, separate problem.

## Priority read

The certain, build-now pains are **1, 2, 3** — the human-as-glue tax, amnesiac agents, and
app silos. They're acute, they're felt single-player, and they're directly addressed by the
floor of the product (per-session identity + awareness + cross-host messaging). Pains **6, 7, 8**
are the pull of the vision and the reason to keep going. Pain **5** is the seductive demo we
must not let define the project.
