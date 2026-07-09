use super::super::*;

const PROJECT_MEMBER_READY_TIMEOUT: Duration = Duration::from_secs(90);

// ── project_list ─────────────────────────────────────────────────────────────

/// List NIP-29 groups: refresh the local cache via the provider (which fetches
/// kind:39000 from the relay), then return the read-model list.
pub async fn rpc_project_list(state: &Arc<DaemonState>) -> Result<serde_json::Value> {
    // Provider fetches kind:39000 from the relay and upserts relay_channels.
    // Best-effort: a relay timeout must not prevent returning cached results.
    state.provider.refresh_project_list().await.ok();

    let projects: Vec<serde_json::Value> = state
        .with_store(|s| s.list_projects_read_model())
        .unwrap_or_default()
        .into_iter()
        .map(|c| serde_json::json!({ "slug": c.channel_h, "about": c.about }))
        .collect();

    Ok(serde_json::json!({ "projects": projects }))
}

// ── project_edit ─────────────────────────────────────────────────────────────

/// Publish a NIP-29 kind:9002 (edit-metadata) event signed by the human user's
/// nsec. The relay validates admin rights and updates its kind:39000 accordingly.
pub async fn rpc_project_edit(
    state: &Arc<DaemonState>,
    params: &serde_json::Value,
) -> Result<serde_json::Value> {
    #[derive(serde::Deserialize)]
    struct P {
        project: String,
        description: String,
    }
    let p: P = serde_json::from_value(params.clone()).context("project_edit params")?;

    let user_keys = state.management_keys()?;

    // NIP-29 edit-metadata: the wire shape lives in the nip29 lifecycle module.
    // The relay validates admin rights and re-publishes kind:39000.
    let builder = crate::fabric::nip29::lifecycle::group_edit_metadata(&p.project, &p.description)?;
    let event_id = state.transport.publish_signed(builder, &user_keys).await?;

    let confirmed = wait_for_channel_about(state, &p.project, &p.description).await;

    Ok(serde_json::json!({
        "event_id": event_id.to_hex(),
        "project": p.project,
        "confirmed": confirmed,
    }))
}

// ── project_members ──────────────────────────────────────────────────────────

/// Return the cached NIP-29 membership roster for a project. Before reading the
/// cache, refresh admin/member snapshots from the relay so interactive project
/// edits start from relay state rather than only local optimistic state.
pub async fn rpc_project_members(
    state: &Arc<DaemonState>,
    params: &serde_json::Value,
) -> Result<serde_json::Value> {
    #[derive(serde::Deserialize)]
    struct P {
        project: String,
    }
    let p: P = serde_json::from_value(params.clone()).context("project_members params")?;
    refresh_project_members_cache(state, &p.project).await;

    let member_pubkeys = state
        .with_store(|s| s.list_channel_members(&p.project))
        .unwrap_or_default()
        .into_iter()
        .map(|m| m.pubkey)
        .collect::<Vec<_>>();
    crate::profile::warm(state, &member_pubkeys).await;

    let members = state
        .with_store(|s| s.list_channel_members(&p.project))
        .unwrap_or_default()
        .into_iter()
        .map(|m| {
            let slug = state
                .with_store(|s| s.resolve_slug_for_pubkey(&m.pubkey).ok().flatten())
                .unwrap_or_default();
            serde_json::json!({ "pubkey": m.pubkey, "slug": slug, "role": m.role })
        })
        .collect::<Vec<_>>();

    Ok(serde_json::json!({
        "project": p.project,
        "members": members,
    }))
}

