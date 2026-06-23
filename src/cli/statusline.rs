//! `tenex-edge statusline` — the fabric, one line at a time.
//!
//! Renders the awareness floor for a host status bar:
//!   claude@kubrick tenex-edge #session-a1b2 [Refactoring inbox] [writing tests]
//!   └ identity ┘  └ project┘  └ channel ┘ └ channel title ┘ └ 30315 status ┘
//!
//! `agentName` is exactly what the session published in its kind:0 profile
//! (`name`). `project-name` is the work-root project. `#session-123` is the
//! channel the session is currently on, so it changes when `channels switch`
//! changes the session's route. `[title]` is that channel's title. `[status]`
//! is the kind:30315 live activity line when busy, or `idle` when not.

use super::*;

const TITLE_MAX_CHARS: usize = 48;
const ACTIVITY_MAX_CHARS: usize = 48;

pub(super) fn statusline(
    session: Option<String>,
    agent_arg: Option<String>,
    cwd_arg: Option<String>,
    pane_arg: Option<String>,
    tmux_fmt: bool,
) -> Result<()> {
    let raw: serde_json::Value = if io::stdin().is_terminal() {
        serde_json::Value::Null
    } else {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf).ok();
        serde_json::from_str(&buf).unwrap_or(serde_json::Value::Null)
    };
    let env_session = raw
        .get("session_id")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let cwd = cwd_arg
        .or_else(|| {
            raw.pointer("/workspace/current_dir")
                .or_else(|| raw.get("cwd"))
                .and_then(|v| v.as_str())
                .map(str::to_string)
        })
        .or_else(|| {
            std::env::current_dir()
                .ok()
                .map(|p| p.to_string_lossy().to_string())
        });
    let agent = agent_arg.or_else(agent_env_slug);
    let pane = pane_arg.filter(|s| !s.is_empty());

    let params = serde_json::json!({
        "session": session,
        "env_session": env_session,
        "cwd": cwd,
        "agent": agent,
        "pane": pane,
    });
    let Ok(v) = crate::daemon::blocking::call_no_spawn("statusline", params) else {
        return Ok(());
    };
    let Ok(view) = serde_json::from_value::<StatuslineView>(v) else {
        return Ok(());
    };
    let line = if tmux_fmt {
        render_statusline_tmux(&view)
    } else {
        render_statusline(&view, true)
    };
    println!("{line}");
    Ok(())
}

#[derive(Debug, Default, serde::Deserialize)]
pub struct StatuslineView {
    #[serde(default)]
    agent: String,
    #[serde(default)]
    host: String,
    #[serde(default)]
    work_root: String,
    #[serde(default)]
    channel: String,
    #[serde(default)]
    channel_title: String,
    #[serde(default)]
    member_count: u64,
    #[serde(default = "default_true")]
    is_member: bool,
    #[serde(default)]
    working: bool,
    #[serde(default)]
    activity: String,
    #[serde(default)]
    distill_error: Option<String>,
}

fn default_true() -> bool {
    true
}

fn ansi_to_tmux_style(code: &str) -> &'static str {
    match code {
        "36" => "fg=colour6",
        "2" => "dim",
        "32" => "fg=colour2",
        "1;31" => "fg=colour1,bold",
        _ => "default",
    }
}

pub fn render_statusline(v: &StatuslineView, color: bool) -> String {
    render_statusline_inner(v, color, false)
}

pub fn render_statusline_tmux(v: &StatuslineView) -> String {
    render_statusline_inner(v, true, true)
}

fn render_statusline_inner(v: &StatuslineView, color: bool, tmux_fmt: bool) -> String {
    let paint = |s: String, code: &str| -> String {
        if tmux_fmt {
            format!("#[{}]{}#[default]", ansi_to_tmux_style(code), s)
        } else if color {
            format!("\x1b[{code}m{s}\x1b[0m")
        } else {
            s
        }
    };
    let mut segs = Vec::new();

    segs.push(paint(
        format!("{}@{}", v.agent, slugify_host(&v.host)),
        "36",
    ));
    segs.push(paint(v.work_root.clone(), "2"));
    segs.push(paint(format!("#{}", v.channel), "2"));

    if !v.channel_title.is_empty() {
        segs.push(format!(
            "[{}]",
            truncate_chars(&v.channel_title, TITLE_MAX_CHARS)
        ));
    }

    let status = if v.working {
        if v.activity.is_empty() {
            "working".to_string()
        } else {
            truncate_chars(&v.activity, ACTIVITY_MAX_CHARS)
        }
    } else {
        "idle".to_string()
    };
    segs.push(paint(format!("[{status}]"), "2"));

    if !v.is_member && v.member_count > 0 {
        segs.push(paint(
            format!("⚠ not in channel {}", v.channel),
            "1;31",
        ));
    }

    if let Some(ref err) = v.distill_error {
        segs.push(paint(
            format!("⚠ distill: {}", truncate_chars(err, 40)),
            "1;31",
        ));
    }

    segs.join(" ")
}

fn truncate_chars(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let cut: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{}…", cut.trim_end())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view() -> StatuslineView {
        StatuslineView {
            agent: "claude".into(),
            host: "Kubrick's Mac".into(),
            work_root: "tenex-edge".into(),
            channel: "session-a1b2c3d4e5f60718".into(),
            channel_title: "Refactoring inbox".into(),
            member_count: 4,
            is_member: true,
            working: true,
            activity: "writing tests".into(),
            distill_error: None,
        }
    }

    #[test]
    fn renders_identity_project_channel_title_and_status() {
        let s = render_statusline(&view(), false);
        assert_eq!(
            s,
            "claude@kubrick-s-mac tenex-edge #session-a1b2c3d4e5f60718 \
             [Refactoring inbox] [writing tests]"
        );
    }

    #[test]
    fn busy_without_activity_says_working() {
        let mut v = view();
        v.activity = String::new();
        let s = render_statusline(&v, false);
        assert!(s.ends_with("[working]"), "got: {s}");
    }

    #[test]
    fn idle_says_idle() {
        let mut v = view();
        v.working = false;
        let s = render_statusline(&v, false);
        assert!(s.ends_with("[idle]"), "got: {s}");
    }

    #[test]
    fn empty_title_is_omitted() {
        let mut v = view();
        v.channel_title = String::new();
        let s = render_statusline(&v, false);
        assert!(!s.contains("[]"), "got: {s}");
        assert!(s.contains("[writing tests]"), "got: {s}");
    }

    #[test]
    fn membership_warning_is_kept() {
        let mut v = view();
        v.is_member = false;
        let s = render_statusline(&v, false);
        assert!(
            s.contains("⚠ not in channel session-a1b2c3d4e5f60718"),
            "got: {s}"
        );

        v.member_count = 0;
        let s = render_statusline(&v, false);
        assert!(!s.contains("not in channel"), "got: {s}");
    }

    #[test]
    fn distill_warning_is_kept() {
        let mut v = view();
        v.distill_error = Some("LLM rate-limited".into());
        let s = render_statusline(&v, false);
        assert!(s.contains("⚠ distill: LLM rate-limited"), "got: {s}");
    }

    #[test]
    fn truncates_long_title_and_activity() {
        let mut v = view();
        v.channel_title = "x".repeat(100);
        v.activity = "y".repeat(100);
        let s = render_statusline(&v, false);
        assert!(s.matches('…').count() >= 2, "got: {s}");
    }
}
