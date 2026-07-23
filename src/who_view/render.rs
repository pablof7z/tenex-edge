use super::model::*;
use std::fmt::Write as _;

pub(super) fn render_agent_who(view: &AgentWhoView) -> String {
    let mut out = String::from("<mosaico>");
    let _ = write!(
        out,
        "\n  <self name=\"@{}\" host=\"{}\" headless=\"{}\" />",
        attr(&view.self_name),
        attr(&view.self_host),
        if view.headless { "on" } else { "off" },
    );
    render_hosts(&mut out, &view.hosts);
    out.push_str("\n  <workspaces>");
    for workspace in &view.workspaces {
        render_workspace(&mut out, workspace);
    }
    out.push_str("\n  </workspaces>\n</mosaico>");
    out
}

fn render_hosts(out: &mut String, hosts: &[HostView]) {
    out.push_str("\n  <hosts>");
    for host in hosts {
        let _ = write!(out, "\n    <host name=\"{}\">", attr(&host.name));
        out.push_str("\n      <agents>");
        for agent in &host.agents {
            let _ = write!(out, "\n        <agent ref=\"{}\"", attr(&agent.reference));
            if !agent.about.is_empty() {
                let _ = write!(out, " about=\"{}\"", attr(&agent.about));
            }
            out.push_str(" />");
        }
        out.push_str("\n      </agents>\n    </host>");
    }
    out.push_str("\n  </hosts>");
}

fn render_workspace(out: &mut String, workspace: &WorkspaceView) {
    let _ = write!(out, "\n    <workspace name=\"{}\"", attr(&workspace.name));
    if !workspace.about.is_empty() {
        let _ = write!(out, " about=\"{}\"", attr(&workspace.about));
    }
    let _ = write!(
        out,
        " members=\"{}\" hosts=\"{}\"",
        workspace.member_count,
        attr(&workspace.hosts.join(","))
    );
    if !workspace.expanded {
        out.push_str(" />");
        return;
    }
    out.push('>');
    render_members(out, &workspace.members, 6);
    out.push_str("\n      <channels>");
    for channel in &workspace.channels {
        render_channel(out, channel, 8);
    }
    out.push_str("\n      </channels>\n    </workspace>");
}

fn render_channel(out: &mut String, channel: &ChannelView, indent: usize) {
    let pad = " ".repeat(indent);
    let _ = write!(
        out,
        "\n{pad}<channel name=\"{}\" id=\"{}\" members=\"{}\"",
        attr(&channel.name),
        attr(&channel.id),
        channel.member_count
    );
    if !channel.about.is_empty() {
        let _ = write!(out, " about=\"{}\"", attr(&channel.about));
    }
    if !channel.expanded {
        out.push_str(" />");
        return;
    }
    out.push('>');
    render_members(out, &channel.members, indent + 2);
    for child in &channel.children {
        render_channel(out, child, indent + 2);
    }
    let _ = write!(out, "\n{pad}</channel>");
}

fn render_members(out: &mut String, members: &[MemberView], indent: usize) {
    if members.is_empty() {
        return;
    }
    let pad = " ".repeat(indent);
    let child_pad = " ".repeat(indent + 2);
    let _ = write!(out, "\n{pad}<members>");
    for member in members {
        let tag = match member.kind {
            MemberKind::Agent => "agent",
            MemberKind::Human => "human",
        };
        let _ = write!(
            out,
            "\n{child_pad}<{tag} name=\"@{}\" state=\"{}\"",
            attr(member.name.trim_start_matches('@')),
            member.state.as_str()
        );
        if !member.status.is_empty() {
            let _ = write!(out, " status=\"{}\"", attr(&member.status));
        }
        let _ = write!(out, " since=\"{}\" />", attr(&member.since));
    }
    let _ = write!(out, "\n{pad}</members>");
}

fn attr(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
