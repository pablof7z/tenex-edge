---
type: research-record
date: 2026-07-03
session: 75f62bb9-f564-4633-8741-997dfea1d0e7
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/75f62bb9-f564-4633-8741-997dfea1d0e7.jsonl
source_lines: 1100-1195
agent_attribution: fable-review
has_preregistered_criteria: true
has_method: true
has_structured_report: true
characterization: "Structured editorial copy review against pre-registered copywriting-skill rubric, scoring hero 8.5/10 with prioritized P0/P1/P2 line-by-line findings and a top-5 ranked verdict"
captured_at: 2026-07-03T09:16:04Z
---

Structured editorial copy review against pre-registered copywriting-skill rubric, scoring hero 8.5/10 with prioritized P0/P1/P2 line-by-line findings and a top-5 ranked verdict

---

<teammate-message teammate_id="fable-review" color="pink" summary="Full six-section copy review of README/landing/CLI">
Editorial review — tenex-edge README + landing.html + CLI surfaces. Rubric: /Users/pablofernandez/.agents/skills/copywriting/SKILL.md + both references (read in full).

=== 1. VERDICT ===

This copy will convert for the agent-power-user audience — unusually disciplined for developer marketing: proof-first hero, named competitors, honest scoping, real numbers. GREATEST STRENGTH: the proof architecture (transcript → "374 green" → "verified live on relay.tenex.chat" → named-competitor table) makes every claim falsifiable, which is exactly what a hype-allergic dev rewards. BIGGEST WEAKNESS: the trust/privacy objection is unhandled — the product broadcasts LLM-distilled transcript summaries to a relay and delivers remote text into your agent's live prompt, and the copy never says who can see or send what. Worse, the one line gesturing at it ("Nothing leaves your keys") reads as false on inspection. For this audience that's the conversion killer.

=== 2. HERO / ABOVE THE FOLD — 8.5/10 ===

Judged against the skill's Above-the-Fold framework (headline = single most important message; subheadline adds specificity; CTA communicates what you get):

- Proof-first works. The two-terminal transcript + beat line is show-don't-tell at the structural level (skill principle 5, "Show over tell"). The eyebrow ("Identity & awareness fabric for coding agents") supplies the category so the proof isn't context-free.
- Headline "Stop being the message bus between your own agents." matches the skill's problem-focused formula ("Stop [pain]. Start [pleasure]."), uses the customer's own mental model, and "your own agents" twists the knife. Keep it. Optional full-formula second beat: "Start @-mentioning them." — only if it doesn't crowd the lede.
- Lede is one sentence too long; "You stop hand-carrying." dangles its object (skill: Clarity Over Cleverness). Fix: "You stop hand-carrying context."
- CTA "Get it on GitHub →" paired with the literal `$ tenex-edge install --all` snippet is honest and adequate; the snippet does the "what you get" work.
- The one real above-the-fold gap: the reader who just watched a message get injected into Codex's terminal immediately thinks "…so anyone can inject turns into my agent?" Nothing on the landing answers it. One clause fixes it — P0-3 below.

=== 3. LINE-BY-LINE FINDINGS (prioritized) ===

--- P0, fix before launch ---

P0-1. Landing footer lede: "Nothing leaves your keys."
Problem: vague to the point of reading false — keys don't leave, but activity lines, presence, and messages DO leave, to a relay. Violates: Honest over sensational + Specificity over vagueness.
Rewrite: "Your keys never leave your disk. Your agents' traffic goes only to relays you choose." — true, and stronger because precise.

P0-2. CLI about-string + Cargo description violate locked constraints (d) and (e).
src/cli/args.rs:14 — "Citizenship for your agents: identity + awareness on the Nostr fabric."
Cargo.toml:5 — "Citizenship for your agents: durable identity + awareness on the Nostr fabric, host-neutral."
`--help` is a FIRST-TOUCH surface — seen seconds after install, before "How it works." Both lead with the demoted metaphor (constraint e: exactly one earned use) and name Nostr before any property framing (constraint d). Also breaks the skill's consistency mandate (Voice and Tone: "Maintain consistency").
Aligned wording — CLI about: "An identity and awareness fabric for the coding agents you already run." Cargo description: "Identity and awareness fabric for coding agents: durable, self-owned identity and live @mentions across Claude Code, Codex, OpenCode, and Grok." (Put nostr/nip29 in Cargo `keywords` for crates.io discoverability, not the description.)

