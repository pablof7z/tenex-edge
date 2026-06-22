//! Detect local agent harnesses and wire tenex-edge hooks into each.
//! Mirrors the `pc install` surface: --all, --harness, --dry-run, --status,
//! and --uninstall.

use anyhow::{bail, Context, Result};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event as TermEvent, KeyCode, KeyModifiers},
    execute,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use owo_colors::OwoColorize;
use std::io::{self, IsTerminal as _, Write as _};
use std::path::{Path, PathBuf};

pub(super) struct InstallOpts {
    pub all: bool,
    pub harness: Option<String>,
    pub dry_run: bool,
    pub status: bool,
    pub uninstall: bool,
}

// Embedded opencode plugin: the same file the source tree ships at
// integrations/opencode/tenex-edge.ts.
const OPENCODE_PLUGIN_TS: &str = include_str!("../../integrations/opencode/tenex-edge.ts");
const CODEX_ROOT_HOOK_EVENTS: &[&str] =
    &["SessionStart", "UserPromptSubmit", "PostToolUse", "Stop"];

#[derive(Debug)]
struct Harness {
    id: &'static str,
    display: &'static str,
    config_path: PathBuf,
    detected: bool,
}

struct HarnessChoice {
    index: usize,
    selected: bool,
}

fn harnesses() -> Vec<Harness> {
    let home = home_dir();
    vec![
        Harness {
            id: "claude-code",
            display: "Claude Code",
            config_path: home.join(".claude/settings.json"),
            detected: home.join(".claude").exists() || bin_on_path("claude"),
        },
        Harness {
            id: "codex",
            display: "Codex",
            config_path: home.join(".codex/hooks.json"),
            detected: home.join(".codex").exists() || bin_on_path("codex"),
        },
        Harness {
            id: "opencode",
            display: "opencode",
            config_path: home.join(".config/opencode/plugin/tenex-edge.ts"),
            detected: home.join(".config/opencode").exists() || bin_on_path("opencode"),
        },
    ]
}

fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn bin_on_path(bin: &str) -> bool {
    let Ok(path) = std::env::var("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| dir.join(bin).is_file())
}

/// The hook signature we dedupe by: `tenex-edge hook --host <host> --type <type>`.
fn sig(host: &str, ty: &str) -> String {
    format!("tenex-edge hook --host {host} --type {ty}")
}

fn claude_hook_entries() -> Vec<(&'static str, serde_json::Value)> {
    let mk = |ty: &str, timeout: u64| {
        serde_json::json!({
            "hooks": [{
                "type": "command",
                "command": sig("claude-code", ty),
                "timeout": timeout,
            }]
        })
    };
    vec![
        ("SessionStart", mk("session-start", 10)),
        ("SessionEnd", mk("session-end", 30)),
        ("UserPromptSubmit", mk("user-prompt-submit", 30)),
        ("PostToolUse", mk("post-tool-use", 10)),
        ("Stop", mk("stop", 10)),
    ]
}

fn codex_hook_entries() -> Vec<(&'static str, serde_json::Value)> {
    let mk = |ty: &str, timeout: u64, matcher: Option<&str>| {
        let mut entry = serde_json::json!({
            "hooks": [{
                "type": "command",
                "command": sig("codex", ty),
                "timeout": timeout,
            }]
        });
        if let Some(m) = matcher {
            entry["matcher"] = serde_json::Value::String(m.into());
        }
        entry
    };
    vec![
        (
            "SessionStart",
            mk("session-start", 30, Some("startup|resume")),
        ),
        ("UserPromptSubmit", mk("user-prompt-submit", 30, None)),
        ("PostToolUse", mk("post-tool-use", 10, None)),
        ("Stop", mk("stop", 30, None)),
    ]
}

fn hook_entries(h: &Harness) -> Vec<(&'static str, serde_json::Value)> {
    match h.id {
        "claude-code" => claude_hook_entries(),
        "codex" => codex_hook_entries(),
        _ => Vec::new(),
    }
}

fn host_for_harness(h: &Harness) -> &'static str {
    match h.id {
        "claude-code" => "claude-code",
        "codex" => "codex",
        _ => h.id,
    }
}

