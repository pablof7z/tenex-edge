//! Agent->client request handling: permission auto-allow + jailed fs bridge.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::protocol::RpcErrorObject;

/// Policy for `session/request_permission`.
#[derive(Clone)]
pub enum PermissionPolicy {
    /// Auto-allow every request: pick the first allow-shaped option, else the
    /// first option. This is the daemon's headless posture — no human is
    /// attached to answer a prompt (mirrors the PTY path launching with
    /// `--dangerously-skip-permissions`).
    AllowAll,
    /// Escape hatch for future policy.
    #[allow(clippy::type_complexity)]
    Custom(Arc<dyn Fn(&serde_json::Value) -> Option<String> + Send + Sync>),
}

impl std::fmt::Debug for PermissionPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PermissionPolicy::AllowAll => f.write_str("AllowAll"),
            PermissionPolicy::Custom(_) => f.write_str("Custom(..)"),
        }
    }
}

/// Filesystem bridge for `fs/read_text_file` / `fs/write_text_file`, jailed
/// under `root` (the session cwd).
#[derive(Clone, Debug)]
pub struct FsBridge {
    pub root: PathBuf,
}

/// The full callback bundle handed to the reader task.
#[derive(Clone, Debug)]
pub struct Callbacks {
    pub permission: PermissionPolicy,
    pub fs: FsBridge,
}

impl Callbacks {
    pub fn allow_all(root: PathBuf) -> Self {
        Self {
            permission: PermissionPolicy::AllowAll,
            fs: FsBridge { root },
        }
    }
}

impl PermissionPolicy {
    /// Choose an `optionId` for a `session/request_permission` params object.
    pub fn choose(&self, params: &serde_json::Value) -> Option<String> {
        match self {
            PermissionPolicy::Custom(f) => f(params),
            PermissionPolicy::AllowAll => {
                let options = params.get("options")?.as_array()?;
                // Prefer an allow-kind option, else the first.
                let allow = options.iter().find(|o| {
                    let kind = o
                        .get("kind")
                        .or_else(|| o.get("optionId"))
                        .and_then(|k| k.as_str())
                        .unwrap_or("");
                    kind.starts_with("allow")
                });
                let chosen = allow.or_else(|| options.first())?;
                chosen
                    .get("optionId")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            }
        }
    }
}

impl FsBridge {
    /// Resolve a requested path against/within `root`. Absolute paths are
    /// jailed under `root`; escapes are refused.
    fn resolve(&self, requested: &str) -> Result<PathBuf, RpcErrorObject> {
        let p = Path::new(requested);
        let joined = if p.is_absolute() {
            p.to_path_buf()
        } else {
            self.root.join(p)
        };
        // Canonicalize the parent (the file may not exist yet for writes) and
        // verify containment.
        let anchor = joined.parent().unwrap_or(&self.root);
        let canon_root = self
            .root
            .canonicalize()
            .unwrap_or_else(|_| self.root.clone());
        let canon_anchor = anchor
            .canonicalize()
            .unwrap_or_else(|_| anchor.to_path_buf());
        let escaped = |requested: &str| RpcErrorObject {
            code: -32001,
            message: format!("path {requested:?} escapes session root"),
            data: None,
        };
        if !canon_anchor.starts_with(&canon_root) {
            return Err(escaped(requested));
        }
        // If the target itself already exists, canonicalize the *full* path so a
        // symlink inside the root that points outside it (`root/link ->
        // /etc/passwd`) is refused rather than followed out of the jail. New
        // (not-yet-existing) write targets fall through to the parent check
        // above, which already refuses `../` traversal.
        if let Ok(canon_full) = joined.canonicalize() {
            if !canon_full.starts_with(&canon_root) {
                return Err(escaped(requested));
            }
        }
        Ok(joined)
    }

    pub async fn read_text(
        &self,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, RpcErrorObject> {
        let path = params
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| bad("fs/read_text_file missing path"))?;
        let resolved = self.resolve(path)?;
        match tokio::fs::read_to_string(&resolved).await {
            Ok(content) => Ok(serde_json::json!({ "content": content })),
            Err(e) => Err(RpcErrorObject {
                code: -32002,
                message: format!("reading {path:?}: {e}"),
                data: None,
            }),
        }
    }

    pub async fn write_text(
        &self,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, RpcErrorObject> {
        let path = params
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| bad("fs/write_text_file missing path"))?;
        let content = params
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let resolved = self.resolve(path)?;
        if let Some(parent) = resolved.parent() {
            let _ = tokio::fs::create_dir_all(parent).await;
        }
        match tokio::fs::write(&resolved, content).await {
            Ok(()) => Ok(serde_json::json!({})),
            Err(e) => Err(RpcErrorObject {
                code: -32003,
                message: format!("writing {path:?}: {e}"),
                data: None,
            }),
        }
    }
}

fn bad(msg: &str) -> RpcErrorObject {
    RpcErrorObject {
        code: -32602,
        message: msg.to_string(),
        data: None,
    }
}
