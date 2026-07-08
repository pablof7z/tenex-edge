use super::{attempt, ChannelCtx, Nip29Provider};
use std::collections::{HashMap, HashSet};

pub(super) struct Outcome {
    pub(super) repaired: bool,
    pub(super) degraded_reason: Option<String>,
}

pub(super) async fn ensure_invariants(
    provider: &Nip29Provider,
    ctx: &ChannelCtx<'_>,
    mgmt_pubkey: &str,
    parent_admins: &[String],
    roles: &HashMap<String, String>,
    members: &HashSet<String>,
) -> Outcome {
    let mut repaired = false;
    let mut failures = Vec::new();
    let mut required_admins: Vec<String> = vec![mgmt_pubkey.to_string()];
    if ctx.repair_whitelisted_admins {
        required_admins.extend(provider.whitelisted_pubkeys.iter().cloned());
    }
    for pk in parent_admins {
        if !required_admins.contains(pk) {
            required_admins.push(pk.clone());
        }
    }
    for pk in &required_admins {
        if roles.get(pk.as_str()).map(String::as_str) == Some("admin") {
            continue;
        }
        if provider
            .grant_admin_confirmed(ctx.channel, pk)
            .await
            .is_confirmed()
        {
            repaired = true;
        } else {
            eprintln!(
                "[daemon] ensure_channel_ready: admin grant for {pk} in {:?} not confirmed on the relay",
                ctx.channel
            );
            failures.push(format!("admin grant for {pk} not confirmed"));
        }
    }
    if !failures.is_empty() {
        return Outcome {
            repaired,
            degraded_reason: Some(attempt::reason(&failures)),
        };
    }
    if ctx.expect_member.is_empty() {
        return Outcome {
            repaired,
            degraded_reason: None,
        };
    }

    let expect_already_admin = mgmt_pubkey == ctx.expect_member
        || provider
            .whitelisted_pubkeys
            .iter()
            .any(|pk| pk == ctx.expect_member)
        || parent_admins.iter().any(|pk| pk == ctx.expect_member);
    if !expect_already_admin
        && !members.contains(ctx.expect_member)
        && !roles.contains_key(ctx.expect_member)
    {
        if provider
            .grant_member_confirmed(ctx.channel, ctx.expect_member)
            .await
            .is_confirmed()
        {
            repaired = true;
        } else {
            eprintln!(
                "[daemon] ensure_channel_ready: member add for {} in {:?} not confirmed on the relay",
                ctx.expect_member, ctx.channel
            );
            failures.push(format!(
                "member add for {} not confirmed",
                ctx.expect_member
            ));
        }
    } else {
        sync_local_member_mirror(provider, ctx, roles);
    }
    Outcome {
        repaired,
        degraded_reason: (!failures.is_empty()).then(|| attempt::reason(&failures)),
    }
}

fn sync_local_member_mirror(
    provider: &Nip29Provider,
    ctx: &ChannelCtx<'_>,
    roles: &HashMap<String, String>,
) {
    let locally =
        provider.with_store(
            |s| match s.is_channel_member(ctx.channel, ctx.expect_member) {
                Ok(present) => present,
                Err(e) => {
                    tracing::error!(
                        channel = ctx.channel,
                        pubkey = ctx.expect_member,
                        error = %e,
                        "ensure_channel_ready: is_channel_member probe failed; re-syncing"
                    );
                    false
                }
            },
        );
    if locally {
        return;
    }
    let role = roles
        .get(ctx.expect_member)
        .map(String::as_str)
        .unwrap_or("member");
    provider.with_store(|s| {
        if let Err(e) = s.upsert_channel_member(
            ctx.channel,
            ctx.expect_member,
            role,
            crate::util::now_secs(),
        ) {
            tracing::error!(
                channel = ctx.channel,
                pubkey = ctx.expect_member,
                error = %e,
                "ensure_channel_ready: local member mirror sync failed"
            );
        }
    });
}
