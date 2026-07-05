# Trellis Diagnosis Corpus Ground Truth

These replay scripts seed the diagnosis-metric experiment from issue #228.
They are not bug reintroductions; they are known-answer scenarios for comparing
counterfactual tooling and model diagnoses.

## `leaked-close.json`

Ground truth: a shared subscription must close only when the last owning scope
leaves. Reverting the refcount fix from #205 makes the first departing owner
close `sub/h/room` even while `s2` still owns the same channel.

Expected counterfactual signal: removing `subscriptions/session/s1/channels`
from the second snapshot must not close the room while `s2` remains present.
A close after the first owner leaves identifies the root cause as owner/refcount
collapse, not relay behavior.

## `false-republish.json`

Ground truth: status publishes are content/TTL deduped. A tick in the same TTL
bucket with unchanged status content must emit no second status command.

Expected counterfactual signal: changing only unused metadata, such as a distill
window hash, must not change the status artifact. A publish on unchanged content
identifies the root cause as disabled status dedup, not LLM output drift.
