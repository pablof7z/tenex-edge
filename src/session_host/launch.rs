use super::admission;
use crate::daemon::server::DaemonState;
use crate::session_host::registry::{
    apply_agent_def_args, build_resume_command, find_spawn_def, resolve_spawn_entry,
    resume_shape_for_bin,
};
use crate::session_host::transport::{select_transport, LaunchSpec, ResumeSpec, TransportImpl};
use anyhow::{Context, Result};
use std::sync::Arc;

mod spawn;
pub(crate) use spawn::{spawn_agent, SpawnRequest, SpawnSource};
pub use spawn::{spawn_dispatched_ephemeral_agent, spawn_ephemeral_agent, DispatchedSpawn};

/// Resolve which transport hosts `slug`, from its configured harness bundle. An
/// agent with no bundle (the overwhelming majority) resolves to the PTY, and its
/// launch path is byte-identical to before this wiring existed.
fn transport_for_slug(slug: &str) -> Result<TransportImpl> {
    let bundle = crate::identity::agent_harness_bundle(&crate::config::edge_home(), slug);
    select_transport(bundle.as_deref())
}

/// Kill a just-opened endpoint through its transport (PTY supervisor or ACP
/// child) — used to roll back a session whose registration failed.
async fn kill_endpoint(transport: &TransportImpl, endpoint_id: &str) {
    use crate::session_host::transport::EndpointRef;
    let ep = EndpointRef {
        kind: transport.kind(),
        endpoint_id: endpoint_id.to_string(),
    };
    let _ = transport.kill(&ep).await;
}

/// Resolve the base spawn command + inline agent definition for `slug`.
///
/// PTY agents must have a resolvable harness command (unchanged). An ACP/
/// app-server agent is launched from its harness bundle's driver argv, not a PTY
/// command, so when it has no `commands` entry we synthesize a nominal command
/// (the bundle's harness slug) purely so harness inference + recorded session
/// metadata are correct; the actual child argv comes from the bundle driver.
fn resolve_spawn_command(
    slug: &str,
    transport: &TransportImpl,
) -> Result<(Vec<String>, Option<serde_json::Value>)> {
    match resolve_spawn_entry(slug) {
        Ok(v) => Ok(v),
        Err(e) => {
            if matches!(transport, TransportImpl::Acp(_)) {
                let bundle =
                    crate::identity::agent_harness_bundle(&crate::config::edge_home(), slug);
                let cfg = crate::harness::config::HarnessesConfig::load()?;
                let harness =
                    crate::harness::bundle_harness_with(&cfg, bundle.as_deref().unwrap_or(slug))?;
                Ok((vec![harness.as_str().to_string()], None))
            } else {
                Err(e)
            }
        }
    }
}

pub(super) fn workspace_abs_path(
    state: &Arc<DaemonState>,
    channel: &str,
    client_cwd: Option<&std::path::Path>,
) -> Result<String> {
    if let Some(cwd) = client_cwd {
        let abs = cwd.to_string_lossy().to_string();
        let now = crate::util::now_secs();
        // The recorded workspace path is what the resume path reads back; if the
        // write is dropped, a later resume falls into the "no workspace" branch and
        // we'd spawn in the wrong directory. Propagate the failure, don't swallow.
        state
            .with_store(|s| s.upsert_workspace(channel, &abs, now))
            .with_context(|| format!("recording workspace path for {channel:?}"))?;
        return Ok(abs);
    }
    // Resume path (no client cwd): the workspace path MUST already be recorded.
    // Never guess the daemon's current_dir here; an unrelated daemon cwd would
    // land the agent in the wrong directory. Fail loud on a read error or
    // missing row.
    let abs = state
        .with_store(|s| s.workspace_path(channel))
        .with_context(|| format!("looking up workspace path for {channel:?}"))?;
    abs.ok_or_else(|| {
        anyhow::anyhow!("cannot resolve workspace path for {channel:?} (no recorded path)")
    })
}

