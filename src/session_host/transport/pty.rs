//! `PtyTransport`: thin forward to the existing `src/pty/*` path. No logic
//! moves here — every method mirrors the current `open_agent_session` /
//! `resume_agent_in_channel` bodies so PTY sessions spawn/resume/inject/kill
//! byte-identically.

use anyhow::{Context, Result};

use super::{
    EndpointRef, LaunchSpec, ResumeSpec, SessionEndpoint, SessionTransport, TransportKind,
};
use crate::session_host::registry::{build_resume_command, resume_shape_for_bin};

pub struct PtyTransport;

impl PtyTransport {
    fn spawn(spec: &LaunchSpec, command: &[String]) -> Result<SessionEndpoint> {
        let meta = crate::pty::spawn_session(crate::pty::SpawnSessionArgs {
            id: None,
            agent: spec.slug.clone(),
            root: spec.root.clone(),
            cwd: std::path::PathBuf::from(&spec.abs_path),
            channel: spec.group.clone().filter(|g| !g.is_empty()),
            ephemeral: spec.ephemeral,
            durable_reservation: None,
            command: command.to_vec(),
        })?;
        Ok(SessionEndpoint {
            kind: TransportKind::Pty,
            endpoint_id: meta.id.clone(),
            watch_pid: i32::try_from(meta.supervisor_pid).ok(),
            meta,
        })
    }
}

impl SessionTransport for PtyTransport {
    fn kind(&self) -> TransportKind {
        TransportKind::Pty
    }

    async fn launch(&self, spec: &LaunchSpec) -> Result<SessionEndpoint> {
        Self::spawn(spec, &spec.base_command)
    }

    async fn resume(&self, spec: &LaunchSpec, resume: &ResumeSpec) -> Result<SessionEndpoint> {
        if resume.native_id.is_empty() {
            anyhow::bail!("session has no resume token (not resumable)");
        }
        let bin = spec.base_command.first().map(String::as_str).unwrap_or("");
        let shape = resume_shape_for_bin(bin).with_context(|| {
            format!(
                "don't know how to resume harness binary {bin:?} (agent {:?})",
                spec.slug
            )
        })?;
        let command = build_resume_command(&spec.base_command, shape, &resume.native_id);
        Self::spawn(spec, &command)
    }

    async fn deliver(&self, ep: &EndpointRef, text: &str, submit: bool) -> Result<()> {
        crate::pty::inject(&ep.endpoint_id, text, true, submit)
    }

    fn is_live(&self, ep: &EndpointRef) -> bool {
        crate::pty::is_live(&ep.endpoint_id)
    }

    async fn kill(&self, ep: &EndpointRef) -> Result<()> {
        crate::pty::kill(&ep.endpoint_id)
    }
}
