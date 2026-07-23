use std::fmt::Write as _;

use super::{esc_attr, render_channel};
use crate::fabric_context::model::{ChannelBlock, WorkspaceRow};

pub(super) fn render_workspace_block(
    out: &mut String,
    workspace: &WorkspaceRow,
    root: Option<&ChannelBlock>,
    channels: &[ChannelBlock],
) {
    let _ = write!(
        out,
        "\n\n  <workspace name=\"{}\"",
        esc_attr(&workspace.name)
    );
    if !workspace.about.is_empty() {
        let _ = write!(out, " about=\"{}\"", esc_attr(&workspace.about));
    }
    out.push('>');
    if let Some(root) = root {
        render_channel(out, root, 4);
    }
    if !channels.is_empty() {
        out.push_str("\n    <channels>");
        for channel in channels {
            render_channel(out, channel, 6);
        }
        out.push_str("\n    </channels>");
    }
    out.push_str("\n  </workspace>");
}
