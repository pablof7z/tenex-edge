use crate::who_snapshot::OtherRootSummary;
use owo_colors::OwoColorize as _;
use std::fmt::Write as _;

pub(super) fn append_other_roots(out: &mut String, other_roots: &[OtherRootSummary], color: bool) {
    if other_roots.is_empty() {
        return;
    }
    let _ = writeln!(
        out,
        "{}",
        style("Other workspaces", color, HumanStyle::Header)
    );
    for root in other_roots {
        let name = style(&root.root, color, HumanStyle::Root);
        let agents = root
            .agents
            .iter()
            .map(|agent| style(&format!("@{agent}"), color, HumanStyle::Agent))
            .collect::<Vec<_>>()
            .join(", ");
        let count = format!(
            "{} agent{}",
            root.agent_count,
            if root.agent_count == 1 { "" } else { "s" }
        );
        let about = root
            .about
            .as_deref()
            .filter(|about| !about.trim().is_empty())
            .map(|about| format!(" - {about}"))
            .unwrap_or_default();
        if agents.is_empty() {
            let _ = writeln!(out, "  {}  {}{}", name, dim(&count, color), about);
        } else {
            let _ = writeln!(
                out,
                "  {}  {}  {}{}",
                name,
                dim(&count, color),
                agents,
                about
            );
        }
    }
    out.push('\n');
}

#[derive(Clone, Copy)]
enum HumanStyle {
    Agent,
    Header,
    Root,
}

fn style(text: &str, color: bool, style: HumanStyle) -> String {
    if !color {
        return text.to_string();
    }
    match style {
        HumanStyle::Agent => text.cyan().to_string(),
        HumanStyle::Header => text.bold().to_string(),
        HumanStyle::Root => text.blue().bold().to_string(),
    }
}

fn dim(text: &str, color: bool) -> String {
    if color {
        text.dimmed().to_string()
    } else {
        text.to_string()
    }
}