#[allow(clippy::too_many_arguments)]
async fn open_agent_session(
    transport: &TransportImpl,
    slug: &str,
    root: &str,
    abs_path: &str,
    command: &[String],
    group: Option<&str>,
    session_name: Option<&str>,
    ephemeral: bool,
    pubkey: &str,
    pty_launch: Option<PtyLaunchSpec>,
) -> Result<crate::pty::LaunchMetadata> {
    match transport {
        TransportImpl::Pty(_) => {
            let pty_launch = pty_launch.unwrap_or_default();
            let meta = crate::pty::spawn_session(crate::pty::SpawnSessionArgs {
                id: pty_launch.id,
                agent: slug.to_string(),
                root: root.to_string(),
                cwd: std::path::PathBuf::from(abs_path),
                channel: group.filter(|g| !g.is_empty()).map(str::to_string),
                session_name: session_name.map(str::to_string),
                ephemeral,
                command: command.to_vec(),
                env: pty_launch
                    .env
                    .into_iter()
                    .chain([(String::from("TENEX_EDGE_PUBKEY"), pubkey.to_string())])
                    .collect(),
                env_remove: pty_launch.env_remove,
            })?;
            Ok(meta)
        }
        TransportImpl::Acp(t) => {
            use crate::session_host::transport::SessionTransport;
            let spec = LaunchSpec {
                slug: slug.to_string(),
                // The bundle NAME (harnesses.json key) is distinct from the agent
                // slug; the ACP transport resolves its harness/driver from this,
                // never from the slug (defect #1).
                bundle: crate::identity::agent_harness_bundle(&crate::config::edge_home(), slug),
                root: root.to_string(),
                abs_path: abs_path.to_string(),
                group: group.map(str::to_string),
                ephemeral,
                base_command: command.to_vec(),
                pubkey: pubkey.to_string(),
            };
            let endpoint = t.launch(&spec).await?;
            Ok(endpoint.meta)
        }
    }
}

/// Resume a prior session by replaying its harness with the native resume token.
pub async fn resume_agent(
    state: &Arc<DaemonState>,
    slug: &str,
    root: &str,
    resume_id: &str,
) -> Result<String> {
    resume_agent_in_channel(state, slug, root, root, resume_id).await
}

/// Resume a prior session into an explicit channel while using `root` to
/// resolve the working directory.
pub async fn resume_agent_in_channel(
    state: &Arc<DaemonState>,
    slug: &str,
    root: &str,
    group: &str,
    resume_id: &str,
) -> Result<String> {
    if resume_id.is_empty() {
        anyhow::bail!("session has no resume token (not resumable)");
    }

    let transport = transport_for_slug(slug)?;
    let abs_path = workspace_abs_path(state, root, None)?;
    let (base, _agent_def) = resolve_spawn_command(slug, &transport)?;
    let harness = crate::daemon::server::session_start::bootstrap::infer_harness(&base);
    let reservation =
        admission::reserve_resume(state, slug, harness.as_str(), root, group, resume_id)?;
    let meta = match &transport {
        TransportImpl::Pty(_) => {
            let bin = base.first().map(String::as_str).unwrap_or("");
            let shape = resume_shape_for_bin(bin).with_context(|| {
                format!("don't know how to resume harness binary {bin:?} (agent {slug:?})")
            })?;
            let resume_command = build_resume_command(&base, shape, resume_id);
            open_agent_session(
                &transport,
                slug,
                root,
                &abs_path,
                &resume_command,
                Some(group),
                None,
                false,
                &reservation.pubkey,
                None,
            )
            .await?
        }
        // ACP/app-server: re-enter the native session by its resume token
        // (`session/load` or `thread/resume`); the driver argv comes from the
        // harness bundle, so no PTY resume-command shaping applies.
        TransportImpl::Acp(t) => {
            use crate::session_host::transport::SessionTransport;
            let spec = LaunchSpec {
                slug: slug.to_string(),
                bundle: crate::identity::agent_harness_bundle(&crate::config::edge_home(), slug),
                root: root.to_string(),
                abs_path: abs_path.clone(),
                group: Some(group.to_string()),
                ephemeral: false,
                base_command: base,
                pubkey: reservation.pubkey.clone(),
            };
            let resume = ResumeSpec {
                native_id: resume_id.to_string(),
            };
            t.resume(&spec, &resume).await?.meta
        }
    };
    let pty_id = meta.id.clone();
    if let Err(e) = crate::daemon::server::session_start::bootstrap_pty_session_start(
        state,
        &meta,
        crate::daemon::server::session_start::bootstrap::PtySessionStart {
            pubkey: &reservation.pubkey,
            reclaimed_pubkey: None,
            channel: Some(group),
            channels: &[],
            resume_id: Some(resume_id),
            dispatch_event: None,
            session_name: None,
        },
    )
    .await
    {
        kill_endpoint(&transport, &pty_id).await;
        admission::release(state, &reservation);
        return Err(e.context("registering resumed hosted session"));
    }
    Ok(pty_id)
}

#[derive(Default)]
struct PtyLaunchSpec {
    id: Option<String>,
    env: Vec<(String, String)>,
    env_remove: Vec<String>,
}
