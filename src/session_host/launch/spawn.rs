use super::source::ResolvedSource;
use super::*;

pub struct DispatchedSpawn {
    pub endpoint: crate::session_host::transport::EndpointRef,
    pub pubkey: String,
}

pub(crate) struct HostedSpawn {
    pub(crate) endpoint: crate::session_host::transport::SessionEndpoint,
    pub(crate) pubkey: String,
}

pub(crate) struct SpawnRequest<'a> {
    pub(crate) group: Option<&'a str>,
    pub(crate) client_cwd: Option<&'a std::path::Path>,
    pub(crate) session_name: Option<&'a str>,
    pub(crate) extra_args: &'a [String],
    pub(crate) intent: LaunchIntent,
}

/// Spawn and register a hosted harness in `root`'s directory. Returns the
/// transport metadata the caller needs to attach or report the endpoint.
pub(crate) async fn spawn_agent(
    state: &Arc<DaemonState>,
    slug: &str,
    root: &str,
    request: SpawnRequest<'_>,
) -> Result<HostedSpawn> {
    let (endpoint, pubkey) = spawn_agent_inner(
        state,
        slug,
        root,
        request.group,
        request.client_cwd,
        request.session_name,
        request.extra_args,
        false,
        request.intent,
    )
    .await?;
    Ok(HostedSpawn { endpoint, pubkey })
}

pub async fn spawn_ephemeral_agent(
    state: &Arc<DaemonState>,
    slug: &str,
    root: &str,
    group: Option<&str>,
    client_cwd: Option<&std::path::Path>,
) -> Result<crate::session_host::transport::EndpointRef> {
    Ok(spawn_agent_inner(
        state,
        slug,
        root,
        group,
        client_cwd,
        None,
        &[],
        true,
        LaunchIntent::Managed,
    )
    .await?
    .0
    .endpoint_ref())
}

pub(crate) async fn spawn_ephemeral_agent_for_pubkey(
    state: &Arc<DaemonState>,
    slug: &str,
    root: &str,
    group: Option<&str>,
    client_cwd: Option<&std::path::Path>,
    expected_pubkey: &str,
) -> Result<crate::session_host::transport::EndpointRef> {
    Ok(spawn_agent_inner_full(
        state,
        slug,
        root,
        group,
        None,
        None,
        client_cwd,
        None,
        &[],
        true,
        LaunchIntent::Managed,
        Some(expected_pubkey),
    )
    .await?
    .0
    .endpoint_ref())
}

pub async fn spawn_dispatched_ephemeral_agent(
    state: &Arc<DaemonState>,
    slug: &str,
    root: &str,
    channels: &[String],
    dispatch_event: &str,
) -> Result<DispatchedSpawn> {
    let group = channels
        .first()
        .context("dispatch spawn requires at least one channel")?;
    let (endpoint, pubkey) = spawn_agent_inner_full(
        state,
        slug,
        root,
        Some(group),
        Some(channels),
        Some(dispatch_event),
        None,
        None,
        &[],
        true,
        LaunchIntent::Managed,
        None,
    )
    .await?;
    Ok(DispatchedSpawn {
        endpoint: endpoint.endpoint_ref(),
        pubkey,
    })
}

#[allow(clippy::too_many_arguments)]
async fn spawn_agent_inner(
    state: &Arc<DaemonState>,
    slug: &str,
    root: &str,
    group: Option<&str>,
    client_cwd: Option<&std::path::Path>,
    session_name: Option<&str>,
    extra_args: &[String],
    ephemeral: bool,
    intent: LaunchIntent,
) -> Result<(crate::session_host::transport::SessionEndpoint, String)> {
    spawn_agent_inner_full(
        state,
        slug,
        root,
        group,
        None,
        None,
        client_cwd,
        session_name,
        extra_args,
        ephemeral,
        intent,
        None,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
async fn spawn_agent_inner_full(
    state: &Arc<DaemonState>,
    slug: &str,
    root: &str,
    group: Option<&str>,
    joined_channels: Option<&[String]>,
    dispatch_event: Option<&str>,
    client_cwd: Option<&std::path::Path>,
    session_name: Option<&str>,
    extra_args: &[String],
    ephemeral: bool,
    intent: LaunchIntent,
    expected_pubkey: Option<&str>,
) -> Result<(crate::session_host::transport::SessionEndpoint, String)> {
    let abs_path = workspace_abs_path(state, root, client_cwd)?;
    let mut resolved = resolve_agent_source(state, slug, std::path::Path::new(&abs_path), intent)?;
    append_extra_args(&mut resolved, extra_args);
    let agent_slug = resolved.identity.slug.clone();
    let agent_command = resolved.command.clone();
    let harness = resolved.harness;
    let reservation = match expected_pubkey {
        Some(pubkey) => admission::reserve_fresh_for_pubkey(
            state,
            &resolved.identity,
            harness.as_str(),
            &resolved.bundle,
            resolved.transport.kind().as_str(),
            root,
            group,
            pubkey,
        )?,
        None => admission::reserve_fresh(
            state,
            &resolved.identity,
            harness.as_str(),
            &resolved.bundle,
            resolved.transport.kind().as_str(),
            root,
            group,
            session_name,
        )?,
    };
    let endpoint = match open_agent_session(
        &resolved.transport,
        &agent_slug,
        root,
        &abs_path,
        &agent_command,
        group,
        session_name,
        ephemeral,
        &reservation.pubkey,
        &reservation.agent_nsec,
        resolved.native_agent.as_ref(),
        resolved.prepared_launch,
    )
    .await
    {
        Ok(meta) => meta,
        Err(error) => {
            admission::release(state, &reservation);
            return Err(error);
        }
    };
    let channels = joined_channels.unwrap_or(&[]);
    let pubkey = match crate::daemon::server::session_start::bootstrap_hosted_session_start(
        state,
        &endpoint,
        crate::daemon::server::session_start::bootstrap::HostedSessionStart {
            pubkey: &reservation.pubkey,
            reclaimed_pubkey: reservation.reclaimed_pubkey.as_deref(),
            channel: group,
            channels,
            resume_id: None,
            dispatch_event,
            session_name,
            observed_harness: harness,
            admitted_bundle: &resolved.bundle,
            admitted_transport: resolved.transport.kind(),
        },
    )
    .await
    {
        Ok(pubkey) => pubkey,
        Err(e) => {
            kill_endpoint(&resolved.transport, &endpoint.endpoint_id).await;
            admission::release(state, &reservation);
            return Err(e.context("registering hosted session"));
        }
    };
    state.schedule_backend_profile_refresh();
    Ok((endpoint, pubkey))
}

fn append_extra_args(resolved: &mut ResolvedSource, extra_args: &[String]) {
    let rpc_argv = resolved
        .prepared_launch
        .rpc
        .as_mut()
        .map(|rpc| &mut rpc.argv);
    append_command_args(&mut resolved.command, rpc_argv, extra_args);
}

fn append_command_args(
    command: &mut Vec<String>,
    rpc_argv: Option<&mut Vec<String>>,
    extra_args: &[String],
) {
    command.extend_from_slice(extra_args);
    if let Some(argv) = rpc_argv {
        argv.extend_from_slice(extra_args);
    }
}

#[cfg(test)]
#[path = "spawn/tests.rs"]
mod tests;
