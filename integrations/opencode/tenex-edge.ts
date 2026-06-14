import type { Plugin } from "@opencode-ai/plugin"
import { execFile } from "node:child_process"
import { writeFile } from "node:fs/promises"
import { existsSync } from "node:fs"
import { tmpdir, homedir } from "node:os"
import { join } from "node:path"

// ── tenex-edge ⇄ opencode bridge ─────────────────────────────────────────────
//
// Makes an opencode session a citizen on the tenex-edge fabric. Lifecycle steps
// all go through the single `hook` entry point (same as Claude Code / Codex) —
// this plugin pipes a JSON payload on stdin and reads stdout back:
//   session-start (hook)        → on first message of a session (spawns the
//                   presence engine, watching opencode's PID so it reaps on exit;
//                   the daemon generates the session id and returns it on stdout)
//   user-prompt-submit (hook)   → experimental.chat.messages.transform, gated to
//                   once per user message (marks the session "working" so the
//                   engine starts its distillation timer)
//   stop (hook)                 → event → session.idle (marks the session idle)
//   inject        → experimental.chat.messages.transform (prepends peer mentions
//                   addressed to this session, so the agent SEES them and acts)
//
// Distillation is turn-bracketed, not tool-driven: the engine reads the
// transcript ~30s after turn-start (then every 5 min until turn-end). opencode
// has no transcript file, so we keep a temp JSONL snapshot fresh — rewriting it
// at turn-start and again on every tool.execute.after — so the path handed to
// the engine always reflects the recent conversation.
//
// tenex-edge knows nothing about opencode; this plugin is the straw.
// Env: TENEX_EDGE_BIN (path), TENEX_EDGE_AGENT (slug, default "opencode").

