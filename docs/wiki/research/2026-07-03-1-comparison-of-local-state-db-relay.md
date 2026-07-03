---
type: research-record
date: 2026-07-03
session: a685f611-39bd-4a18-a6b7-ea4e38334b82
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/a685f611-39bd-4a18-a6b7-ea4e38334b82.jsonl
source_lines: 758-779
agent_attribution: main
has_preregistered_criteria: true
has_method: true
has_structured_report: true
characterization: "Comparison of local state.db relay_channels table vs relay ground-truth (nak req kind 39000-39003); verdict: 7/8 channels match, tenex-edge channel missing all NIP-29 group-state on relay"
captured_at: 2026-07-03T11:26:15Z
---

Comparison of local state.db relay_channels table vs relay ground-truth (nak req kind 39000-39003); verdict: 7/8 channels match, tenex-edge channel missing all NIP-29 group-state on relay

---

Assistant: Ran the comparison. Local expectation = `relay_channels` table in `state.db`; ground truth = direct `nak req -k 39000..39003 -t d=<channel>` against `wss://nip29.f7z.io` (parameterized-replaceable events, so each query returns 0 or 1 authoritative result).

| channel_h | name (local) | parent (local) | relay 39000–39003 |
|---|---|---|---|
| `trellis` | trellis | — | ✅ all 4 present, matches |
| `nostr-multi-platform` | nostr-multi-platform | — | ✅ all 4 present, matches |
| `40a5ba9d` | x-ray | nostr-multi-platform | ✅ all 4 present, matches |
| `58b2b02b` | x-ray | trellis | ✅ all 4 present, matches |
| `5c379a8c` | trellis-tenex-edge | trellis | ✅ all 4 present, matches |
| `ffffeb50` | wallet-work | nostr-multi-platform | ✅ all 4 present, matches |
| `project` | project | — | ✅ all 4 present, matches |
| **`tenex-edge`** | tenex-edge | — | ❌ **all 4 missing** |

7 of 8 match exactly (name + parent). **`tenex-edge` — the channel this very session runs in — has zero NIP-29 group-state on the relay.** No `39000` (metadata), `39001` (admins), `39002` (members), or `39003` (roles). I checked this isn't an auth-gating artifact: the relay never issues a NIP-42 challenge on connect (`--force-pre-auth` confirms "no challenge, can't AUTH"), and I re-ran the query authenticated as the daemon's own backend key with the same empty result.

It gets stranger: I also queried for the underlying moderation events (`kind 9000/9001/9002/9007/9008`, `#h=tenex-edge`) and got **nothing back** — yet the relay's own live journal (from the earlier investigation) shows it *receiving* exactly these events continuously:
```
→ EVENT ... kind=9000 pub=09417c934409… group="tenex-edge"
→ EVENT ... kind=9002 pub=09417c934409… group="tenex-edge"
→ EVENT ... kind=9007 pub=09417c934409… group="tenex-edge"
```
So the daemon is sending valid admin ops into `tenex-edge`, the relay is receiving them on the wire, but nothing ever materializes as queryable state — no raw event, no derived `39000-39003`. Local `state.db` still has a cached `tenex-edge` row (from `created_at=1783072711`, i.e. an earlier successful sync), so the daemon's cache is stale relative to what the relay can now produce.