/// Does a hook group contain a tenex-edge command for `host`?
fn group_is_ours(group: &serde_json::Value, host: &str) -> bool {
    let needle = format!("tenex-edge hook --host {host} --type ");
    group
        .get("hooks")
        .and_then(|h| h.as_array())
        .map(|hooks| {
            hooks.iter().any(|h| {
                h.get("command")
                    .and_then(|c| c.as_str())
                    .is_some_and(|c| c.contains(&needle))
            })
        })
        .unwrap_or(false)
}

fn ensure_object(v: &mut serde_json::Value) {
    if !v.is_object() {
        *v = serde_json::json!({});
    }
}

fn ensure_hooks_object(
    root: &mut serde_json::Value,
) -> &mut serde_json::Map<String, serde_json::Value> {
    ensure_object(root);
    let root_obj = root.as_object_mut().expect("root forced to object");
    let hooks = root_obj
        .entry("hooks")
        .or_insert_with(|| serde_json::json!({}));
    if !hooks.is_object() {
        *hooks = serde_json::json!({});
    }
    hooks.as_object_mut().expect("hooks forced to object")
}

/// Codex used both root event keys and nested `hooks` JSON during the transition
/// away from TOML. Keep user hooks by moving root event arrays under `hooks`.
fn migrate_codex_root_events(root: &mut serde_json::Value) {
    ensure_object(root);
    let Some(root_obj) = root.as_object_mut() else {
        return;
    };
    let mut moved = Vec::new();
    for event in CODEX_ROOT_HOOK_EVENTS {
        if let Some(value) = root_obj.remove(*event) {
            moved.push(((*event).to_string(), value));
        }
    }
    if moved.is_empty() {
        return;
    }

    let hooks = ensure_hooks_object(root);
    for (event, incoming) in moved {
        match (hooks.get_mut(&event), incoming) {
            (Some(serde_json::Value::Array(existing)), serde_json::Value::Array(mut incoming)) => {
                existing.append(&mut incoming);
            }
            (None, value) => {
                hooks.insert(event, value);
            }
            _ => {}
        }
    }
}

/// Merge our hook entries into a `{"hooks": {<Event>: [...]}}` JSON object,
/// replacing any existing groups that match our signature.
fn merge_hooks(
    root: &mut serde_json::Value,
    entries: &[(&str, serde_json::Value)],
    host: &str,
    uninstall: bool,
) -> usize {
    let hooks_obj = ensure_hooks_object(root);
    let mut removed = 0usize;
    for (event, entry) in entries {
        let slot = hooks_obj
            .entry((*event).to_string())
            .or_insert_with(|| serde_json::Value::Array(Vec::new()));
        if !slot.is_array() {
            *slot = serde_json::Value::Array(Vec::new());
        }
        let groups = slot.as_array_mut().expect("event forced to array");
        let before = groups.len();
        groups.retain(|g| !group_is_ours(g, host));
        removed += before - groups.len();
        if !uninstall {
            groups.push(entry.clone());
        }
    }
    hooks_obj.retain(|_, v| v.as_array().map(|a| !a.is_empty()).unwrap_or(true));
    removed
}

fn is_json_harness_installed(h: &Harness) -> bool {
    let Ok(content) = std::fs::read_to_string(&h.config_path) else {
        return false;
    };
    let Ok(mut v) = serde_json::from_str::<serde_json::Value>(&content) else {
        return false;
    };
    if h.id == "codex" {
        migrate_codex_root_events(&mut v);
    }
    let host = host_for_harness(h);
    hook_entries(h).iter().all(|(evt, _)| {
        v.get("hooks")
            .and_then(|h| h.get(evt))
            .and_then(|a| a.as_array())
            .is_some_and(|arr| arr.iter().any(|g| group_is_ours(g, host)))
    })
}

fn is_installed(h: &Harness) -> bool {
    match h.id {
        "opencode" => {
            h.config_path.exists()
                && std::fs::read_to_string(&h.config_path)
                    .map(|s| s.contains("tenex-edge") && s.contains("opencode"))
                    .unwrap_or(false)
        }
        "claude-code" | "codex" => is_json_harness_installed(h),
        _ => false,
    }
}

