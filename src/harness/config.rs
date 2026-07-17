//! `~/.mosaico/harnesses.json` loader + serde.
//!
//! The file is a map of **bundle name -> bundle spec**. A bundle is the
//! user-facing name you spawn (`codex-acp`, `planner`, …); it binds a `harness`
//! (which CLI) to a `transport` (how mosaico drives it) plus operational args.
//! Missing file => empty map; malformed JSON => hard error. There are no
//! built-in bundle fallbacks.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

use crate::session::Harness;

/// Whole-file shape: bundle name -> bundle spec.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct HarnessesConfig {
    pub bundles: BTreeMap<String, HarnessBundle>,
}

/// One bundle: which CLI, how we drive it, and opaque tuning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HarnessBundle {
    /// Which underlying CLI. Parsed via `Harness::from_str` so `"claude"` and
    /// `"claude-code"` both resolve; `Harness::Unknown` is rejected.
    #[serde(with = "harness_serde")]
    pub harness: Harness,
    /// How mosaico drives that CLI.
    pub transport: Transport,
    /// Bundle-owned operational flags appended to the code-owned driver argv.
    /// The executable itself is never configurable JSON.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
}

/// How mosaico drives a CLI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Transport {
    /// Interactive terminal via the existing portable-pty supervisor.
    Pty,
    /// ACP: JSON-RPC 2.0 over stdio (OpenCode native, Claude via adapter).
    Acp,
    /// Codex `app-server`: its own JSON-RPC dialect.
    AppServer,
    /// One-shot run-to-exit (`claude -p`, `codex exec`, `opencode run`).
    HeadlessExec,
}

impl Transport {
    pub fn as_str(&self) -> &'static str {
        match self {
            Transport::Pty => "pty",
            Transport::Acp => "acp",
            Transport::AppServer => "app-server",
            Transport::HeadlessExec => "headless-exec",
        }
    }
}

impl HarnessesConfig {
    /// Load `<mosaico_home>/harnesses.json`. Absent file => empty map; a launch
    /// still fails unless its agent-selected bundle is explicitly configured.
    pub fn load() -> anyhow::Result<Self> {
        let path = crate::config::mosaico_home().join("harnesses.json");
        Self::load_from(&path)
    }

    pub fn load_from(path: &std::path::Path) -> anyhow::Result<Self> {
        let bytes = match std::fs::read(path) {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(e) => {
                return Err(e).map_err(|e| {
                    anyhow::anyhow!("reading harnesses config {}: {e}", path.display())
                })
            }
        };
        serde_json::from_slice(&bytes)
            .map_err(|e| anyhow::anyhow!("parsing harnesses config {}: {e}", path.display()))
    }

    pub fn get(&self, bundle: &str) -> Option<&HarnessBundle> {
        self.bundles.get(bundle)
    }

    /// Persist the complete bundle map with a temp-file rename.
    pub fn save_to(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating {}", parent.display()))?;
        }
        let body = serde_json::to_string_pretty(self).context("serializing harnesses config")?;
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, format!("{body}\n"))
            .with_context(|| format!("writing {}", tmp.display()))?;
        std::fs::rename(&tmp, path)
            .with_context(|| format!("renaming {} into {}", tmp.display(), path.display()))
    }

    /// Reuse an exactly matching bundle or insert it under a deterministic name.
    ///
    /// A conflicting canonical name gets a numeric suffix. Existing entries are
    /// never changed, so their operational args remain operator-owned.
    pub fn ensure_bundle(
        &mut self,
        canonical_name: &str,
        desired: HarnessBundle,
    ) -> Result<(String, bool)> {
        let canonical_name = canonical_name.trim();
        if canonical_name.is_empty() {
            anyhow::bail!("harness bundle name must not be empty");
        }
        if let Some((name, _)) = self
            .bundles
            .iter()
            .find(|(_, configured)| **configured == desired)
        {
            return Ok((name.clone(), false));
        }

        let name = if !self.bundles.contains_key(canonical_name) {
            canonical_name.to_string()
        } else {
            (2..)
                .map(|suffix| format!("{canonical_name}-{suffix}"))
                .find(|candidate| !self.bundles.contains_key(candidate))
                .expect("unbounded bundle suffix search")
        };
        self.bundles.insert(name.clone(), desired);
        Ok((name, true))
    }
}

/// (De)serialize `Harness` through its existing `from_str`/`as_str` so the
/// config axis stays independent of the enum's Rust spelling and rejects
/// unknown harnesses loudly.
mod harness_serde {
    use super::Harness;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(h: &Harness, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(h.as_str())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Harness, D::Error> {
        let raw = String::deserialize(d)?;
        match Harness::from_str(&raw) {
            Harness::Unknown => Err(serde::de::Error::custom(format!(
                "unknown harness {raw:?} (expected claude-code|codex|opencode|grok)"
            ))),
            h => Ok(h),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bundle(harness: Harness, transport: Transport, args: &[&str]) -> HarnessBundle {
        HarnessBundle {
            harness,
            transport,
            args: args.iter().map(|arg| (*arg).to_string()).collect(),
        }
    }

    #[test]
    fn ensure_bundle_reuses_an_exact_existing_entry() {
        let mut config = HarnessesConfig::default();
        config.bundles.insert(
            "my-claude".into(),
            bundle(Harness::ClaudeCode, Transport::Acp, &[]),
        );

        let (name, created) = config
            .ensure_bundle(
                "claude-acp",
                bundle(Harness::ClaudeCode, Transport::Acp, &[]),
            )
            .unwrap();

        assert_eq!(name, "my-claude");
        assert!(!created);
        assert_eq!(config.bundles.len(), 1);
    }

    #[test]
    fn ensure_bundle_preserves_conflicts_and_uses_stable_suffixes() {
        let mut config = HarnessesConfig::default();
        let tuned = bundle(
            Harness::ClaudeCode,
            Transport::Pty,
            &["--dangerously-skip-permissions"],
        );
        config.bundles.insert("claude-pty".into(), tuned.clone());
        config.bundles.insert(
            "claude-pty-2".into(),
            bundle(Harness::Codex, Transport::Pty, &[]),
        );

        let desired = bundle(Harness::ClaudeCode, Transport::Pty, &[]);
        let (name, created) = config.ensure_bundle("claude-pty", desired.clone()).unwrap();

        assert!(created);
        assert_eq!(name, "claude-pty-3");
        assert_eq!(config.bundles["claude-pty"], tuned);
        assert_eq!(config.bundles["claude-pty-3"], desired);
    }

    #[test]
    fn save_round_trip_preserves_every_entry_and_arg() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested/harnesses.json");
        let mut config = HarnessesConfig::default();
        config.bundles.insert(
            "claude-pty".into(),
            bundle(
                Harness::ClaudeCode,
                Transport::Pty,
                &["--dangerously-skip-permissions"],
            ),
        );
        config.bundles.insert(
            "codex-app".into(),
            bundle(Harness::Codex, Transport::AppServer, &[]),
        );

        config.save_to(&path).unwrap();

        assert_eq!(HarnessesConfig::load_from(&path).unwrap(), config);
        assert!(std::fs::read_to_string(path).unwrap().ends_with('\n'));
    }

    #[test]
    fn ensure_bundle_rejects_a_blank_name() {
        let mut config = HarnessesConfig::default();
        assert!(config
            .ensure_bundle("  ", bundle(Harness::Codex, Transport::Pty, &[]))
            .is_err());
    }
}
