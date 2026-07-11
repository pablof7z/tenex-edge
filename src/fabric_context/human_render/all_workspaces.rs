use super::{render_agents, render_human_workspace};
use crate::fabric_context::model::{shared_agents, workspace_agents, FabricView};

pub(in crate::fabric_context) fn render_human_views(views: &[FabricView], color: bool) -> String {
    let shared = shared_agents(views);
    let mut out = String::new();
    render_agents(
        &mut out,
        &shared,
        "Available agents (all workspaces)",
        color,
    );

    for view in views {
        let additions = workspace_agents(view, &shared);
        render_human_workspace(
            &mut out,
            view,
            &additions,
            "Workspace-specific agents",
            color,
        );
    }

    out
}