function stripAnsi(s: string): string {
  return s.replace(/\x1b\[[0-9;]*m/g, "")
}

function resolveBin(): string {
  if (process.env.TENEX_EDGE_BIN) return process.env.TENEX_EDGE_BIN
  for (const c of [join(homedir(), ".local", "bin", "tenex-edge"), "/usr/local/bin/tenex-edge"]) {
    if (existsSync(c)) return c
  }
  return "tenex-edge"
}

export const TenexEdge: Plugin = async ({ client, directory }) => {
  const BIN = resolveBin()
  let hinted = false

  function run(args: string[]): Promise<string> {
    return new Promise((resolve) => {
      execFile(BIN, args, { timeout: 60_000, maxBuffer: 8 * 1024 * 1024 }, (_e, out) =>
        resolve(out ?? ""),
      )
    })
  }

  // Session/turn lifecycle goes through the single `hook` entry point — the same
  // door Claude Code and Codex use — by piping a small JSON payload on stdin and
  // reading stdout back. tenex-edge has no per-step subcommands anymore; `hook
  // --host opencode --type <t>` parses this payload and drives the lifecycle.
  // (Peer queries — `who`, `inbox`, `send-message` — stay plain subcommands.)
  function runHook(type: string, payload: Record<string, unknown>): Promise<string> {
    return new Promise((resolve) => {
      const child = execFile(
        BIN,
        ["hook", "--host", "opencode", "--type", type],
        { timeout: 60_000, maxBuffer: 8 * 1024 * 1024 },
        (_e, out) => resolve(out ?? ""),
      )
      child.stdin?.end(JSON.stringify(payload))
    })
  }

  // ── transcript extraction (opencode-specific; the binary just reads a path) ──
  // opencode has no transcript file: the conversation lives in the SDK message
  // store. So — exactly like pc — fetch recent messages, flatten the text parts
  // to a flat {role,content} JSONL temp file, and hand that path to the engine,
  // which distills the agent's *intent* from the real conversation. The temp
  // path is deterministic per opencode session (tenex-oc-<ocSID>.jsonl), so we
  // rewrite it in place to keep it fresh as the turn progresses (at turn-start
  // and on each tool.execute.after) — the path the engine holds stays valid.
  // `_tenexInjected` parts (our own peer briefings) are filtered out so they
  // don't pollute the distiller context.
  function partsToText(parts: any[]): string {
    return (parts ?? [])
      .filter((p) => p?.type === "text" && !p?._tenexInjected && typeof p?.text === "string")
      .map((p) => p.text)
      .join("\n")
      .trim()
  }

  async function writeTranscript(
    msgs: Array<{ info: { role: string }; parts: any[] }>,
    ocSessionID: string,
  ): Promise<string | undefined> {
    const lines: string[] = []
    for (const m of msgs.slice(-20)) {
      const role = m.info?.role
      if (role !== "user" && role !== "assistant") continue
      const text = partsToText(m.parts ?? [])
      if (text) lines.push(JSON.stringify({ role, content: text }))
    }
    if (!lines.length) return undefined
    const path = join(tmpdir(), `tenex-oc-${ocSessionID}.jsonl`)
    await writeFile(path, lines.join("\n"))
    return path
  }

  // Fetch via the opencode client using the *opencode* session id (NOT the
  // tenex-edge SID). Returns a temp transcript path, or undefined on any failure
  // so the caller proceeds without --transcript.
  async function fetchTranscript(ocSessionID: string): Promise<string | undefined> {
    try {
      const res: any = await client.session.messages({ path: { id: ocSessionID } })
      const data = res?.data ?? res
      if (!Array.isArray(data)) return undefined
      return await writeTranscript(data, ocSessionID)
    } catch {
      return undefined
    }
  }

  // Start the session in the background (fire-and-forget) so plugin load NEVER
  // blocks opencode startup. When it completes we record the session id + export
  // it so the agent's own `tenex-edge` shell commands resolve to THIS session.
  // Watch opencode's PID so the engine reaps + goes idle when opencode exits.
  // No session id of our own — session-start generates one and returns it on
  // stdout. We pass our PID so the engine reaps when opencode exits. The agent
  // slug comes from the inherited TENEX_EDGE_AGENT env (default "opencode").
  let SID = ""
  runHook("session-start", { cwd: directory, pid: process.pid })
    .then((out) => {
      SID = out.trim()
      if (SID) process.env.TENEX_EDGE_SESSION = SID
    })
    .catch(() => {})

  // Turn bracketing. The transform handler fires once per *model invocation*
  // (i.e. many times per user turn in an agentic loop), so turn-start is gated
  // to fire only when the latest user message id changes — otherwise we'd reset
  // the engine's 30s distillation timer on every tool round-trip and never
  // distill. We also remember the opencode session id so we can keep the
  // transcript snapshot fresh on tool.execute.after (which has no message id).
  let lastTurnMsgID = ""
  let ocSessionForTurn = ""

  return {
    // Inject peer mentions before the model sees the turn, and (once per user
    // message) mark the turn "working" with a fresh transcript snapshot.
    "experimental.chat.messages.transform": async (_input, output) => {
      const msgs = output.messages as Array<{ info: any; parts: any[] }>
      let lastUser: { info: any; parts: any[] } | undefined
      for (let i = msgs.length - 1; i >= 0; i--) {
        if (msgs[i].info?.role === "user") { lastUser = msgs[i]; break }
      }
      if (!lastUser) return
      const msgID: string = lastUser.info.id
      const ocSessionID: string = lastUser.info.sessionID
      ocSessionForTurn = ocSessionID

      // New user message → start of a new turn. Snapshot the conversation and
      // hand the (deterministic) path to the engine; tool.execute.after keeps
      // that same path fresh as the turn progresses.
      if (SID && msgID && msgID !== lastTurnMsgID) {
        lastTurnMsgID = msgID
        const transcriptPath = await fetchTranscript(ocSessionID)
        // Turn start maps to the user-prompt-submit hook. We deliberately omit
        // `prompt`, so (unlike Claude Code / Codex) the prompt is NOT published
        // as a kind:1 OP — preserving this plugin's existing behavior.
        // Forward opencode's native session id as the resume token: it is what
        // `opencode --session <id>` wants, and (unlike claude/codex) it differs
        // from our synthetic `te-*` SID, so the daemon can't derive it itself.
        await runHook("user-prompt-submit", {
          session_id: SID,
          resume_id: ocSessionID,
          ...(transcriptPath ? { transcript_path: transcriptPath } : {}),
        })
      }

      let blocks: string[] = []
      // One-time hint so the agent knows how to message peers from the shell.
      if (!hinted) {
        hinted = true
        blocks.push(
          `You are a citizen on the tenex-edge fabric. Peers (across Claude Code, ` +
          `Codex, opencode) are reachable by name. See them: \`${BIN} who\`. ` +
          `Check messages others sent you: \`${BIN} inbox\`. ` +
          `Message one: \`${BIN} send-message --recipient <agent@project|session-id> --message "..."\`. ` +
          `(No session id needed — it's resolved from the working directory.) ` +
          `Run \`${BIN} wait-for-mention\` with run_in_background=true now so you are woken when a mention arrives. Re-run it each time one is received.`,
        )
      }
      // Use SID when known, else let the CLI resolve via the env we exported.
      const inboxArgs = SID ? ["inbox", "--session", SID] : ["inbox"]
      const inbox = (await run(inboxArgs)).trim()
      if (inbox) blocks.push("Messages from other agents (via tenex-edge):\n" + inbox)

      // Who's around and what they're doing.
      const who = stripAnsi((await run(["who"])).trim())
      if (who && !who.includes("no live agents")) {
        blocks.push("tenex-edge fabric — agents you can message (and what they're doing):\n" + who)
      }

      if (blocks.length) {
        lastUser.parts.unshift({
          id: `tenex-edge-${msgID}`,
          sessionID: ocSessionID,
          messageID: msgID,
          type: "text",
          text: blocks.join("\n\n"),
          _tenexInjected: true,
        } as any)
      }
    },

    // Keep the transcript snapshot fresh DURING the turn. We don't call the CLI
    // here — distillation is turn-bracketed, not tool-driven. We just rewrite
    // the same deterministic temp JSONL path (per opencode session) that we
    // handed the engine at turn-start, so when the engine re-reads it ~30s in
    // (and every 5 min) it reflects the work done so far this turn.
    "tool.execute.after": async (input: any, _output: any) => {
      if (!SID) return
      const ocSessionID = String(input?.sessionID ?? ocSessionForTurn ?? "")
      if (!ocSessionID) return
      await fetchTranscript(ocSessionID)
    },

    // Turn finished: opencode emits session.idle when the assistant is done
    // responding. Mark the session idle. session.idle carries the *opencode*
    // session id, but the stop hook wants the tenex-edge SID (1:1 in this plugin).
    event: async ({ event }: { event: any }) => {
      if (!SID) return
      if (event?.type === "session.idle") {
        await runHook("stop", { session_id: SID })
      }
    },
  }
}
