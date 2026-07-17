//! Portable-PTY implementation of the complete hosted-session transport seam.

use anyhow::Result;

use super::{
    EndpointRef, LaunchSpec, ResumeSpec, SessionEndpoint, SessionTransport, TransportKind,
};

pub struct PtyTransport;

#[async_trait::async_trait]
impl SessionTransport for PtyTransport {
    fn kind(&self) -> TransportKind {
        TransportKind::Pty
    }

    async fn launch(&self, spec: &LaunchSpec) -> Result<SessionEndpoint> {
        let mut env = spec.pty.env.clone();
        let mut env_remove = spec.pty.env_remove.clone();
        crate::session_host::agent_env::assign(
            &mut env,
            &mut env_remove,
            &spec.pubkey,
            &spec.agent_nsec,
        );
        let meta = crate::pty::spawn_session(crate::pty::SpawnSessionArgs {
            id: spec.pty.id.clone(),
            agent: spec.slug.clone(),
            root: spec.root.clone(),
            cwd: std::path::PathBuf::from(&spec.abs_path),
            channel: spec.group.clone().filter(|group| !group.is_empty()),
            session_name: spec.session_name.clone(),
            ephemeral: spec.ephemeral,
            command: spec.base_command.clone(),
            env,
            env_remove,
        })?;
        Ok(SessionEndpoint {
            kind: TransportKind::Pty,
            endpoint_id: meta.id.clone(),
            watch_pid: i32::try_from(meta.supervisor_pid).ok(),
            meta,
        })
    }

    async fn resume(&self, spec: &LaunchSpec, _resume: &ResumeSpec) -> Result<SessionEndpoint> {
        self.launch(spec).await
    }

    async fn deliver(&self, ep: &EndpointRef, text: &str, submit: bool) -> Result<()> {
        if !self.is_live(ep) {
            anyhow::bail!("pty session {} is not live", ep.endpoint_id);
        }
        crate::pty::inject(&ep.endpoint_id, text, true, false)?;
        if submit {
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            crate::pty::inject(&ep.endpoint_id, "", false, true)?;
        }
        Ok(())
    }

    fn is_live(&self, ep: &EndpointRef) -> bool {
        crate::pty::is_live(&ep.endpoint_id)
    }

    fn opening_delivery_delay(&self) -> std::time::Duration {
        std::time::Duration::from_millis(2000)
    }

    async fn kill(&self, ep: &EndpointRef) -> Result<()> {
        if !self.is_live(ep) {
            return Ok(());
        }
        crate::pty::kill(&ep.endpoint_id)
    }
}