P0-3. The trust-model objection is never handled.
"@mention delivered as a real turn" is, framed adversarially, remote prompt injection into your agent — and this audience WILL frame it that way. `whitelistedPubkeys` appears once as raw config, never as a promise. Violates: the skill's Objection Handling core section (Page Structure Framework).
Fix: one clause in the @mention feature cell — "Only keys you whitelist can reach your agents." — plus a README FAQ entry: "Can anyone message my agents?" → "No. The daemon only acts on events signed by pubkeys you whitelist in ~/.tenex-edge/config.json." Same treatment for the activity line: one sentence on which LLM distills, whose tokens, where the sentence is published.

P0-4. Quickstart clone URL is SSH: `git clone git@github.com:pablof7z/tenex-edge.git` (README lines 23 and 156).
The copy-paste conversion path fails for anyone without GitHub SSH keys — friction at the exact moment of conversion. Skill: Be Direct / reduce friction at the CTA.
Rewrite: `git clone https://github.com/pablof7z/tenex-edge.git`.

--- P1, fix soon ---

P1-5. The README has no headline. Its "headline" is the repo name; the money line ("You stop being the message bus.") is buried as the last sentence of a dense paragraph (README:20). Skill: headline = "your single most important message." Fix: after the transcript caption, open with bolded "Stop being the message bus between your own agents." then the paragraph.

P1-6. README intro paragraph does too much (lines 15–20: pain + category + three features + payoff in one block). Skill: One Idea Per Section / "Sentences trying to do too much?" Split: pain sentence; then "tenex-edge is an identity and awareness fabric for the agents you already run." standalone; then the three capabilities.

P1-7. Passive voice on the flagship feature, twice. README:62 + landing cell 2: "the running transcript is distilled by an LLM into one plain sentence." Skill: Active over passive. Rewrite: "An LLM distills each turn into one plain sentence — 'reworking the auth migration' — and broadcasts it."

P1-8. README:134: "This audience is owed the boundary, so here it is, plainly." Talks ABOUT the reader in third person, to the reader. Skill: Customer Language / directness. The landing already has the right version — port it: "You're owed the boundary. Here it is."

P1-9. README:138–139: "Opening the borders before the customs office works is how you ship the dangerous thing first." Convoluted negation-of-a-negation; "the dangerous thing" is vague. Skill: Clarity Over Cleverness. The landing version is the fix: "You build the customs office before you open the borders."

P1-10. Landing has zero objection-handling section. Structure runs pain → ships → identity → how → isn't → CTA. "What this isn't" handles SCOPE objections brilliantly, but the skill's strong-page template (copy-frameworks: "Varied, Engaging Page") wants FAQ/objections before the final CTA. A three-question strip (who can message my agents / which LLM & whose tokens / what if the relay dies) or a "Common questions →" link to the README FAQ closes it.

--- P2, polish ---

P2-11. Table microcopy "❌ registered names" (README:102) / "named" (landing). Cryptic — the reader can't tell why "named" is a ✕. Rewrite the cell: "central registry" — that's the actual contrast with "keys on disk."

P2-12. Landing h2: "The hooks are the straw. The fabric is the milkshake." The skill endorses analogies, and in the README the metaphor lands because the explaining bullet is adjacent. As a cold section heading it's a riddle before the referent. Demote it to the first body line; promote something plain to h2 — e.g. "Hosts adapt to it — not the other way around."

P2-13. CTA "Get it on GitHub →" is passable per the skill's CTA formula (verb + thing) and honest for an OSS tool. Slightly more pull without hype: "Clone tenex-edge →". Low stakes; the adjacent install snippet is the real CTA.

P2-14. Landing: "Every claim here is running, and tested." The comma splices the rhythm. → "Every claim here runs — and is tested." or "Everything here runs, today."

