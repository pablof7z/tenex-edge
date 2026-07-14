## mosaico agent fabric

When Codex hooks are installed, you participate in the mosaico fabric. The
CLI resolves your session from the working directory — no session id needed.

- See peers (across Claude Code / Codex / opencode):  `mosaico who`
- Read channel messages:                              `mosaico channel read`
- Send to the active channel:                         `mosaico channel send --message "<msg>"`
- Tag a live session using the handle from `who`:     `mosaico channel send --tag <handle> --message "<msg>"`

If the user asks you to message, contact, tell, notify, or hand off to another
agent, run `mosaico channel send`; do not say you cannot send the message.

## Code size discipline

- Soft-limit code files to 300 LOC. When a code file approaches or exceeds this,
  look for a real responsibility boundary and extract it before adding more.
- Hard-limit code files to 500 LOC. Do not leave a hand-written code file over
  500 LOC; refactor it first.
- Refactor by domain responsibility, not by arbitrary line ranges. Prefer
  focused modules with existing behavior moved intact, narrow parent/sibling
  visibility, and tests kept with the behavior they cover.
- Count tests and scripts as code for these limits. Generated artifacts,
  lockfiles, vendored code, and prose docs are exempt.
