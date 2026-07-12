//! `AcpTransport`: stdio JSON-RPC backend over `crate::rpc_harness`. Does NOT
//! use `src/pty/*`. ACP has no unix socket, so liveness = child alive; the
//! per-session child lives in a process-global registry keyed by our endpoint
//! id.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use anyhow::{Context, Result};

use super::{
    EndpointRef, LaunchSpec, ResumeSpec, SessionEndpoint, SessionTransport, TransportKind,
};
use crate::harness::{self, Transport};
use crate::rpc_harness::{
    spawn_config_from_driver, AcpClient, AppServerClient, Callbacks, Dialect, RpcHandle,
};

/// A live ACP/app-server child plus its native session token.
struct AcpChild {
    handle: RpcHandle,
    /// ACP `sessionId` or app-server `threadId`.
    native_id: String,
    cwd: std::path::PathBuf,
}

fn registry() -> &'static Mutex<HashMap<String, AcpChild>> {
    static REG: OnceLock<Mutex<HashMap<String, AcpChild>>> = OnceLock::new();
    REG.get_or_init(|| Mutex::new(HashMap::new()))
}

pub struct AcpTransport;

/// The outcome of opening (or resuming) an RPC-hosted session, before it is
/// wrapped into a `SessionEndpoint`.
pub struct AcpOpen {
    pub endpoint_id: String,
    pub native_id: String,
    pub pid: Option<u32>,
}

impl AcpTransport {
    /// Resolve the harness bundle for `slug` and spawn its RPC child, returning
    /// the live handle + dialect. Shared by launch/resume.
    async fn spawn_child(
        spec: &LaunchSpec,
    ) -> Result<(RpcHandle, Dialect, harness::ResolvedHarness)> {
        let scratch = crate::config::edge_home()
            .join("harness-profiles")
            .join(&spec.slug);
        let resolved = harness::resolve(&spec.slug, &scratch)
            .with_context(|| format!("resolving harness bundle {:?}", spec.slug))?;
        if !matches!(resolved.transport, Transport::Acp | Transport::AppServer) {
            anyhow::bail!(
                "harness bundle {:?} is transport {} — not an RPC transport",
                spec.slug,
                resolved.transport.as_str()
            );
        }
        // Materialize any profile settings files before launch.
        for (path, contents) in &resolved.profile.files {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("creating profile dir {}", parent.display()))?;
            }
            std::fs::write(path, contents)
                .with_context(|| format!("writing profile file {}", path.display()))?;
        }
        let cwd = std::path::PathBuf::from(&spec.abs_path);
        let callbacks = Callbacks::allow_all(cwd.clone());
        let cfg = spawn_config_from_driver(
            resolved.driver,
            &resolved.base_argv,
            &resolved.profile.extra_env,
            cwd,
            callbacks,
        )?;
        let dialect = cfg.dialect;
        let (handle, _updates) = RpcHandle::spawn(cfg)
            .await
            .map_err(|e| anyhow::anyhow!("spawning RPC harness for {:?}: {e}", spec.slug))?;
        Ok((handle, dialect, resolved))
    }

    fn endpoint_id(slug: &str) -> String {
        let now = crate::util::now_secs();
        format!("te-acp-{slug}-{now}-{}", std::process::id())
    }

    fn synth_meta(spec: &LaunchSpec, endpoint_id: &str) -> crate::pty::LaunchMetadata {
        crate::pty::LaunchMetadata {
            id: endpoint_id.to_string(),
            socket: String::new(),
            supervisor_pid: 0,
            agent: spec.slug.clone(),
            root: spec.root.clone(),
            cwd: spec.abs_path.clone(),
            ephemeral: spec.ephemeral,
            command: spec.base_command.clone(),
        }
    }

    /// Open a fresh session and register it; returns the open descriptor.
    pub async fn open(&self, spec: &LaunchSpec) -> Result<AcpOpen> {
        let (handle, dialect, _resolved) = Self::spawn_child(spec).await?;
        let cwd = std::path::PathBuf::from(&spec.abs_path);
        let native_id = match dialect {
            Dialect::Acp => {
                let client = AcpClient::new(handle.clone());
                client
                    .initialize()
                    .await
                    .map_err(|e| anyhow::anyhow!("ACP initialize: {e}"))?;
                client
                    .session_new(&cwd)
                    .await
                    .map_err(|e| anyhow::anyhow!("ACP session/new: {e}"))?
            }
            Dialect::AppServer => {
                let client = AppServerClient::new(handle.clone());
                client
                    .initialize("tenex-edge", env!("CARGO_PKG_VERSION"))
                    .await
                    .map_err(|e| anyhow::anyhow!("app-server initialize: {e}"))?;
                client
                    .thread_start(&cwd)
                    .await
                    .map_err(|e| anyhow::anyhow!("app-server thread/start: {e}"))?
            }
        };
        let endpoint_id = Self::endpoint_id(&spec.slug);
        let pid = handle.pid;
        registry().lock().unwrap().insert(
            endpoint_id.clone(),
            AcpChild {
                handle,
                native_id: native_id.clone(),
                cwd,
            },
        );
        Ok(AcpOpen {
            endpoint_id,
            native_id,
            pid,
        })
    }
}