async fn wait_for_channel_about(
    state: &Arc<DaemonState>,
    project: &str,
    description: &str,
) -> bool {
    for _ in 0..20 {
        state.provider.refresh_project_list().await.ok();
        let matches = state.with_store(|s| {
            s.channel_meta_read_model(project)
                .ok()
                .flatten()
                .map(|c| c.about)
                .as_deref()
                == Some(description)
        });
        if matches {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    false
}

/// `channel add <pubkey|npub|nip05> <channel> [--admin]` — add a human member to
/// a channel. Resolves the project-relative channel reference to its opaque `h`,
/// ensures the channel is ready (management is admin), then publishes an explicit
/// NIP-29 kind:9000 put-user granting `member` (or `admin` when `admin`), read
/// back for confirmation.
pub async fn rpc_project_add(
    state: &Arc<DaemonState>,
    params: &serde_json::Value,
) -> Result<serde_json::Value> {
    #[derive(serde::Deserialize)]
    struct P {
        project: String,
        pubkey: String,
        #[serde(default)]
        admin: bool,
    }
    let p: P = serde_json::from_value(params.clone()).context("project_add params")?;

    let channel_h = match resolve_add_channel(state, params, &p.project)? {
        ChannelResolution::Unique(h) => h,
        ChannelResolution::Ambiguous(refs) => {
            return Ok(serde_json::json!({ "ambiguous": refs, "reference": p.project }));
        }
        ChannelResolution::NotFound => {
            anyhow::bail!("no channel matching {:?} in this project", p.project)
        }
    };

    let pubkey_hex = resolve_project_member_pubkey_hex(&p.pubkey).await?;
    let role = if p.admin { "admin" } else { "member" };
    log_nip29_role_decision(
        &channel_h,
        &pubkey_hex,
        role,
        "rpc_project_add manual add via confirmed provider mutation",
    );

    let parent_hint = state
        .with_store(|s| s.channel_parent(&channel_h).unwrap_or(None))
        .filter(|parent| !parent.is_empty());
    let ready = state
        .provider
        .ensure_channel_ready(crate::fabric::nip29::readiness::ChannelCtx {
            channel: &channel_h,
            expect_member: &pubkey_hex,
            parent_hint: parent_hint.as_deref(),
            name: None,
            repair_whitelisted_admins: true,
        });
    let gate = match tokio::time::timeout(PROJECT_MEMBER_READY_TIMEOUT, ready).await {
        Ok(gate) => gate,
        Err(_) => crate::fabric::nip29::readiness::ChannelGate::Degraded,
    };
    if matches!(gate, crate::fabric::nip29::readiness::ChannelGate::Degraded) {
        anyhow::bail!(
            "project_add could not verify channel {} readiness within {}s",
            channel_h,
            PROJECT_MEMBER_READY_TIMEOUT.as_secs()
        );
    }

    // Explicit grant of the requested role. `ensure_channel_ready` above only
    // guarantees the management key is admin; the member/admin put-user is
    // published here and read back for confirmation.
    let outcome = if p.admin {
        state
            .provider
            .grant_admin_confirmed(&channel_h, &pubkey_hex)
            .await
    } else {
        state
            .provider
            .grant_member_confirmed(&channel_h, &pubkey_hex)
            .await
    };
    if !outcome.is_confirmed() {
        anyhow::bail!(
            "could not confirm {} as {role} of channel {channel_h}",
            crate::util::pubkey_short(&pubkey_hex)
        );
    }

    Ok(serde_json::json!({
        "project": channel_h,
        "pubkey": pubkey_hex,
        "role": role,
        "confirmed": true,
    }))
}

/// Resolve a project-relative channel reference for `channel add`, from the
/// caller's session anchor when present, else from the invoking directory (a
/// human running the verb from a project checkout). An exact `h` passes through.
fn resolve_add_channel(
    state: &Arc<DaemonState>,
    params: &serde_json::Value,
    reference: &str,
) -> Result<ChannelResolution> {
    let anchor = CallerAnchor::from_params(params);
    let root = match resolve_session_inner(state, &anchor, ResolveScope::Strict) {
        Ok(rec) => state.with_store(|s| project_root(s, &rec.channel_h)),
        Err(_) => {
            let cwd = anchor
                .cwd
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
            crate::project::resolve(&cwd)
                .context("channel add must run inside an agent session or project directory")?
        }
    };
    Ok(state.with_store(|s| resolve_channel_ref(s, &root, reference)))
}

// ── project_remove ───────────────────────────────────────────────────────────

/// Publish a NIP-29 kind:9001 (remove-user) event to remove a pubkey from the
/// group. Accepts hex, npub (bech32), or a NIP-05 address (user@domain.com).
pub async fn rpc_project_remove(
    state: &Arc<DaemonState>,
    params: &serde_json::Value,
) -> Result<serde_json::Value> {
    #[derive(serde::Deserialize)]
    struct P {
        project: String,
        pubkey: String,
    }
    let p: P = serde_json::from_value(params.clone()).context("project_remove params")?;

    let pubkey_hex = resolve_pubkey_hex(&p.pubkey).await?;

    let outcome = state
        .provider
        .remove_member_confirmed(&p.project, &pubkey_hex)
        .await;
    if outcome.is_rejected() {
        anyhow::bail!(
            "project_remove rejected for {} in {}",
            crate::util::pubkey_short(&pubkey_hex),
            p.project
        );
    }

    Ok(serde_json::json!({
        "project": p.project,
        "pubkey": pubkey_hex,
        "confirmed": outcome.is_confirmed(),
    }))
}
