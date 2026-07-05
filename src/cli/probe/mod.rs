//! `tenex-edge probe` — hidden diagnostic surface over the reconciler frontier
//! (frontier design §4). A thin client: it forwards a verb + params to the
//! daemon's `probe` RPC and renders the JSON it returns. `--json` (global) emits
//! the raw daemon JSON instead of the human view.
//!
//! Verbs: `stats` (aggregate value evidence, §4.1), `oracle` (live
//! incremental-equals-full correctness, §4.6), `seams` (frontier modes, §4.5),
//! `simulate` (dry-run a fact via `tx.preview()`, the keystone, §3), `why` (live
//! causality for a handle, §4.3), and `state` (live per-surface values, §4.3).

mod render;
mod state_render;
mod stats_render;

use anyhow::Result;
use clap::{Args, Subcommand};
use serde_json::{json, Value};
use std::path::PathBuf;

#[derive(Args)]
pub(in crate::cli) struct ProbeArgs {
    #[command(subcommand)]
    action: ProbeAction,
    /// Emit the raw JSON the daemon returned instead of the human view.
    #[arg(long, global = true)]
    json: bool,
}

#[derive(Subcommand)]
enum ProbeAction {
    /// Aggregate value evidence per surface over the all-commit ledger:
    /// commits, effectful vs suppressed no-ops, command/output totals, latency.
    Stats {
        /// One surface (`status` | `subscriptions` | `hook_context`); omit for all.
        #[arg(long)]
        surface: Option<String>,
        /// Only count commits with `created_at` at/after this unix-millis stamp.
        #[arg(long, default_value = "0")]
        since: i64,
    },
    /// Run the incremental-equals-full oracle on each live reconciler graph.
    Oracle {
        /// Advisory/no-op: accepted for symmetry but IGNORED — the oracle always
        /// runs live over the daemon-held graphs regardless of this flag.
        #[arg(long)]
        now: bool,
        /// Restrict reporting to one surface (advisory; all are checked).
        #[arg(long)]
        surface: Option<String>,
    },
    /// Show the code-owned authority-frontier registrations and bypass risks.
    Seams,
    /// Replay a stored input capsule by id; `--assert` checks deterministic replay.
    Replay {
        /// Capsule id from the replay-capsule store.
        capsule: String,
        /// Assert two independent replays match, including Trellis ledgers.
        #[arg(long = "assert")]
        assert_replay: bool,
        /// Write a Flight Recorder SerializedScenario trace JSON to this path.
        #[arg(long, value_name = "PATH")]
        export_trace: Option<PathBuf>,
    },
    /// Dry-run a fact against a surface via `tx.preview()` — nothing is applied.
    Simulate {
        /// The surface to simulate (`status` | `subscriptions`).
        #[arg(default_value = "status")]
        surface: String,
        /// Exact serde JSON for `InputFact`; preferred over the status shorthand.
        #[arg(long, value_name = "JSON")]
        fact: Option<String>,
        /// Session whose status the legacy shorthand applies to.
        #[arg(long)]
        session: Option<String>,
        /// Legacy status shorthand: distilled live-activity line to set.
        #[arg(long)]
        activity: Option<String>,
        /// Legacy status shorthand: distilled title to set.
        #[arg(long)]
        title: Option<String>,
        /// Unix **seconds** (NOT millis) — the reconciler's clock unit. When set,
        /// re-arms the TTL bucket (`now / refresh_secs`) exactly as a real distill
        /// would; passing millis lands in the wrong bucket and fabricates a
        /// spurious TTL refresh in the previewed plan. Omit to preview only the
        /// content change.
        #[arg(long)]
        now: Option<u64>,
    },
    /// Explain the latest change to a handle (`sub:<channel>` | `status:<session>` | `hook:<session>`).
    Why { handle: String },
    /// Live values for a surface (`status` | `subscriptions` | `hook_context`).
    State {
        surface: String,
        /// Surface-specific handle; for `hook_context`, this is the session id.
        handle: Option<String>,
        /// Include verbose graph debug output when supported by the surface.
        #[arg(long)]
        dump: bool,
    },
}

impl ProbeAction {
    /// Project the parsed verb into the `{verb, ...}` RPC params the daemon's
    /// `probe` arm dispatches on.
    fn to_rpc(&self) -> Result<(String, Value)> {
        match self {
            ProbeAction::Stats { surface, since } => Ok((
                "stats".into(),
                json!({ "verb": "stats", "surface": surface, "since": since }),
            )),
            ProbeAction::Oracle { surface, .. } => Ok((
                "oracle".into(),
                json!({ "verb": "oracle", "surface": surface }),
            )),
            ProbeAction::Seams => Ok(("seams".into(), json!({ "verb": "seams" }))),
            ProbeAction::Replay {
                capsule,
                assert_replay,
                export_trace,
            } => Ok((
                "replay".into(),
                json!({
                    "verb": "replay",
                    "capsule": capsule,
                    "assert": assert_replay,
                    "export_trace": export_trace.is_some(),
                }),
            )),
            ProbeAction::Simulate {
                surface,
                fact,
                session,
                activity,
                title,
                now,
            } => {
                let fact = fact
                    .as_deref()
                    .map(serde_json::from_str::<Value>)
                    .transpose()?;
                Ok((
                    "simulate".into(),
                    json!({ "verb": "simulate", "surface": surface, "fact": fact,
                            "session": session, "activity": activity, "title": title,
                            "now": now }),
                ))
            }
            ProbeAction::Why { handle } => {
                Ok(("why".into(), json!({ "verb": "why", "handle": handle })))
            }
            ProbeAction::State {
                surface,
                handle,
                dump,
            } => Ok((
                "state".into(),
                json!({ "verb": "state", "surface": surface, "handle": handle, "dump": dump }),
            )),
        }
    }

