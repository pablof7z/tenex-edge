//! `mosaico debug explain <handle>` — point at an artifact, see what produced it.

use anyhow::Result;
use clap::Args;
use serde_json::Value;

#[derive(Args)]
pub(super) struct ExplainArgs {
    /// The artifact handle: `event:<id>`, `session:<id>[@<ts>]`,
    /// `hook:<id>[@<ts>]`, `txn:<surface>:<id>[@<ts>]`, or `sub:<channel>`.
    pub(super) handle: String,
    /// Emit the raw receipt record as JSON instead of the human view.
    #[arg(long)]
    pub(super) json: bool,
}

pub(super) fn explain(args: ExplainArgs) -> Result<()> {
    crate::explain::parse_handle(&args.handle)?;
    let params = serde_json::json!({ "handle": args.handle });
    let value = crate::daemon::blocking::call("explain", params)?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&value)?);
    } else {
        print!("{}", render(&args.handle, &value));
    }
    Ok(())
}

fn render(handle: &str, value: &Value) -> String {
    use std::fmt::Write as _;
    let mut out = String::new();
    let kind = value.get("kind").and_then(Value::as_str).unwrap_or("?");
    let _ = writeln!(out, "explain {handle}  ({kind})\n");

    let receipts = value.get("receipts").and_then(Value::as_array);
    match receipts.filter(|rows| !rows.is_empty()) {
        None => {
            let _ = writeln!(out, "receipts: (none found for this handle)");
        }
        Some(rows) => {
            let _ = writeln!(out, "receipts:");
            for receipt in rows {
                let text = |key| receipt.get(key).and_then(Value::as_str).unwrap_or("");
                let number = |key| receipt.get(key).and_then(Value::as_i64).unwrap_or(0);
                let artifact = receipt
                    .get("artifact_ref")
                    .and_then(Value::as_str)
                    .unwrap_or("(none)");
                let _ = writeln!(
                    out,
                    "  [{}] txn {} rev {} → {}  (at {})",
                    text("surface"),
                    number("transaction_id"),
                    number("revision"),
                    artifact,
                    number("created_at"),
                );
                let _ = writeln!(out, "      changed: {}", text("changed_summary"));
                let _ = writeln!(out, "      commands: {}", text("commands"));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_render_surfaces_receipt_evidence() {
        let value = serde_json::json!({
            "kind": "event",
            "receipts": [{
                "surface": "status", "transaction_id": 5, "revision": 2,
                "changed_summary": "{\"pubkey\":\"pk\"}",
                "commands": "[]", "artifact_ref": "event-id", "created_at": 10,
            }],
        });
        let text = render("event:event-id", &value);
        assert!(text.contains("[status] txn 5 rev 2 → event-id"));
        assert!(text.contains("changed: {\"pubkey\":\"pk\"}"));
    }
}