pub(super) async fn install(opts: InstallOpts) -> Result<()> {
    let all = harnesses();

    if opts.status {
        print_status(&all);
        return Ok(());
    }

    let selected = resolve_selection(&all, &opts)?;
    if selected.is_empty() {
        println!("No harnesses selected. Detected: {}", detected_list(&all));
        return Ok(());
    }

    let verb = if opts.uninstall {
        "Uninstalling from"
    } else {
        "Installing into"
    };
    let flag = if opts.dry_run { " (dry-run)" } else { "" };

    for h in selected {
        println!("\n{} {}{flag}", verb.bold(), h.display.cyan().bold());
        match h.id {
            "claude-code" | "codex" => install_json_harness(h, &opts)?,
            "opencode" => install_opencode(h, &opts)?,
            _ => {}
        }
    }

    if opts.dry_run {
        println!("\n{}", "(dry run; nothing was written)".dimmed());
    } else if !opts.uninstall {
        println!("\nDone. Restart any open harness sessions to pick up the hooks.");
    }
    Ok(())
}

fn print_status(all: &[Harness]) {
    println!("{}", "tenex-edge harness status".bold());
    for h in all {
        let detected = if h.detected {
            "detected".green().to_string()
        } else {
            "not detected".dimmed().to_string()
        };
        let installed = if is_installed(h) {
            "installed".green().to_string()
        } else {
            "-".dimmed().to_string()
        };
        println!(
            "  {:<12} {:<14} {:<10} {}",
            h.display.cyan(),
            detected,
            installed,
            h.config_path.display().to_string().dimmed()
        );
    }
}

fn detected_list(all: &[Harness]) -> String {
    let detected = all
        .iter()
        .filter(|h| h.detected)
        .map(|h| h.id)
        .collect::<Vec<_>>();
    if detected.is_empty() {
        "(none)".to_string()
    } else {
        detected.join(", ")
    }
}

fn resolve_selection<'a>(all: &'a [Harness], opts: &InstallOpts) -> Result<Vec<&'a Harness>> {
    if let Some(ids) = &opts.harness {
        let wanted = ids
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();
        let unknown = wanted
            .iter()
            .copied()
            .filter(|id| !all.iter().any(|h| h.id == *id))
            .collect::<Vec<_>>();
        if !unknown.is_empty() {
            bail!(
                "unknown harness id(s): {}. Known: {}",
                unknown.join(", "),
                all.iter().map(|h| h.id).collect::<Vec<_>>().join(", ")
            );
        }
        return Ok(all.iter().filter(|h| wanted.contains(&h.id)).collect());
    }

    if opts.all {
        return Ok(all.iter().filter(|h| h.detected).collect());
    }

    if io::stdin().is_terminal() && io::stdout().is_terminal() {
        return interactive_select(all);
    }

    Ok(all.iter().filter(|h| h.detected).collect())
}

fn interactive_select(all: &[Harness]) -> Result<Vec<&Harness>> {
    let mut choices = all
        .iter()
        .enumerate()
        .map(|(index, h)| HarnessChoice {
            index,
            selected: h.detected,
        })
        .collect::<Vec<_>>();

    if !run_selector(all, &mut choices)? {
        return Ok(Vec::new());
    }

    Ok(choices
        .into_iter()
        .filter(|c| c.selected)
        .map(|c| &all[c.index])
        .collect())
}

fn run_selector(all: &[Harness], choices: &mut [HarnessChoice]) -> Result<bool> {
    let _guard = TerminalGuard::enter()?;
    let mut active = 0usize;

    loop {
        render_selector(all, choices, active)?;
        match event::read()? {
            TermEvent::Key(key) => match key.code {
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Ok(false);
                }
                KeyCode::Esc | KeyCode::Char('q') => return Ok(false),
                KeyCode::Enter => return Ok(true),
                KeyCode::Up | KeyCode::Char('k') => {
                    active = active.saturating_sub(1);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if active + 1 < choices.len() {
                        active += 1;
                    }
                }
                KeyCode::Char(' ') => {
                    choices[active].selected = !choices[active].selected;
                }
                _ => {}
            },
            TermEvent::Resize(_, _) => {}
            _ => {}
        }
    }
}

