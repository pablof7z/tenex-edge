use super::*;

pub(in crate::agent_catalog) fn discover_codex_named_profiles(
    dir: &Path,
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
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let Some(slug) = file_name.strip_suffix(".config.toml") else {
            continue;
        };
        if !valid_profile_name(slug) {
            continue;
        }
        let body = std::fs::read_to_string(&path)
            .with_context(|| format!("reading named Codex profile {}", path.display()))?;
        let value: toml::Value = toml::from_str(&body)
            .with_context(|| format!("parsing named Codex profile {}", path.display()))?;
        if optional_string(&value, "developer_instructions").is_none() {
            continue;
        }
        let use_criteria = optional_string(&value, "description")
            .or_else(|| comment_field(&body, "Summary"))
            .unwrap_or_default();
        prefer_named_profile(out, slug, &scope);
        out.push(NativeAgentProfile {
            slug: slug.to_string(),
            use_criteria,
            harness: Harness::Codex,
            path: path.clone(),
            scope: scope.clone(),
            modified_at: modified_at(&path).unwrap_or(0),
        });
    }
    Ok(())
}

fn prefer_named_profile(out: &mut Vec<NativeAgentProfile>, slug: &str, scope: &AgentScope) {
    out.retain(|profile| {
        profile.harness != Harness::Codex || &profile.scope != scope || profile.slug != slug
    });
}

fn valid_profile_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn optional_string(value: &toml::Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(toml::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn comment_field(body: &str, key: &str) -> Option<String> {
    let prefix = format!("# {key}:");
    body.lines()
        .find_map(|line| line.trim().strip_prefix(&prefix))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}
