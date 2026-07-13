use anyhow::Result;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct SessionRow {
    pub(super) session_id: String,
    pub(super) handle: String,
    pub(super) agent: String,
    pub(super) workspace: String,
    pub(super) workspace_id: String,
    pub(super) channels: Vec<String>,
    pub(super) channel_ids: Vec<String>,
    pub(super) title: String,
    pub(super) activity: String,
    pub(super) busy: bool,
    pub(super) last_seen: u64,
    pub(super) host: String,
    pub(super) harness: String,
    pub(super) transport: String,
    pub(super) pty_id: Option<String>,
    pub(super) pty_live: bool,
    pub(super) cwd: Option<String>,
    pub(super) command: Vec<String>,
}

impl SessionRow {
    pub(super) fn display_title(&self) -> &str {
        let trimmed = self.title.trim();
        if trimmed.is_empty() {
            "(untitled)"
        } else {
            trimmed
        }
    }

    pub(super) fn title_with_activity(&self) -> String {
        let title = self.display_title();
        let activity = self.activity.trim();
        if activity.is_empty() || activity == title || title == "(untitled)" {
            title.to_string()
        } else {
            format!("{title} - {activity}")
        }
    }

    pub(super) fn search_text(&self) -> String {
        [
            self.handle.as_str(),
            self.agent.as_str(),
            self.title.as_str(),
            self.activity.as_str(),
            self.workspace.as_str(),
            self.workspace_id.as_str(),
            &self.channels.join(" "),
            &self.channel_ids.join(" "),
            self.host.as_str(),
            self.harness.as_str(),
            self.transport.as_str(),
            self.cwd.as_deref().unwrap_or_default(),
        ]
        .join(" ")
    }

    pub(super) async fn resolve_pty_id(&self) -> Result<Option<String>> {
        Ok(self.pty_live.then(|| self.pty_id.clone()).flatten())
    }
}

pub(super) async fn fetch_sessions() -> Result<Vec<SessionRow>> {
    let value = super::super::daemon_call_async("operator_sessions", serde_json::json!({})).await?;
    Ok(rows_from_value(&value))
}

fn rows_from_value(value: &serde_json::Value) -> Vec<SessionRow> {
    let mut rows = value["sessions"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or(&[])
        .iter()
        .filter_map(parse_row)
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| {
        b.busy
            .cmp(&a.busy)
            .then_with(|| b.pty_live.cmp(&a.pty_live))
            .then_with(|| b.last_seen.cmp(&a.last_seen))
            .then_with(|| a.handle.cmp(&b.handle))
    });
    rows
}

fn parse_row(value: &serde_json::Value) -> Option<SessionRow> {
    let session_id = value["session_id"].as_str()?.to_string();
    let endpoint = value.get("endpoint").filter(|value| !value.is_null());
    let channels = value["channels"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    Some(SessionRow {
        session_id,
        handle: value["handle"].as_str().unwrap_or("?").to_string(),
        agent: value["agent"].as_str().unwrap_or("?").to_string(),
        workspace: value["workspace"]["name"]
            .as_str()
            .unwrap_or("")
            .to_string(),
        workspace_id: value["workspace"]["id"].as_str().unwrap_or("").to_string(),
        channels: channels
            .iter()
            .filter_map(|channel| channel["name"].as_str().map(str::to_string))
            .collect(),
        channel_ids: channels
            .iter()
            .filter_map(|channel| channel["id"].as_str().map(str::to_string))
            .collect(),
        title: value["title"].as_str().unwrap_or("").to_string(),
        activity: value["activity"].as_str().unwrap_or("").to_string(),
        busy: value["busy"].as_bool().unwrap_or(false),
        last_seen: value["last_seen"].as_u64().unwrap_or(0),
        host: value["host"].as_str().unwrap_or("").to_string(),
        harness: value["harness"].as_str().unwrap_or("").to_string(),
        transport: value["transport"].as_str().unwrap_or("").to_string(),
        pty_id: endpoint
            .and_then(|endpoint| endpoint["pty_id"].as_str())
            .map(str::to_string),
        pty_live: endpoint
            .and_then(|endpoint| endpoint["live"].as_bool())
            .unwrap_or(false),
        cwd: endpoint
            .and_then(|endpoint| endpoint["cwd"].as_str())
            .or_else(|| value["workspace"]["path"].as_str())
            .map(str::to_string),
        command: endpoint
            .and_then(|endpoint| endpoint["command"].as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_canonical_local_projection_with_non_pty_rows() {
        let value = serde_json::json!({
            "sessions": [
                {
                    "session_id": "s1",
                    "handle": "opal-codex",
                    "agent": "codex",
                    "workspace": {"id": "root", "name": "tenex-edge", "path": "/repo"},
                    "channels": [{"id": "root", "name": "tenex-edge"}],
                    "title": "shipping the TUI",
                    "activity": "running tests",
                    "busy": true,
                    "last_seen": 12,
                    "host": "laptop",
                    "harness": "codex",
                    "transport": "harness",
                    "endpoint": null
                }
            ]
        });

        let rows = rows_from_value(&value);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].session_id, "s1");
        assert_eq!(rows[0].handle, "opal-codex");
        assert_eq!(rows[0].cwd.as_deref(), Some("/repo"));
        assert_eq!(
            rows[0].title_with_activity(),
            "shipping the TUI - running tests"
        );
        let search = rows[0].search_text();
        assert!(search.contains("tenex-edge"));
        assert!(search.contains("laptop"));
        assert!(search.contains("codex"));
        assert!(rows[0].pty_id.is_none());
    }

    #[tokio::test]
    async fn non_pty_row_is_inspectable_without_attempting_daemon_attach() {
        let row = SessionRow {
            session_id: "local-non-pty".into(),
            handle: "opal-codex".into(),
            ..SessionRow::default()
        };

        assert_eq!(row.resolve_pty_id().await.unwrap(), None);
    }
}