fn render_selector(all: &[Harness], choices: &[HarnessChoice], active: usize) -> Result<()> {
    let mut out = io::stdout();
    execute!(out, MoveTo(0, 0), Clear(ClearType::All))?;
    writeln!(out, "Install tenex-edge hooks")?;
    writeln!(out, "Use up/down to move, space to toggle, enter to apply.")?;
    writeln!(out)?;

    for (idx, choice) in choices.iter().enumerate() {
        let h = &all[choice.index];
        let cursor = if idx == active { ">" } else { " " };
        let mark = if choice.selected { "[x]" } else { "[ ]" };
        let detected = if h.detected {
            "detected".green().to_string()
        } else {
            "not detected".dimmed().to_string()
        };
        let installed = if is_installed(h) {
            "installed".green().to_string()
        } else {
            "-".dimmed().to_string()
        };
        if idx == active {
            writeln!(
                out,
                "{} {} {}  {:<12} {:<14} {}",
                cursor.bold(),
                mark.bold(),
                h.display.bold(),
                detected,
                installed,
                h.config_path.display().to_string().dimmed()
            )?;
        } else {
            writeln!(
                out,
                "{cursor} {mark} {}  {:<12} {:<14} {}",
                h.display,
                detected,
                installed,
                h.config_path.display().to_string().dimmed()
            )?;
        }
    }

    out.flush()?;
    Ok(())
}

fn install_json_harness(h: &Harness, opts: &InstallOpts) -> Result<()> {
    let mut root = read_json_or_default(&h.config_path)?;
    if h.id == "codex" {
        migrate_codex_root_events(&mut root);
    }
    let entries = hook_entries(h);
    let removed = merge_hooks(&mut root, &entries, host_for_harness(h), opts.uninstall);

    if opts.dry_run {
        let action = if opts.uninstall {
            format!("would remove {removed} hook group(s)")
        } else {
            format!("would write {} hook group(s)", entries.len())
        };
        println!("  {action} in {}", h.config_path.display());
        print_json_preview(&root)?;
        return Ok(());
    }

    write_json(&h.config_path, &root)?;
    if opts.uninstall {
        println!("  removed {removed} hook group(s)");
    } else {
        println!("  wrote {}", h.config_path.display());
    }
    Ok(())
}

fn install_opencode(h: &Harness, opts: &InstallOpts) -> Result<()> {
    if opts.uninstall {
        if !h.config_path.exists() {
            println!("  nothing to remove");
            return Ok(());
        }
        if opts.dry_run {
            println!("  would remove {}", h.config_path.display());
        } else {
            std::fs::remove_file(&h.config_path)?;
            println!("  removed {}", h.config_path.display());
        }
        return Ok(());
    }

    if opts.dry_run {
        println!(
            "  would write {} ({} bytes)",
            h.config_path.display(),
            OPENCODE_PLUGIN_TS.len()
        );
    } else {
        write_text(&h.config_path, OPENCODE_PLUGIN_TS)?;
        println!("  wrote {}", h.config_path.display());
    }
    Ok(())
}

fn read_json_or_default(path: &Path) -> Result<serde_json::Value> {
    let mut root = match std::fs::read_to_string(path) {
        Ok(content) if content.trim().is_empty() => serde_json::json!({}),
        Ok(content) => serde_json::from_str(&content)
            .with_context(|| format!("{} is not valid JSON", path.display()))?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => serde_json::json!({}),
        Err(e) => return Err(e).with_context(|| format!("reading {}", path.display())),
    };
    ensure_object(&mut root);
    Ok(root)
}

fn print_json_preview(v: &serde_json::Value) -> Result<()> {
    let pretty = serde_json::to_string_pretty(v)?;
    for line in pretty.lines() {
        println!("    {line}");
    }
    Ok(())
}