impl SessionTransport for AcpTransport {
    fn kind(&self) -> TransportKind {
        TransportKind::Acp
    }

    async fn launch(&self, spec: &LaunchSpec) -> Result<SessionEndpoint> {
        let open = self.open(spec).await?;
        Ok(SessionEndpoint {
            kind: TransportKind::Acp,
            endpoint_id: open.endpoint_id.clone(),
            watch_pid: open.pid.and_then(|p| i32::try_from(p).ok()),
            meta: Self::synth_meta(spec, &open.endpoint_id),
        })
    }

    async fn resume(&self, spec: &LaunchSpec, resume: &ResumeSpec) -> Result<SessionEndpoint> {
        if resume.native_id.is_empty() {
            anyhow::bail!("session has no resume token (not resumable)");
        }
        let (handle, dialect, _resolved) = Self::spawn_child(spec).await?;
        let cwd = std::path::PathBuf::from(&spec.abs_path);
        match dialect {
            Dialect::Acp => {
                let client = AcpClient::new(handle.clone());
                client
                    .initialize()
                    .await
                    .map_err(|e| anyhow::anyhow!("ACP initialize (resume): {e}"))?;
                client
                    .session_load(&resume.native_id, &cwd)
                    .await
                    .map_err(|e| anyhow::anyhow!("ACP session/load: {e}"))?;
            }
            Dialect::AppServer => {
                let client = AppServerClient::new(handle.clone());
                client
                    .initialize("tenex-edge", env!("CARGO_PKG_VERSION"))
                    .await
                    .map_err(|e| anyhow::anyhow!("app-server initialize (resume): {e}"))?;
                client
                    .thread_resume(&resume.native_id, &cwd)
                    .await
                    .map_err(|e| anyhow::anyhow!("app-server thread/resume: {e}"))?;
            }
        }
        let endpoint_id = Self::endpoint_id(&spec.slug);
        let pid = handle.pid;
        registry().lock().unwrap().insert(
            endpoint_id.clone(),
            AcpChild {
                handle,
                native_id: resume.native_id.clone(),
                cwd,
            },
        );
        Ok(SessionEndpoint {
            kind: TransportKind::Acp,
            endpoint_id: endpoint_id.clone(),
            watch_pid: pid.and_then(|p| i32::try_from(p).ok()),
            meta: Self::synth_meta(spec, &endpoint_id),
        })
    }

    async fn deliver(&self, ep: &EndpointRef, text: &str, submit: bool) -> Result<()> {
        // Snapshot the pieces we need without holding the lock across await.
        let (handle, native_id, dialect) = {
            let reg = registry().lock().unwrap();
            let child = reg
                .get(&ep.endpoint_id)
                .with_context(|| format!("no ACP session registered for {:?}", ep.endpoint_id))?;
            (
                child.handle.clone(),
                child.native_id.clone(),
                child.handle.dialect,
            )
        };
        match (dialect, submit) {
            (Dialect::Acp, _) => {
                // ACP has no steer RPC; both submit and non-submit map to a
                // fresh prompt (between-turns delivery).
                let client = AcpClient::new(handle);
                client
                    .session_prompt(&native_id, text)
                    .await
                    .map_err(|e| anyhow::anyhow!("ACP session/prompt: {e}"))?;
                Ok(())
            }
            (Dialect::AppServer, true) => {
                let client = AppServerClient::new(handle);
                client
                    .turn_start(&native_id, text)
                    .await
                    .map_err(|e| anyhow::anyhow!("app-server turn/start: {e}"))?;
                Ok(())
            }
            (Dialect::AppServer, false) => {
                // Mid-turn steer needs the running turn id, which we do not track
                // here; fall back to starting a fresh turn.
                let client = AppServerClient::new(handle);
                client
                    .turn_start(&native_id, text)
                    .await
                    .map_err(|e| anyhow::anyhow!("app-server turn/start (steer fallback): {e}"))?;
                Ok(())
            }
        }
    }

    fn is_live(&self, ep: &EndpointRef) -> bool {
        registry()
            .lock()
            .unwrap()
            .get(&ep.endpoint_id)
            .map(|c| c.handle.is_alive())
            .unwrap_or(false)
    }

    async fn kill(&self, ep: &EndpointRef) -> Result<()> {
        let child = registry().lock().unwrap().remove(&ep.endpoint_id);
        if let Some(child) = child {
            let _ = child.cwd; // retained for parity/debugging
            if child.handle.dialect == Dialect::Acp {
                AcpClient::new(child.handle.clone())
                    .session_cancel(&child.native_id)
                    .await;
            }
            child.handle.kill().await;
        }
        Ok(())
    }
}
