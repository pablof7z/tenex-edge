//! Transport-agnostic session hosting seam.
//!
//! A "hosted session" is opened over exactly one transport. Today that is
//! always the portable PTY (`PtyTransport`, byte-identical to the current
//! `open_agent_session` path). `AcpTransport` adds a stdio JSON-RPC backend
//! (ACP / codex app-server) for harnesses that expose an `RpcTurn` model.
//!
//! Object dispatch is via the [`TransportImpl`] enum rather than `dyn` (native
//! async-fn-in-trait is not object-safe and we avoid pulling `async-trait`).
//! The [`SessionTransport`] trait documents the contract each backend fulfils.

pub mod acp;
pub mod pty;

use anyhow::Result;

/// Which transport hosts a session. Stringifies to `"pty"` / `"acp"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportKind {
    Pty,
    Acp,
}

impl TransportKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransportKind::Pty => "pty",
            TransportKind::Acp => "acp",
        }
    }
}

/// Fully-resolved, transport-agnostic launch intent.
#[derive(Debug, Clone)]
pub struct LaunchSpec {
    pub slug: String,
    pub root: String,
    pub abs_path: String,
    pub group: Option<String>,
    pub ephemeral: bool,
    /// Resolved argv incl. base_argv + profile + user flags + agent-def args.
    pub base_command: Vec<String>,
}

/// A prior session's native resume token.
#[derive(Debug, Clone)]
pub struct ResumeSpec {
    pub native_id: String,
}

/// What the daemon needs after a session opens. For PTY this carries a real
/// `LaunchMetadata` so `bootstrap_pty_session_start` stays unchanged.
#[derive(Debug)]
pub struct SessionEndpoint {
    pub kind: TransportKind,
    pub endpoint_id: String,
    pub watch_pid: Option<i32>,
    pub meta: crate::pty::LaunchMetadata,
}

/// A live-session address the daemon holds after registration.
#[derive(Debug, Clone)]
pub struct EndpointRef {
    pub kind: TransportKind,
    pub endpoint_id: String,
}

/// The transport contract. Each method maps to an existing daemon behavior;
/// the two backends fulfil them by fundamentally different mechanisms
/// (relaunch-with-flag + bracketed-paste vs. JSON-RPC).
#[allow(async_fn_in_trait)]
pub trait SessionTransport {
    fn kind(&self) -> TransportKind;

    /// Open a brand-new harness session.
    async fn launch(&self, spec: &LaunchSpec) -> Result<SessionEndpoint>;

    /// Reopen a prior session by its native token.
    async fn resume(&self, spec: &LaunchSpec, resume: &ResumeSpec) -> Result<SessionEndpoint>;

    /// Deliver text; `submit` completes the turn.
    async fn deliver(&self, ep: &EndpointRef, text: &str, submit: bool) -> Result<()>;

    fn is_live(&self, ep: &EndpointRef) -> bool;

    async fn kill(&self, ep: &EndpointRef) -> Result<()>;
}

pub use acp::AcpTransport;
pub use pty::PtyTransport;

/// Enum dispatcher giving dynamic transport selection without `dyn`.
pub enum TransportImpl {
    Pty(PtyTransport),
    Acp(AcpTransport),
}

impl TransportImpl {
    pub fn kind(&self) -> TransportKind {
        match self {
            TransportImpl::Pty(t) => t.kind(),
            TransportImpl::Acp(t) => t.kind(),
        }
    }

    pub async fn launch(&self, spec: &LaunchSpec) -> Result<SessionEndpoint> {
        match self {
            TransportImpl::Pty(t) => t.launch(spec).await,
            TransportImpl::Acp(t) => t.launch(spec).await,
        }
    }

    pub async fn resume(&self, spec: &LaunchSpec, resume: &ResumeSpec) -> Result<SessionEndpoint> {
        match self {
            TransportImpl::Pty(t) => t.resume(spec, resume).await,
            TransportImpl::Acp(t) => t.resume(spec, resume).await,
        }
    }

    pub async fn deliver(&self, ep: &EndpointRef, text: &str, submit: bool) -> Result<()> {
        match self {
            TransportImpl::Pty(t) => t.deliver(ep, text, submit).await,
            TransportImpl::Acp(t) => t.deliver(ep, text, submit).await,
        }
    }

    pub fn is_live(&self, ep: &EndpointRef) -> bool {
        match self {
            TransportImpl::Pty(t) => t.is_live(ep),
            TransportImpl::Acp(t) => t.is_live(ep),
        }
    }

    pub async fn kill(&self, ep: &EndpointRef) -> Result<()> {
        match self {
            TransportImpl::Pty(t) => t.kill(ep).await,
            TransportImpl::Acp(t) => t.kill(ep).await,
        }
    }
}

/// Phase-1 selector: PTY for every harness (behavior unchanged). The
/// per-agent `(harness, transport)` selection that would return `Acp` is the
/// deferred post-#380 rebase; until then callers that want ACP construct
/// `AcpTransport` explicitly (e.g. the `__acp-smoke` debug command).
pub fn select_transport(_slug: &str) -> TransportImpl {
    TransportImpl::Pty(PtyTransport)
}

#[cfg(test)]
#[path = "transport/tests.rs"]
mod transport_tests;
