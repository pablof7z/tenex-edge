use super::*;

pub struct DispatchedSpawn {
    pub pty_id: String,
    pub session_id: String,
}

pub(crate) struct SpawnRequest<'a> {
    pub(crate) launch_args: Vec<String>,
    pub(crate) base_override: Option<Vec<String>>,
    pub(crate) group: Option<&'a str>,
    pub(crate) client_cwd: Option<&'a std::path::Path>,
    pub(crate) session_name: Option<&'a str>,
}

/// Spawn a new hosted harness in `root`'s directory. Returns the endpoint id.
pub(crate) async fn spawn_agent(
    state: &Arc<DaemonState>,
    slug: &str,
    root: &str,
    request: SpawnRequest<'_>,
) -> Result<String> {
    spawn_agent_inner(
        state,
        slug,
        root,
        request.launch_args,
        request.base_override,
        request.group,
        request.client_cwd,
        request.session_name,
        false,
    )
    .await
}

pub async fn spawn_ephemeral_agent(
    state: &Arc<DaemonState>,
    slug: &str,
    root: &str,
    launch_args: Vec<String>,
    base_override: Option<Vec<String>>,
    group: Option<&str>,
    client_cwd: Option<&std::path::Path>,
) -> Result<String> {
    spawn_agent_inner(
        state,
        slug,
        root,
        launch_args,
        base_override,
        group,
        client_cwd,
        None,
        true,
    )
    .await
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
    let (pty_id, session_id) = spawn_agent_inner_full(
        state,
        slug,
        root,
        Vec::new(),
        None,
        Some(group),
        Some(channels),
        Some(dispatch_event),
        None,
        None,
        true,
    )
    .await?;
    Ok(DispatchedSpawn { pty_id, session_id })
}

#[allow(clippy::too_many_arguments)]
async fn spawn_agent_inner(
    state: &Arc<DaemonState>,
    slug: &str,
    root: &str,
    launch_args: Vec<String>,
    base_override: Option<Vec<String>>,
    group: Option<&str>,
    client_cwd: Option<&std::path::Path>,
    session_name: Option<&str>,
    ephemeral: bool,
) -> Result<String> {
    Ok(spawn_agent_inner_full(
        state,
        slug,
        root,
        launch_args,
        base_override,
        group,
        None,
        None,
        client_cwd,
        session_name,
        ephemeral,
    )
    .await?
    .0)
}

#[allow(clippy::too_many_arguments)]
async fn spawn_agent_inner_full(
    state: &Arc<DaemonState>,
    slug: &str,
    root: &str,
    launch_args: Vec<String>,
    base_override: Option<Vec<String>>,
    group: Option<&str>,
    joined_channels: Option<&[String]>,
    dispatch_event: Option<&str>,
    client_cwd: Option<&std::path::Path>,
    session_name: Option<&str>,
    ephemeral: bool,
) -> Result<(String, String)> {
    let transport = transport_for_slug(slug)?;
    let (base_command, agent_def) = match base_override {
        Some(cmd) => (cmd, None),
        None => resolve_spawn_command(slug, &transport)?,
    };
    let mut agent_command = apply_agent_def_args(base_command, slug, agent_def);
    if !launch_args.is_empty() {
        agent_command.extend(launch_args);
    }
    let _ = find_spawn_def(slug);

    let abs_path = workspace_abs_path(state, root, client_cwd)?;
    let reservation = admission::reserve(state, slug)?;
    let meta = match open_agent_session(
        &transport,
        slug,
        root,
        &abs_path,
        &agent_command,
        group,
        session_name,
        ephemeral,
        reservation.clone(),
    )
    .await
    {
        Ok(meta) => meta,
        Err(error) => {
            admission::release(state, reservation.as_deref());
            return Err(error);
        }
    };
    let pty_id = meta.id.clone();
    let channels = joined_channels.unwrap_or(&[]);
    let session_id = match crate::daemon::server::session_start::bootstrap_pty_session_start(
        state,
        &meta,
        crate::daemon::server::session_start::bootstrap::PtySessionStart {
            channel: group,
            channels,
            resume_id: None,
            dispatch_event,
            session_name,
            durable_reservation: reservation.as_deref(),
        },
    )
    .await
    {
        Ok(session_id) => session_id,
        Err(e) => {
            kill_endpoint(&transport, &pty_id).await;
            return Err(e.context("registering hosted session"));
        }
    };
    Ok((pty_id, session_id))
}