fn write_json(path: &Path, v: &serde_json::Value) -> Result<()> {
    let pretty = serde_json::to_string_pretty(v)?;
    write_text(path, &(pretty + "\n"))
}

fn write_text(path: &Path, text: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, text)?;
    Ok(())
}

struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> Result<Self> {
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, Hide)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(io::stdout(), Show, LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn harness(id: &'static str, path: PathBuf) -> Harness {
        Harness {
            id,
            display: id,
            config_path: path,
            detected: true,
        }
    }

    #[test]
    fn merge_hooks_preserves_foreign_groups_and_replaces_ours() {
        let mut root = serde_json::json!({
            "hooks": {
                "UserPromptSubmit": [
                    {
                        "hooks": [{
                            "type": "command",
                            "command": "pc hook inject --harness codex",
                            "timeout": 30
                        }]
                    },
                    {
                        "hooks": [{
                            "type": "command",
                            "command": "tenex-edge hook --host codex --type old",
                            "timeout": 1
                        }]
                    }
                ]
            }
        });

        merge_hooks(&mut root, &codex_hook_entries(), "codex", false);

        let groups = root
            .pointer("/hooks/UserPromptSubmit")
            .and_then(|v| v.as_array())
            .unwrap();
        assert_eq!(groups.len(), 2);
        assert!(groups.iter().any(|g| {
            g.pointer("/hooks/0/command")
                .and_then(|v| v.as_str())
                .is_some_and(|c| c == "pc hook inject --harness codex")
        }));
        assert!(groups.iter().any(|g| {
            g.pointer("/hooks/0/command")
                .and_then(|v| v.as_str())
                .is_some_and(|c| c == "tenex-edge hook --host codex --type user-prompt-submit")
        }));
    }

    #[test]
    fn uninstall_removes_ours_and_empty_events_only() {
        let mut root = serde_json::json!({
            "hooks": {
                "Stop": [
                    {
                        "hooks": [{
                            "type": "command",
                            "command": "tenex-edge hook --host codex --type stop",
                            "timeout": 30
                        }]
                    }
                ],
                "UserPromptSubmit": [
                    {
                        "hooks": [{
                            "type": "command",
                            "command": "pc hook inject --harness codex",
                            "timeout": 30
                        }]
                    },
                    {
                        "hooks": [{
                            "type": "command",
                            "command": "tenex-edge hook --host codex --type user-prompt-submit",
                            "timeout": 30
                        }]
                    }
                ]
            }
        });

        let removed = merge_hooks(&mut root, &codex_hook_entries(), "codex", true);

        assert_eq!(removed, 2);
        assert!(root.pointer("/hooks/Stop").is_none());
        let groups = root
            .pointer("/hooks/UserPromptSubmit")
            .and_then(|v| v.as_array())
            .unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(
            groups[0]
                .pointer("/hooks/0/command")
                .and_then(|v| v.as_str()),
            Some("pc hook inject --harness codex")
        );
    }

    #[test]
    fn codex_root_events_are_migrated_under_hooks() {
        let mut root = serde_json::json!({
            "Stop": [{
                "hooks": [{
                    "type": "command",
                    "command": "foreign stop",
                    "timeout": 1
                }]
            }],
            "hooks": {
                "Stop": [{
                    "hooks": [{
                        "type": "command",
                        "command": "existing stop",
                        "timeout": 1
                    }]
                }]
            }
        });

        migrate_codex_root_events(&mut root);

        assert!(root.get("Stop").is_none());
        let groups = root
            .pointer("/hooks/Stop")
            .and_then(|v| v.as_array())
            .unwrap();
        assert_eq!(groups.len(), 2);
    }

    #[test]
    fn write_json_creates_parent_directories() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("a/b/hooks.json");
        write_json(&path, &serde_json::json!({"hooks": {}})).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn status_detects_installed_codex_hooks() {
        let temp = tempfile::tempdir().unwrap();
        let h = harness("codex", temp.path().join("hooks.json"));
        let mut root = serde_json::json!({});
        merge_hooks(&mut root, &codex_hook_entries(), "codex", false);
        write_json(&h.config_path, &root).unwrap();

        assert!(is_installed(&h));
    }
}
