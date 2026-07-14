use super::*;

pub struct DispatchedSpawn {
    pub pty_id: String,
    pub session_id: String,
}

pub(crate) enum SpawnSource {
    Configured,
    PtyCommand(Vec<String>),
    PtyBundle(String),
}

pub(crate) struct SpawnRequest<'a> {
    pub(crate) source: SpawnSource,
    pub(crate) launch_args: Vec<String>,
    pub(crate) group: Option<&'a str>,
    pub(crate) client_cwd: Option<&'a std::path::Path>,
    pub(crate) session_name: Option<&'a str>,
}

/// Spawn and register a hosted harness in `root`'s directory. Returns the
/// transport metadata the caller needs to attach or report the endpoint.
pub(crate) async fn spawn_agent(
    state: &Arc<DaemonState>,
    slug: &str,
    root: &str,
    request: SpawnRequest<'_>,
) -> Result<crate::pty::LaunchMetadata> {
    spawn_agent_inner(
        state,
        slug,
        root,
        request.source,
        request.launch_args,
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
    group: Option<&str>,
    client_cwd: Option<&std::path::Path>,
) -> Result<String> {
    Ok(spawn_agent_inner(
        state,
        slug,
        root,
        SpawnSource::Configured,
        launch_args,
        group,
        client_cwd,
        None,
        true,
    )
    .await?
    .id)
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
    let (meta, session_id) = spawn_agent_inner_full(
        state,
        slug,
        root,
        SpawnSource::Configured,
        Vec::new(),
        Some(group),
        Some(channels),
        Some(dispatch_event),
        None,
        None,
        true,
    )
    .await?;
    Ok(DispatchedSpawn {
        pty_id: meta.id,
        session_id,
    })
}

#[allow(clippy::too_many_arguments)]
async fn spawn_agent_inner(
    state: &Arc<DaemonState>,
    slug: &str,
    root: &str,
    source: SpawnSource,
    launch_args: Vec<String>,
    group: Option<&str>,
    client_cwd: Option<&std::path::Path>,
    session_name: Option<&str>,
    ephemeral: bool,
) -> Result<crate::pty::LaunchMetadata> {
    Ok(spawn_agent_inner_full(
        state,
        slug,
        root,
        source,
        launch_args,
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
    source: SpawnSource,
    launch_args: Vec<String>,
    group: Option<&str>,
    joined_channels: Option<&[String]>,
    dispatch_event: Option<&str>,
    client_cwd: Option<&std::path::Path>,
    session_name: Option<&str>,
    ephemeral: bool,
) -> Result<(crate::pty::LaunchMetadata, String)> {
    let resolved = resolve_source(slug, source)?;
    let mut agent_command = apply_agent_def_args(resolved.command, slug, resolved.agent_def);
    if !launch_args.is_empty() {
        agent_command.extend(launch_args);
    }
    let _ = find_spawn_def(slug);

    let abs_path = workspace_abs_path(state, root, client_cwd)?;
    let reservation = admission::reserve(state, slug)?;
    let meta = match open_agent_session(
        &resolved.transport,
        slug,
        root,
        &abs_path,
        &agent_command,
        group,
        session_name,
        ephemeral,
        reservation.clone(),
        resolved.pty_launch,
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
            kill_endpoint(&resolved.transport, &pty_id).await;
            return Err(e.context("registering hosted session"));
        }
    };
    Ok((meta, session_id))
}

struct ResolvedSource {
    transport: crate::session_host::transport::TransportImpl,
    command: Vec<String>,
    agent_def: Option<serde_json::Value>,
    pty_launch: Option<PtyLaunchSpec>,
}

fn resolve_source(slug: &str, source: SpawnSource) -> Result<ResolvedSource> {
    match source {
        SpawnSource::Configured => {
            let transport = transport_for_slug(slug)?;
            let (command, agent_def) = resolve_spawn_command(slug, &transport)?;
            Ok(ResolvedSource {
                transport,
                command,
                agent_def,
                pty_launch: None,
            })
        }
        SpawnSource::PtyCommand(command) => Ok(ResolvedSource {
            transport: crate::session_host::transport::select_transport(None)?,
            command,
            agent_def: None,
            pty_launch: None,
        }),
        SpawnSource::PtyBundle(bundle) => {
            let id = crate::pty::new_session_id(slug);
            let scratch = crate::config::edge_home()
                .join("harness-profiles")
                .join(&id);
            let resolved = crate::harness::resolve(&bundle, &scratch)
                .with_context(|| format!("resolving PTY harness bundle {bundle:?}"))?;
            if resolved.transport != crate::harness::Transport::Pty {
                anyhow::bail!(
                    "harness bundle {bundle:?} uses the {} transport, which cannot attach to a terminal",
                    resolved.transport.as_str()
                );
            }
            resolved.profile.materialize()?;
            let mut env = resolved.profile.extra_env;
            let mut env_remove = Vec::new();
            for directive in resolved.driver.base_env {
                match directive {
                    crate::harness::EnvDirective::Set(key, value) => {
                        env.push((key.to_string(), value.to_string()));
                    }
                    crate::harness::EnvDirective::Remove(key) => {
                        env_remove.push(key.to_string());
                    }
                }
            }
            Ok(ResolvedSource {
                transport: crate::session_host::transport::select_transport(None)?,
                command: resolved.base_argv,
                agent_def: None,
                pty_launch: Some(PtyLaunchSpec {
                    id: Some(id),
                    env,
                    env_remove,
                }),
            })
        }
    }
}
