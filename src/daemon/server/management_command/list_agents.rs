//! Agent roster listing for backend-addressed management commands.

use super::super::DaemonState;
use anyhow::Result;
use std::collections::BTreeMap;
use std::sync::Arc;

pub(super) fn list_agents(state: &Arc<DaemonState>) -> Result<String> {
    let now = crate::util::now_secs();
    let rows = state.with_store(|s| s.list_agent_roster())?;
    let mut by_slug: BTreeMap<String, crate::state::AgentAvailability> = BTreeMap::new();
    for row in rows {
        if row.host != state.host {
            continue;
        }
        by_slug
            .entry(row.slug.clone())
            .and_modify(|existing| {
                if row.updated_at >= existing.updated_at {
                    *existing = row.clone();
                }
            })
            .or_insert(row);
    }
    if by_slug.is_empty() {
        return Ok(format!("mgmt ok: no agents known on {}", state.host));
    }
    let mut lines = vec![format!(
        "mgmt ok: {} agent(s) on {}",
        by_slug.len(),
        state.host
    )];
    for (slug, row) in by_slug {
        let criteria = row.use_criteria.trim();
        let age = crate::util::relative_time(row.updated_at, now);
        if criteria.is_empty() {
            lines.push(format!("- {slug} (updated {age})"));
        } else {
            lines.push(format!("- {slug}: {criteria} (updated {age})"));
        }
    }
    Ok(lines.join("\n"))
}
