use std::collections::{BTreeMap, BTreeSet};

use super::{members, message_rows, presence_rows};
use crate::fabric_context::capture::{ChannelCap, ViewInputs, WorkspaceCap};
use crate::fabric_context::model::{ChannelBlock, WorkspaceView};
use crate::util::relative_time;

pub(super) fn workspace_rows(
    inputs: &ViewInputs,
    cursor: u64,
    now: u64,
    full: bool,
) -> Option<Vec<WorkspaceView>> {
    let rows = inputs
        .meta
        .workspaces
        .iter()
        .filter_map(|workspace| workspace_row(inputs, workspace, cursor, now, full))
        .collect::<Vec<_>>();
    (full || !rows.is_empty()).then_some(rows)
}

fn workspace_row(
    inputs: &ViewInputs,
    workspace: &WorkspaceCap,
    cursor: u64,
    now: u64,
    full: bool,
) -> Option<WorkspaceView> {
    let caps = workspace
        .channels
        .iter()
        .map(|channel| (channel.reference.clone(), channel))
        .collect::<BTreeMap<_, _>>();
    let selected = if full {
        full_channel_ids(inputs, workspace, &caps)
    } else {
        delta_channel_ids(inputs, workspace, cursor, now)
    };
    let workspace_changed = !full && workspace.updated_at > cursor && workspace.updated_at <= now;
    if !full && selected.is_empty() && !workspace_changed {
        return None;
    }
    let content =
        if full && (inputs.meta.self_row.is_none() || workspace_is_expanded(inputs, workspace)) {
            selected.clone()
        } else if full {
            BTreeSet::new()
        } else {
            selected.clone()
        };
    let selected = if full {
        selected
    } else {
        with_ancestors(&selected, &caps)
    };
    let blocks = selected
        .iter()
        .filter_map(|id| {
            caps.get(id).map(|channel| {
                channel_block(inputs, channel, content.contains(id), full, cursor, now)
            })
        })
        .collect();
    let (root, channels) = crate::fabric_context::tree::arrange(&workspace.summary.name, blocks);
    Some(WorkspaceView {
        name: workspace.summary.name.clone(),
        about: workspace.summary.about.clone(),
        hosts: workspace.hosts.clone(),
        root,
        channels,
    })
}

fn full_channel_ids(
    inputs: &ViewInputs,
    workspace: &WorkspaceCap,
    caps: &BTreeMap<String, &ChannelCap>,
) -> BTreeSet<String> {
    if inputs.meta.self_row.is_none() {
        return caps.keys().cloned().collect();
    }
    let root = workspace.summary.channel.clone();
    let mut selected = BTreeSet::new();
    if caps.contains_key(&root) {
        selected.insert(root.clone());
    }
    if workspace_is_expanded(inputs, workspace) {
        add_visible_children(inputs, &root, caps, &mut selected);
    }
    selected
}

fn workspace_is_expanded(inputs: &ViewInputs, workspace: &WorkspaceCap) -> bool {
    workspace
        .channels
        .iter()
        .any(|channel| inputs.meta.active_channels.contains(&channel.h))
}

fn add_visible_children(
    inputs: &ViewInputs,
    parent: &str,
    caps: &BTreeMap<String, &ChannelCap>,
    selected: &mut BTreeSet<String>,
) {
    for (id, channel) in caps
        .iter()
        .filter(|(id, _)| parent_id(id).is_some_and(|candidate| candidate == parent))
    {
        selected.insert(id.clone());
        if is_member(inputs, &channel.h) {
            add_visible_children(inputs, id, caps, selected);
        }
    }
}

fn delta_channel_ids(
    inputs: &ViewInputs,
    workspace: &WorkspaceCap,
    cursor: u64,
    now: u64,
) -> BTreeSet<String> {
    workspace
        .channels
        .iter()
        .filter(|channel| {
            (channel.updated_at > cursor && channel.updated_at <= now)
                || !presence_rows(inputs, &channel.h, cursor, now).is_empty()
                || inputs
                    .messages
                    .channels
                    .get(&channel.h)
                    .is_some_and(|bundle| !message_rows(bundle, cursor, now).0.is_empty())
        })
        .map(|channel| channel.reference.clone())
        .collect()
}

fn with_ancestors(
    content: &BTreeSet<String>,
    caps: &BTreeMap<String, &ChannelCap>,
) -> BTreeSet<String> {
    let mut selected = content.clone();
    for id in content {
        let mut current = id.as_str();
        while let Some(parent) = parent_id(current) {
            if !caps.contains_key(parent) {
                break;
            }
            selected.insert(parent.to_string());
            current = parent;
        }
    }
    selected
}

fn channel_block(
    inputs: &ViewInputs,
    channel: &ChannelCap,
    content: bool,
    full: bool,
    cursor: u64,
    now: u64,
) -> ChannelBlock {
    let member = is_member(inputs, &channel.h);
    let active = inputs.meta.active_channels.contains(&channel.h);
    let member_count = (!active).then(|| member_count(inputs, &channel.h));
    let last_active = (!member)
        .then(|| channel.latest_message_at.map(|at| relative_time(at, now)))
        .flatten();
    let members = if content && full && (member || inputs.meta.self_row.is_none()) {
        members::member_rows(inputs, &channel.h, now)
    } else {
        Vec::new()
    };
    let presence = if content {
        presence_rows(inputs, &channel.h, cursor, now)
    } else {
        Vec::new()
    };
    let (messages, omitted) = if content {
        inputs
            .messages
            .channels
            .get(&channel.h)
            .map(|bundle| message_rows(bundle, cursor, now))
            .unwrap_or_default()
    } else {
        Default::default()
    };
    ChannelBlock {
        name: channel.name.clone(),
        id: channel.reference.clone(),
        about: channel.about.clone(),
        member_count,
        last_active,
        members,
        presence,
        children: Vec::new(),
        messages,
        omitted,
    }
}

fn is_member(inputs: &ViewInputs, channel: &str) -> bool {
    !inputs.meta.self_pubkey.is_empty()
        && inputs
            .members
            .roster
            .get(channel)
            .is_some_and(|members| members.contains_key(&inputs.meta.self_pubkey))
}

fn member_count(inputs: &ViewInputs, channel: &str) -> usize {
    inputs
        .members
        .roster
        .get(channel)
        .map(|members| {
            members
                .keys()
                .filter(|pubkey| !inputs.members.backend.contains(*pubkey))
                .count()
        })
        .unwrap_or_default()
}

fn parent_id(id: &str) -> Option<&str> {
    id.rsplit_once('/')
        .map(|(parent, _)| parent)
        .filter(|parent| !parent.is_empty())
}
