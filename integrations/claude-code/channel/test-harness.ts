#!/usr/bin/env bun
//
// Minimal stdio MCP client harness for manually testing server.ts WITHOUT a real
// Claude Code session. Speaks just enough JSON-RPC to:
//   1. initialize the server
//   2. list tools (assert `reply` is present)
//   3. watch for `notifications/claude/channel` frames (emitted when a mention
//      arrives via the re-arm loop) and print them
//   4. optionally call the `reply` tool: `bun test-harness.ts reply <recipient> <message>`
//
// Usage:
//   bun test-harness.ts                       # boot + list tools + watch for mentions (30s)
//   bun test-harness.ts reply slug@proj "hi"  # boot + call the reply tool, then exit
//
// It spawns server.ts as a subprocess and talks newline-delimited JSON-RPC over
// its stdin/stdout (the framing the MCP stdio transport uses). Server
// diagnostics (stderr) are forwarded to our stderr.
//
const SERVER = `${import.meta.dir}/server.ts`;

const proc = Bun.spawn(["bun", SERVER], {
  stdin: "pipe",
  stdout: "pipe",
  stderr: "inherit", // surface server.ts's log() lines live
  env: { ...process.env },
});

let buf = "";
let nextId = 1;
const pending = new Map<number, (msg: any) => void>();

function send(method: string, params?: any, isNotification = false) {
  const msg: any = { jsonrpc: "2.0", method };
  if (params !== undefined) msg.params = params;
  let id: number | undefined;
  if (!isNotification) {
    id = nextId++;
    msg.id = id;
  }
  const json = JSON.stringify(msg);
  // MCP stdio transport = newline-delimited JSON (one message per line).
  proc.stdin.write(json + "\n");
  proc.stdin.flush();
  return id;
}

function call(method: string, params?: any): Promise<any> {
  const id = send(method, params)!;
  return new Promise((resolve) => pending.set(id, resolve));
}

// Read framed messages off the server's stdout.
(async () => {
  const reader = proc.stdout.getReader();
  const dec = new TextDecoder();
  for (;;) {
    const { value, done } = await reader.read();
    if (done) break;
    buf += dec.decode(value, { stream: true });
    for (;;) {
      const nl = buf.indexOf("\n");
      if (nl === -1) break;
      const line = buf.slice(0, nl).trim();
      buf = buf.slice(nl + 1);
      if (!line) continue;
      let msg: any;
      try {
        msg = JSON.parse(line);
      } catch {
        continue;
      }
      if (msg.id !== undefined && pending.has(msg.id)) {
        pending.get(msg.id)!(msg);
        pending.delete(msg.id);
      } else if (msg.method === "notifications/claude/channel") {
        console.log(
          "\n>>> CHANNEL EVENT <<<\n" + JSON.stringify(msg.params, null, 2) + "\n"
        );
      } else if (msg.method) {
        console.log(`(server notification: ${msg.method})`);
      }
    }
  }
})();

// ── drive the handshake ──
const init = await call("initialize", {
  protocolVersion: "2025-06-18",
  capabilities: {},
  clientInfo: { name: "test-harness", version: "0.0.1" },
});
console.log("initialize ok:", JSON.stringify(init.result?.serverInfo));
console.log(
  "declared capabilities:",
  JSON.stringify(init.result?.capabilities)
);
send("notifications/initialized", {}, true);

const tools = await call("tools/list", {});
console.log("tools:", JSON.stringify(tools.result?.tools?.map((t: any) => t.name)));

const mode = process.argv[2];
if (mode === "reply") {
  const recipient = process.argv[3];
  const message = process.argv[4];
  console.log(`calling reply -> ${recipient}: ${message}`);
  const res = await call("tools/call", {
    name: "reply",
    arguments: { recipient, message },
  });
  console.log("reply result:", JSON.stringify(res.result || res.error));
  proc.kill();
  process.exit(0);
} else {
  console.log(
    "\nWatching for channel events for 30s (trigger one with: " +
      "tenex-edge send-message <this-session-or-slug> \"hi\") ...\n"
  );
  await Bun.sleep(30000);
  proc.kill();
  process.exit(0);
}
