import type { Plugin } from "@opencode-ai/plugin"
import { execFile } from "node:child_process"
import { writeFile } from "node:fs/promises"
import { existsSync } from "node:fs"
import { tmpdir, homedir } from "node:os"
import { join } from "node:path"

// ── mosaico ⇄ opencode bridge ─────────────────────────────────────────────
//
// Makes an opencode session a citizen on the mosaico fabric. Lifecycle steps
// all go through the single `hook` entry point (same as Claude Code / Codex) —
// this plugin pipes a JSON payload on stdin and reads stdout back:
//   session-start (hook)        → on first message of a session (spawns the
//                   presence engine, watching opencode's PID so it reaps on exit)
//   user-prompt-submit (hook)   → experimental.chat.messages.transform, on the
//                   first model invocation of a user message (marks "working",
//                   captures the transcript path). Its stdout — self-identity,
//                   workspace chat, peer roster, all assembled by
//                   the shared Rust hook — is injected verbatim into the turn.
//   post-tool-use (hook)        → experimental.chat.messages.transform, on later
//                   model invocations of the same message (mid-turn checkpoint:
//                   peeks new messages + sibling deltas; same stdout-as-context).
//   stop (hook)                 → event → session.idle (marks the session idle)
//
// The plugin never builds context strings itself: the hook is the single source
// of truth, identical to Claude Code / Codex. We pipe its stdout into the turn.
//
// opencode has no transcript file, so we keep a temp JSONL snapshot fresh for
// the daemon's transcript-backed auto-reply path.
//
// mosaico knows nothing about opencode; this plugin is the straw.
// Env: MOSAICO_BIN (path), MOSAICO_AGENT (slug, default "opencode").

function resolveBin(): string {
  if (process.env.MOSAICO_BIN) return process.env.MOSAICO_BIN
  for (const c of [join(homedir(), ".local", "bin", "mosaico"), "/usr/local/bin/mosaico"]) {
    if (existsSync(c)) return c
  }
  return "mosaico"
}

export const Mosaico: Plugin = async ({ client, directory }) => {
  const BIN = resolveBin()

  // Session/turn lifecycle goes through the single `hook` entry point — the same
  // door Claude Code and Codex use — by piping a small JSON payload on stdin and
  // reading stdout back. mosaico has no per-step subcommands anymore; `harness
  // hook opencode --type <t>` parses this payload and drives the lifecycle.
  // (Peer queries — `who`, `chat` — stay plain subcommands.)
  function runHook(type: string, payload: Record<string, unknown>): Promise<string> {
    return new Promise((resolve) => {
      const child = execFile(
        BIN,
        ["harness", "hook", "opencode", "--type", type],
        { timeout: 60_000, maxBuffer: 8 * 1024 * 1024 },
        (_e, out) => resolve(out ?? ""),
      )
      child.stdin?.end(JSON.stringify(payload))
    })
  }

  // ── transcript extraction (opencode-specific; the daemon reads a path) ──────
  // opencode has no transcript file: the conversation lives in the SDK message
  // store. So — exactly like pc — fetch recent messages, flatten the text parts
  // to a flat {role,content} JSONL temp file, and hand that path to the engine,
  // which can recover the latest assistant response for auto-replies. The temp
  // path is deterministic per opencode session (mosaico-oc-<ocSID>.jsonl), so we
  // rewrite it in place to keep it fresh as the turn progresses (at turn-start
  // and on each tool.execute.after) — the path the engine holds stays valid.
  // `_mosaicoInjected` parts (our own peer briefings) are filtered out.
  function partsToText(parts: any[]): string {
    return (parts ?? [])
      .filter((p) => p?.type === "text" && !p?._mosaicoInjected && typeof p?.text === "string")
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
    const path = join(tmpdir(), `mosaico-oc-${ocSessionID}.jsonl`)
    await writeFile(path, lines.join("\n"))
    return path
  }

  // Fetch via the opencode client using the opencode-native session locator.
  // Returns a temp transcript path, or undefined on any failure
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
  // blocks opencode startup.
  // Watch opencode's PID so the engine reaps + goes idle when opencode exits.
  // No native session locator exists at plugin startup. We pass our PID so the
  // daemon has a typed lifecycle locator until the first turn. The agent
  // slug comes from the inherited MOSAICO_AGENT env (default "opencode").
  runHook("session-start", { cwd: directory, pid: process.pid }).catch(() => {})

  // Turn bracketing. The transform handler fires once per *model invocation*
  // (i.e. many times per user turn in an agentic loop), so turn-start is gated
  // to fire only when the latest user message id changes. We also remember the
  // opencode session id so we can keep the
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

      // The hook is the single source of truth for injected context: the same
      // Rust path that serves Claude Code and Codex assembles the self-identity
      // line, project chat, and peer roster, and prints it on stdout. We don't
      // rebuild any of that here — we just pipe stdout into the turn. Two hook
      // types map to opencode's two moments:
      //   • new user message → user-prompt-submit (turn start: drains chat,
      //     full roster on the first turn, deltas after).
      //   • same user message, later model invocation → post-tool-use (mid-turn
      //     checkpoint: non-destructive peek of new chat + sibling deltas,
      //     rate-limited by the daemon — exactly Claude Code's PostToolUse).
      let context = ""
      if (ocSessionID && msgID && msgID !== lastTurnMsgID) {
        lastTurnMsgID = msgID
        const transcriptPath = await fetchTranscript(ocSessionID)
        // We deliberately omit `prompt`, so (unlike Claude Code / Codex) the
        // prompt is NOT published as a kind:1 OP — preserving prior behavior.
        // The opencode id is a harness locator and resume token, never identity.
        context = (
          await runHook("user-prompt-submit", {
            session_id: ocSessionID,
            resume_id: ocSessionID,
            ...(transcriptPath ? { transcript_path: transcriptPath } : {}),
          })
        ).trim()
      } else if (ocSessionID) {
        context = (await runHook("post-tool-use", { session_id: ocSessionID })).trim()
      }

      if (context) {
        lastUser.parts.unshift({
          id: `mosaico-${msgID}`,
          sessionID: ocSessionID,
          messageID: msgID,
          type: "text",
          text: context,
          _mosaicoInjected: true,
        } as any)
      }
    },

    // Keep the transcript snapshot fresh during the turn for auto-replies.
    "tool.execute.after": async (input: any, _output: any) => {
      const ocSessionID = String(input?.sessionID ?? ocSessionForTurn ?? "")
      if (!ocSessionID) return
      await fetchTranscript(ocSessionID)
    },

    // Turn finished: opencode emits session.idle when the assistant is done
    // responding. Mark the session idle using opencode's native locator.
    event: async ({ event }: { event: any }) => {
      if (event?.type === "session.idle") {
        const ocSessionID = String(event?.properties?.sessionID ?? ocSessionForTurn ?? "")
        if (ocSessionID) await runHook("stop", { session_id: ocSessionID })
      }
    },
  }
}