P2-15. Harmonize the pain headline: README "The problem is the wire, and the wire is you" vs landing "The problem is the wire. The wire is you." The landing's two-sentence version is punchier — adopt in README.

P2-16. Metaphor drift: README:42 "the same thing wearing two hats" vs landing "the same thing twice." Pick one; "the same thing twice" is cleaner.

P2-17. Landing table drops hcom's gloss (README has "hcom (hook-based messaging)"). Most readers won't know it — keep the parenthetical or a <small> gloss; the table's credibility rests on fair characterization of competitors.

(Noted, not a violation: "relay.tenex.chat" / "relay connection" appear above How-it-works. "Relay" isn't on the banned list and a hostname functions as proof; acceptable — but it is the one piece of transport jargon that leaks early.)

=== 4. CONSISTENCY ===

- README ↔ landing: tight. Same transcript, table, voice. Only drift: wire-headline punctuation (P2-15), "two hats" vs "twice" (P2-16), hcom gloss (P2-17), and the landing's better "isn't" intro not backported (P1-8, P1-9).
- CLI about-string: VIOLATES constraints (d) and (e) — leads with "Citizenship" + "Nostr" on a first-touch surface. Fix per P0-2.
- Cargo.toml description: same double violation. Fix per P0-2; move nostr to keywords.
- Recommended aligned one-liner for all short surfaces: "An identity and awareness fabric for the coding agents you already run." It's the eyebrow, the category claim, and the positioning statement in one, and keeps "citizenship" exclusively at its single earned use (README:129 and the landing How-it-works cell — both currently correct).

=== 5. WHAT'S MISSING ===

- Social proof (logos, stars, testimonials): absent, and CORRECTLY so — padding would violate Honest over sensational and this audience punishes it. The transcript, "374 green," and the named-competitor table are the substitute, and better.
- Risk reversal: missing, and there's a free one in the repo — reproducible proof. "Don't take the transcript's word for it — `bash scripts/demo.sh` runs two agents mentioning each other on your machine." That is the dev-native form of a money-back guarantee (skill: Final CTA + risk reversal). One line near the quickstart or footer CTA.
- Objection handling on the landing: the real gap (P0-3, P1-10). Priority order for this reader: (1) who can inject turns into my agent, (2) where do my transcript summaries go / whose LLM tokens, (3) what breaks when the daemon/relay dies (already answered in README FAQ — surface on the landing too).
- License/price signal: the landing never says open source or free; "Get it on GitHub" only implies it. One word in the footer fine-print ("open source" or "MIT-licensed" — whichever is true; verify the actual license first) removes a lurking question.
- Urgency: DO NOT add. Scarcity devices would torch credibility here; it's the one skill conversion element that would backfire with this audience.

=== 6. TOP 5 CHANGES, RANKED ===

1. Handle the injection/trust objection (P0-3). Before: (nothing) → After: "@mention, delivered as a real turn. … Only keys you whitelist can reach your agents." + FAQ "Can anyone message my agents?"
2. Fix the false-ringing footer line (P0-1). Before: "Nothing leaves your keys." → After: "Your keys never leave your disk. Your agents' traffic goes only to relays you choose."
3. Align CLI + Cargo strings (P0-2). Before: "Citizenship for your agents: identity + awareness on the Nostr fabric." → After: "An identity and awareness fabric for the coding agents you already run."
4. Give the README a headline (P1-5). Before: "# tenex-edge" → paragraph → After: transcript, then bold "Stop being the message bus between your own agents.", then the split paragraph (P1-6).
5. HTTPS the clone command (P0-4). Before: git clone git@github.com:pablof7z/tenex-edge.git → After: git clone https://github.com/pablof7z/tenex-edge.git.

Files reviewed: /Users/pablofernandez/src/tenex-edge/README.md · /private/tmp/claude-501/-Users-pablofernandez-src-tenex-edge/75f62bb9-f564-4633-8741-997dfea1d0e7/scratchpad/landing.html · /Users/pablofernandez/src/tenex-edge/src/cli/args.rs:14 · /Users/pablofernandez/src/tenex-edge/Cargo.toml:5
</teammate-message>
