use super::AgentWhoInput;
use crate::state::{Channel, Status};
use crate::util::relative_time;
use crate::who_view::model::*;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn channels_by_parent(channels: Vec<Channel>) -> BTreeMap<String, Vec<Channel>> {
    let mut by_parent: BTreeMap<String, Vec<Channel>> = BTreeMap::new();
    for channel in channels
        .into_iter()
        .filter(|channel| !channel.is_archived())
    {
        by_parent
            .entry(channel.parent.clone())
            .or_default()
            .push(channel);
    }
    for children in by_parent.values_mut() {
        children.sort_by(|a, b| a.name.cmp(&b.name).then(a.channel_h.cmp(&b.channel_h)));
    }
    by_parent
}

pub(super) fn workspace_view(
    aggregation: &crate::who_aggregation::WhoAggregation,
    by_parent: &BTreeMap<String, Vec<Channel>>,
    root: &str,
    input: &AgentWhoInput<'_>,
) -> anyhow::Result<WorkspaceView> {
    let meta = aggregation.channel(root);
    Ok(WorkspaceView {
        name: root.to_string(),
        about: meta
            .map(|channel| channel.about.clone())
            .unwrap_or_default(),
        hosts: aggregation
            .backend_profiles_for_root(Some(root))
            .into_iter()
            .map(|profile| profile.host.clone())
            .filter(|host| !host.is_empty())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
        root: root_channel_view(aggregation, by_parent, root, input)?,
    })
}

fn root_channel_view(
    aggregation: &crate::who_aggregation::WhoAggregation,
    by_parent: &BTreeMap<String, Vec<Channel>>,
    root: &str,
    input: &AgentWhoInput<'_>,
) -> anyhow::Result<ChannelView> {
    let expanded = input.expanded_workspaces.contains(root);
    let member = aggregation.is_member(root, input.self_pubkey);
    let members = member_views(aggregation, root, input);
    let mut seen = BTreeSet::from([root.to_string()]);
    let children = if expanded {
        by_parent
            .get(root)
            .map(Vec::as_slice)
            .unwrap_or_default()
            .iter()
            .map(|child| {
                channel_view(
                    aggregation,
                    by_parent,
                    root,
                    &child.channel_h,
                    input,
                    &mut seen,
                )
            })
            .collect::<anyhow::Result<Vec<_>>>()?
    } else {
        Vec::new()
    };
    Ok(ChannelView {
        name: aggregation.channel_name(root).to_string(),
        id: aggregation.full_channel_ref(root)?,
        about: String::new(),
        member_count: members.len(),
        active: input.active_channels.contains(root),
        member,
        last_active: last_active(aggregation, root, member, input.now),
        expanded,
        members: if expanded { members } else { Vec::new() },
        children,
    })
}

fn channel_view(
    aggregation: &crate::who_aggregation::WhoAggregation,
    by_parent: &BTreeMap<String, Vec<Channel>>,
    workspace: &str,
    channel_h: &str,
    input: &AgentWhoInput<'_>,
    seen: &mut BTreeSet<String>,
) -> anyhow::Result<ChannelView> {
    if !seen.insert(channel_h.to_string()) {
        return Ok(empty_channel(workspace, channel_h));
    }
    let meta = aggregation.channel(channel_h);
    let name = aggregation.channel_name(channel_h).to_string();
    let members = member_views(aggregation, channel_h, input);
    let member = aggregation.is_member(channel_h, input.self_pubkey);
    let children = if member {
        by_parent
            .get(channel_h)
            .map(Vec::as_slice)
            .unwrap_or_default()
            .iter()
            .map(|child| {
                channel_view(
                    aggregation,
                    by_parent,
                    workspace,
                    &child.channel_h,
                    input,
                    seen,
                )
            })
            .collect::<anyhow::Result<Vec<_>>>()?
    } else {
        Vec::new()
    };
    Ok(ChannelView {
        name,
        id: aggregation.full_channel_ref(channel_h)?,
        about: meta
            .map(|channel| channel.about.clone())
            .unwrap_or_default(),
        member_count: members.len(),
        active: input.active_channels.contains(channel_h),
        member,
        last_active: last_active(aggregation, channel_h, member, input.now),
        expanded: member,
        members: if member { members } else { Vec::new() },
        children,
    })
}

fn empty_channel(workspace: &str, channel_h: &str) -> ChannelView {
    ChannelView {
        name: channel_h.to_string(),
        id: crate::channel_ref::format_channel_ref(workspace, &[channel_h.to_string()]),
        about: String::new(),
        member_count: 0,
        active: false,
        member: false,
        last_active: None,
        expanded: false,
        members: Vec::new(),
        children: Vec::new(),
    }
}

fn last_active(
    aggregation: &crate::who_aggregation::WhoAggregation,
    channel_h: &str,
    member: bool,
    now: u64,
) -> Option<String> {
    (!member)
        .then(|| aggregation.latest_message_at(channel_h))
        .flatten()
        .map(|at| relative_time(at, now))
}

fn member_views(
    aggregation: &crate::who_aggregation::WhoAggregation,
    channel: &str,
    input: &AgentWhoInput<'_>,
) -> Vec<MemberView> {
    let backend_pubkeys = aggregation
        .backend_profiles
        .iter()
        .map(|profile| profile.pubkey.clone())
        .collect::<BTreeSet<_>>();
    let statuses = aggregation
        .statuses_for(channel)
        .iter()
        .map(|status| (status.pubkey.clone(), status))
        .collect::<BTreeMap<_, _>>();
    aggregation
        .members_for(channel)
        .iter()
        .filter(|member| {
            member.pubkey != input.backend_pubkey
                && !backend_pubkeys.contains(&member.pubkey)
                && !aggregation
                    .profile(&member.pubkey)
                    .is_some_and(|profile| profile.is_backend)
        })
        .map(|member| {
            member_view(
                aggregation,
                &member.pubkey,
                statuses.get(&member.pubkey).copied(),
                input,
            )
        })
        .collect()
}

fn member_view(
    aggregation: &crate::who_aggregation::WhoAggregation,
    pubkey: &str,
    status: Option<&Status>,
    input: &AgentWhoInput<'_>,
) -> MemberView {
    let profile = aggregation.profile(pubkey);
    let is_agent = pubkey == input.self_pubkey
        || status.is_some()
        || profile.is_some_and(|row| !row.agent_slug.is_empty());
    let name = if pubkey == input.self_pubkey {
        input.self_name.to_string()
    } else {
        aggregation.pubkey_ref(pubkey, input.local_host)
    };
    let (state, text, since) = match status {
        Some(row) => {
            let presence = aggregation.public_presence(pubkey, row);
            (
                presence.state,
                presence.text(),
                relative_time(presence.state_since, input.now),
            )
        }
        None => (
            crate::session_state::SessionState::Offline,
            String::new(),
            "unknown".to_string(),
        ),
    };
    MemberView {
        kind: if is_agent {
            MemberKind::Agent
        } else {
            MemberKind::Human
        },
        name,
        state,
        status: text,
        since,
    }
}
