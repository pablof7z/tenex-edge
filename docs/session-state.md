# Session State

Every user-facing session surface uses one normalized state. Runtime facts stay
below this boundary and must not appear in agent-facing output.

| State | Contract |
|---|---|
| `working` | The session is online and mid-turn. |
| `idle` | The session is online, between turns, and Tenex can automatically drive its attention. |
| `suspended` | The session is online and between turns, but has no live automatic-delivery path. Mentions remain queued until manual resume. |
| `offline` | The session is not live. |

The owning host is authoritative because it alone can classify liveness,
mid-turn activity, and automatic delivery together. It publishes the normalized
value in kind:30315's `state` tag; listings and injected presence render that
value rather than reconstructing state from lower-level details.

Fresh peer status is authoritative. A clean session end publishes `offline`
immediately, while NIP-40 expiration makes an unrefreshed state `offline` for
every viewer. Heartbeats refresh liveness without advancing the semantic
`updated_at` clock; only a title, activity, slug, or normalized-state change
produces a presence delta.

The unavoidable distributed limit is abrupt host or network loss. A remote
viewer retains the last published state until expiration because, without a new
trusted signal, it cannot distinguish a partition from a silent host sooner.
