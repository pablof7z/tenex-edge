use super::{AgentScope, NativeAgentProfile};
use crate::session::Harness;
use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::path::Path;

pub(super) fn discover_dir(
    dir: &Path,
    harness: Harness,
    scope: AgentScope,
    out: &mut Vec<NativeAgentProfile>,
) -> Result<()> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error).with_context(|| format!("reading {}", dir.display())),
    };
    for entry in entries {
        let path = entry?.path();
        let expected = if harness == Harness::Codex {
            "toml"
        } else {
            "md"
        };
        if path.extension().and_then(|value| value.to_str()) != Some(expected) {
            continue;
        }
        out.push(parse_profile(&path, harness, scope.clone())?);
    }
    Ok(())
}

fn parse_profile(path: &Path, harness: Harness, scope: AgentScope) -> Result<NativeAgentProfile> {
    let body = std::fs::read_to_string(path)
        .with_context(|| format!("reading native agent profile {}", path.display()))?;
    let (slug, use_criteria) = if harness == Harness::Codex {
        parse_codex(&body, path)?
    } else {
        parse_markdown(&body, path)?
    };
    let modified_at = std::fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    Ok(NativeAgentProfile {
        slug,
        use_criteria,
        harness,
        path: path.to_path_buf(),
        scope,
        modified_at,
    })
}

fn parse_codex(body: &str, path: &Path) -> Result<(String, String)> {
    let value: toml::Value =
        toml::from_str(body).with_context(|| format!("parsing Codex agent {}", path.display()))?;
    let name = required_string(&value, "name", path)?;
    let description = required_string(&value, "description", path)?;
    required_string(&value, "developer_instructions", path)?;
    Ok((name, description))
}

fn required_string(value: &toml::Value, key: &str, path: &Path) -> Result<String> {
    value
        .get(key)
        .and_then(toml::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .with_context(|| format!("native agent {} requires non-empty {key:?}", path.display()))
}

fn parse_markdown(body: &str, path: &Path) -> Result<(String, String)> {
    let frontmatter = body
        .strip_prefix("---\n")
        .and_then(|rest| rest.split_once("\n---"))
        .map(|(frontmatter, _)| frontmatter)
        .with_context(|| format!("native agent {} requires YAML frontmatter", path.display()))?;
    let fields = frontmatter
        .lines()
        .filter_map(|line| line.split_once(':'))
        .map(|(key, value)| (key.trim(), unquote(value.trim())))
        .collect::<BTreeMap<_, _>>();
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .context("native agent path has no UTF-8 filename stem")?;
    let slug = fields
        .get("name")
        .map(String::as_str)
        .filter(|value| !value.is_empty())
        .unwrap_or(stem)
        .to_string();
    let description = fields
        .get("description")
        .cloned()
        .filter(|value| !value.is_empty())
        .with_context(|| format!("native agent {} requires a description", path.display()))?;
    Ok((slug, description))
}

fn unquote(value: &str) -> String {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .or_else(|| {
            value
                .strip_prefix('\'')
                .and_then(|value| value.strip_suffix('\''))
        })
        .unwrap_or(value)
        .to_string()
}