    fn export_trace_path(&self) -> Option<PathBuf> {
        match self {
            ProbeAction::Replay { export_trace, .. } => export_trace.clone(),
            _ => None,
        }
    }
}

pub(in crate::cli) async fn probe(args: ProbeArgs) -> Result<()> {
    let export_trace_path = args.action.export_trace_path();
    let (verb, params) = args.action.to_rpc()?;
    let mut v = super::daemon_call_async("probe", params).await?;
    if let Some(path) = export_trace_path {
        let trace = v
            .get("trace_json")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("probe replay did not return trace_json"))?;
        std::fs::write(&path, trace)?;
        if let Some(obj) = v.as_object_mut() {
            obj.insert(
                "trace_path".into(),
                Value::String(path.display().to_string()),
            );
        }
    }
    if args.json {
        println!("{}", serde_json::to_string_pretty(&v)?);
    } else {
        print!("{}", render(&verb, &v));
    }
    Ok(())
}

/// Human view. `stats` gets its table here; the other verbs render in `render.rs`.
/// A wired-but-unimplemented shape (e.g. subscriptions simulate v2) prints its
/// marker; an unexpected shape falls back to a raw dump.
fn render(verb: &str, v: &Value) -> String {
    if v.get("implemented").and_then(Value::as_bool) == Some(false) {
        let msg = v
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("not implemented");
        return format!("probe {verb}: {msg}\n");
    }
    match verb {
        "stats" => stats_render::render_stats(v),
        "oracle" => render::render_oracle(v),
        "seams" => render::render_seams(v),
        "replay" => render::render_replay(v),
        "simulate" => render::render_simulate(v),
        "why" => render::render_why(v),
        "state" => state_render::render_state(v),
        _ => format!("{v}\n"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unimplemented_shape_renders_marker() {
        let v = json!({ "verb": "simulate", "implemented": false,
                        "message": "subscriptions simulate is a v2 follow-up" });
        let text = render("simulate", &v);
        assert_eq!(
            text,
            "probe simulate: subscriptions simulate is a v2 follow-up\n"
        );
    }

    #[test]
    fn stats_action_projects_rpc_params() {
        let action = ProbeAction::Stats {
            surface: Some("status".into()),
            since: 42,
        };
        let (verb, params) = action.to_rpc().unwrap();
        assert_eq!(verb, "stats");
        assert_eq!(params["verb"], "stats");
        assert_eq!(params["surface"], "status");
        assert_eq!(params["since"], 42);
    }

    #[test]
    fn seams_action_projects_rpc_params() {
        let (verb, params) = ProbeAction::Seams.to_rpc().unwrap();
        assert_eq!(verb, "seams");
        assert_eq!(params["verb"], "seams");
    }

    #[test]
    fn replay_action_projects_rpc_params() {
        let action = ProbeAction::Replay {
            capsule: "42".into(),
            assert_replay: true,
            export_trace: Some(PathBuf::from("trace.json")),
        };
        let (verb, params) = action.to_rpc().unwrap();
        assert_eq!(verb, "replay");
        assert_eq!(params["capsule"], "42");
        assert_eq!(params["assert"], true);
        assert_eq!(params["export_trace"], true);
    }

    #[test]
    fn simulate_action_projects_rpc_params() {
        let action = ProbeAction::Simulate {
            surface: "status".into(),
            fact: None,
            session: Some("s1".into()),
            activity: Some("reviewing the PR".into()),
            title: None,
            now: None,
        };
        let (verb, params) = action.to_rpc().unwrap();
        assert_eq!(verb, "simulate");
        assert_eq!(params["session"], "s1");
        assert_eq!(params["activity"], "reviewing the PR");
        assert!(params["title"].is_null());
    }

    #[test]
    fn simulate_action_parses_fact_json() {
        let action = ProbeAction::Simulate {
            surface: "subscriptions".into(),
            fact: Some(r#"{"SubscriptionSync":{"snapshot":{"daemon_channels":[],"addressed_pubkeys":[],"archived_channels":[],"sessions":{}},"at":1}}"#.into()),
            session: None,
            activity: None,
            title: None,
            now: None,
        };
        let (_verb, params) = action.to_rpc().unwrap();
        assert!(params["fact"].is_object());
        assert_eq!(params["fact"]["SubscriptionSync"]["at"], 1);
    }
}
