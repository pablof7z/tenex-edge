pub(in crate::daemon::server::probe::validate) struct MembershipTarget {
    pub(super) channel_h: String,
    pub(super) pubkey: String,
    pub(super) require_admin: bool,
}

pub(in crate::daemon::server::probe::validate) fn membership_target(
    target: &str,
) -> Option<MembershipTarget> {
    colon_target(target, "member:", false)
        .or_else(|| colon_target(target, "membership:", false))
        .or_else(|| colon_target(target, "admin:", true))
        .or_else(|| path_target(target, "member/", false))
        .or_else(|| path_target(target, "membership/", false))
        .or_else(|| path_target(target, "admin/", true))
}

fn colon_target(target: &str, prefix: &str, require_admin: bool) -> Option<MembershipTarget> {
    let rest = target.strip_prefix(prefix)?;
    let (channel_h, pubkey) = rest.split_once(':')?;
    build_target(channel_h, pubkey, require_admin)
}

fn path_target(target: &str, prefix: &str, require_admin: bool) -> Option<MembershipTarget> {
    let rest = target.strip_prefix(prefix)?;
    let (channel_h, pubkey) = rest.split_once('/')?;
    build_target(channel_h, pubkey, require_admin)
}

fn build_target(channel_h: &str, pubkey: &str, require_admin: bool) -> Option<MembershipTarget> {
    (!channel_h.trim().is_empty() && !pubkey.trim().is_empty()).then(|| MembershipTarget {
        channel_h: channel_h.to_string(),
        pubkey: pubkey.to_string(),
        require_admin,
    })
}

pub(super) fn summary(
    channel_h: &str,
    pubkey: &str,
    require_admin: bool,
    role: &str,
    found: bool,
    snapshot: bool,
) -> String {
    let target_role = if require_admin { "admin" } else { "member" };
    if found && (!require_admin || role == "admin") {
        return format!("pubkey `{pubkey}` is {role} in channel `{channel_h}`");
    }
    if found {
        return format!(
            "pubkey `{pubkey}` is `{role}` in channel `{channel_h}`, not `{target_role}`"
        );
    }
    if snapshot {
        format!("pubkey `{pubkey}` is not in the hydrated `{channel_h}` membership snapshot")
    } else {
        format!("pubkey `{pubkey}` is not proven in channel `{channel_h}`")
    }
}

pub(super) fn reason(
    found: bool,
    require_admin: bool,
    role: &str,
    channel_found: bool,
    snapshot: bool,
) -> &'static str {
    if found && require_admin && role != "admin" {
        "membership row exists, but it is not an admin role"
    } else if found && !channel_found {
        "membership row exists, but channel metadata is not materialized"
    } else if found && !snapshot {
        "membership row exists, but complete admin/member snapshots are not hydrated"
    } else if !found && snapshot {
        "hydrated channel membership snapshot does not contain this pubkey"
    } else if !found && !channel_found {
        "channel metadata is not materialized and no membership row matched this pubkey"
    } else if !found {
        "membership snapshot is not fully hydrated, so absence is not proven"
    } else {
        ""
    }
}
