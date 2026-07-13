use super::app::App;
use super::selection::KillTarget;
use anyhow::Result;

impl App {
    pub(super) async fn confirm_or_kill_selected(&mut self) -> Result<()> {
        let targets = self.kill_targets();
        if targets.is_empty() {
            self.status = "no session selected".to_string();
            return Ok(());
        }
        if self.pending_kill.as_ref() != Some(&targets) {
            self.status = format!(
                "press K again to kill {} session(s); any other key cancels",
                targets.len()
            );
            self.pending_kill = Some(targets);
            return Ok(());
        }
        let targets = self.pending_kill.take().unwrap_or_default();
        self.kill_sessions(targets).await
    }

    async fn kill_sessions(&mut self, targets: Vec<KillTarget>) -> Result<()> {
        let mut killed = 0usize;
        let mut failures = Vec::new();
        for target in &targets {
            let result = super::super::daemon_call_async(
                "session_kill",
                serde_json::json!({ "session": target.session_id }),
            )
            .await?;
            self.close_panes_for_session(&target.session_id);
            self.marked.remove(&target.session_id);
            if result["killed"].as_bool().unwrap_or(false) {
                killed += 1;
            } else {
                failures.push(format!(
                    "{}: {}",
                    target.label,
                    result["reason"].as_str().unwrap_or("kill failed")
                ));
            }
        }
        self.refresh().await?;
        self.status = if failures.is_empty() {
            format!("killed {killed} session(s)")
        } else {
            format!("killed {killed}; failed: {}", failures.join("; "))
        };
        Ok(())
    }
}
