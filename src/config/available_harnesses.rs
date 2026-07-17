use super::config_path;
use crate::session::Harness;
use anyhow::{Context, Result};
use serde_json::Value;
use std::path::{Path, PathBuf};

const FIELD: &str = "availableHarnesses";

pub fn detect() -> Result<Vec<Harness>> {
    let home = std::env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .context("HOME is required to detect available harnesses")?;
    Ok(detect_with(&home, std::env::var_os("PATH").as_deref()))
}

pub fn ensure_configured() -> Result<Vec<Harness>> {
    let path = config_path();
    let body = match std::fs::read_to_string(&path) {
        Ok(body) => body,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return detect(),
        Err(error) => return Err(error).with_context(|| format!("reading {}", path.display())),
    };
    let mut root: Value =
        serde_json::from_str(&body).with_context(|| format!("parsing {}", path.display()))?;
    let object = root
        .as_object_mut()
        .with_context(|| format!("{} must contain a JSON object", path.display()))?;
    if let Some(value) = object.get(FIELD) {
        return parse(value).with_context(|| format!("parsing {FIELD} in {}", path.display()));
    }

    let detected = detect()?;
    object.insert(
        FIELD.to_string(),
        Value::Array(
            detected
                .iter()
                .map(|harness| Value::String(harness.agent_slug().to_string()))
                .collect(),
        ),
    );
    atomic_write(&path, &serde_json::to_string_pretty(&root)?)?;
    Ok(detected)
}

pub(super) fn parse(value: &Value) -> Result<Vec<Harness>> {
    let values = value
        .as_array()
        .context("availableHarnesses must be an array of harness names")?;
    let mut harnesses = Vec::new();
    for value in values {
        let name = value
            .as_str()
            .context("availableHarnesses entries must be strings")?;
        let harness = Harness::from_str(name);
        if harness == Harness::Unknown {
            anyhow::bail!("unknown available harness {name:?}");
        }
        if !harnesses.contains(&harness) {
            harnesses.push(harness);
        }
    }
    Ok(harnesses)
}

fn detect_with(home: &Path, path: Option<&std::ffi::OsStr>) -> Vec<Harness> {
    let candidates = [
        (Harness::ClaudeCode, ".claude", "claude"),
        (Harness::Codex, ".codex", "codex"),
        (Harness::Opencode, ".config/opencode", "opencode"),
        (Harness::Grok, ".grok", "grok"),
    ];
    candidates
        .into_iter()
        .filter(|(_, dir, bin)| home.join(dir).exists() || bin_on_path(path, bin))
        .map(|(harness, _, _)| harness)
        .collect()
}

fn bin_on_path(path: Option<&std::ffi::OsStr>, bin: &str) -> bool {
    path.into_iter()
        .flat_map(std::env::split_paths)
        .any(|dir| dir.join(bin).is_file())
}

fn atomic_write(path: &Path, body: &str) -> Result<()> {
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, format!("{body}\n"))
        .with_context(|| format!("writing {}", tmp.display()))?;
    std::fs::rename(&tmp, path).with_context(|| format!("renaming into {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_env::EnvGuard;

    #[test]
    fn detects_home_directories_and_path_binaries_in_stable_order() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join(".codex")).unwrap();
        let bin = root.path().join("bin");
        std::fs::create_dir(&bin).unwrap();
        std::fs::write(bin.join("opencode"), "").unwrap();

        assert_eq!(
            detect_with(root.path(), Some(bin.as_os_str())),
            [Harness::Codex, Harness::Opencode]
        );
    }

    #[test]
    fn parses_aliases_deduplicates_and_rejects_unknown_harnesses() {
        let parsed = parse(&serde_json::json!(["claude", "claude-code", "codex"])).unwrap();
        assert_eq!(parsed, [Harness::ClaudeCode, Harness::Codex]);
        assert!(parse(&serde_json::json!(["other"])).is_err());
    }

    #[test]
    fn upserts_only_when_available_harnesses_is_absent() {
        let root = tempfile::tempdir().unwrap();
        let config = root.path().join("mosaico/config.json");
        std::fs::create_dir_all(config.parent().unwrap()).unwrap();
        std::fs::write(&config, r#"{"backendName":"test"}"#).unwrap();
        std::fs::create_dir(root.path().join(".codex")).unwrap();
        let bin = root.path().join("bin");
        std::fs::create_dir(&bin).unwrap();
        let mut env = EnvGuard::set("HOME", root.path());
        env.set_var("MOSAICO_CONFIG", &config);
        env.set_var("MOSAICO_HOME", root.path().join("mosaico"));
        env.set_var("MOSAICO_ISOLATED_HOME_OK", "1");
        env.set_var("PATH", &bin);

        assert_eq!(ensure_configured().unwrap(), [Harness::Codex]);
        std::fs::create_dir_all(root.path().join(".config/opencode")).unwrap();
        assert_eq!(ensure_configured().unwrap(), [Harness::Codex]);
        let saved: Value = serde_json::from_str(&std::fs::read_to_string(config).unwrap()).unwrap();
        assert_eq!(saved[FIELD], serde_json::json!(["codex"]));
        assert_eq!(saved["backendName"], "test");
    }
}
