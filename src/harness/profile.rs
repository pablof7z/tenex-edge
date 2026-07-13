//! Turn a bundle's opaque `profile` object into a `ProfilePlan` the launch
//! layer applies uniformly, so no launch site needs to know codex-vs-claude
//! details.

use std::path::{Path, PathBuf};

use super::driver::ProfileMechanism;
use crate::session::Harness;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodexHomePlan {
    pub source: PathBuf,
    pub target: PathBuf,
}

/// A materialized plan: extra argv/env plus scratch files to write pre-launch.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProfilePlan {
    /// Extra argv appended after `base_argv` (codex `-c k=v` pairs, or the
    /// claude `--settings <path>` selector).
    pub extra_argv: Vec<String>,
    /// Extra env for the child (e.g. `OPENCODE_CONFIG=<path>`).
    pub extra_env: Vec<(String, String)>,
    /// Files to materialize before launch: (absolute path, contents).
    pub files: Vec<(PathBuf, String)>,
    /// Isolated Codex home to prepare before writing the composed config.
    pub codex_home: Option<CodexHomePlan>,
}

impl ProfilePlan {
    pub fn extend(&mut self, other: Self) {
        self.extra_argv.extend(other.extra_argv);
        self.extra_env.extend(other.extra_env);
        self.files.extend(other.files);
        if other.codex_home.is_some() {
            self.codex_home = other.codex_home;
        }
    }

    pub fn materialize(&self) -> anyhow::Result<()> {
        if let Some(home) = &self.codex_home {
            super::codex_profile::prepare_home(home)?;
        }
        for (path, contents) in &self.files {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    anyhow::anyhow!("creating profile dir {}: {e}", parent.display())
                })?;
            }
            std::fs::write(path, contents)
                .map_err(|e| anyhow::anyhow!("writing profile file {}: {e}", path.display()))?;
        }
        Ok(())
    }
}

fn is_empty_object(v: &serde_json::Value) -> bool {
    v.as_object().map(|o| o.is_empty()).unwrap_or(false) || v.is_null()
}

/// Encode a scalar as a bare string; non-scalars as compact JSON (codex accepts
/// JSON/TOML-ish values after `key=`).
fn scalar_or_json(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        other => other.to_string(),
    }
}

/// Build the profile plan for a `(harness, mechanism, profile)` triple.
///
/// `scratch_dir` is a per-session directory the settings file is written into
/// (never the user's repo). `harness` selects how the child is pointed at that
/// file for `CwdSettingsFile`.
pub fn plan_profile(
    harness: Harness,
    mech: ProfileMechanism,
    profile: Option<&serde_json::Value>,
    scratch_dir: &Path,
) -> anyhow::Result<ProfilePlan> {
    let Some(profile) = profile.filter(|v| !is_empty_object(v)) else {
        return Ok(ProfilePlan::default());
    };
    match mech {
        ProfileMechanism::CliConfigFlags { flag } => {
            let obj = profile.as_object().ok_or_else(|| {
                anyhow::anyhow!("profile for a CLI-config-flags harness must be a JSON object")
            })?;
            let mut extra_argv = Vec::with_capacity(obj.len() * 2);
            for (k, v) in obj {
                extra_argv.push(flag.to_string());
                extra_argv.push(format!("{k}={}", scalar_or_json(v)));
            }
            Ok(ProfilePlan {
                extra_argv,
                ..Default::default()
            })
        }
        ProfileMechanism::CwdSettingsFile { relpath } => {
            let path = scratch_dir.join(relpath);
            let contents = serde_json::to_string_pretty(profile)?;
            let mut plan = ProfilePlan {
                files: vec![(path.clone(), contents)],
                ..Default::default()
            };
            // Point the child at the scratch file. The selector is decided in
            // exactly one place, keyed by harness.
            match harness {
                Harness::ClaudeCode => {
                    plan.extra_argv.push("--settings".to_string());
                    plan.extra_argv.push(path.to_string_lossy().into_owned());
                }
                Harness::Opencode => {
                    plan.extra_env.push((
                        "OPENCODE_CONFIG".to_string(),
                        path.to_string_lossy().into_owned(),
                    ));
                }
                _ => {
                    anyhow::bail!("harness {} has no settings-file selector", harness.as_str());
                }
            }
            Ok(plan)
        }
        ProfileMechanism::Unsupported => {
            anyhow::bail!("harness does not support a `profile`; remove it from the bundle")
        }
    }
}
