use super::data::SessionRow;
use super::{data, pane};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use pane::PtyPane;
use std::time::Duration;

pub(super) struct App {
    pub(super) sessions: Vec<SessionRow>,
    pub(super) selected: usize,
    pub(super) panes: Vec<PtyPane>,
    pub(super) active_pane: Option<usize>,
    pub(super) input_mode: bool,
    pub(super) status: String,
    pub(super) pending_kill: Option<String>,
    pub(super) refresh_interval: Duration,
}

impl App {
    pub(super) fn new(refresh_interval: Duration) -> Self {
        Self {
            sessions: Vec::new(),
            selected: 0,
            panes: Vec::new(),
            active_pane: None,
            input_mode: false,
            status: String::new(),
            pending_kill: None,
            refresh_interval,
        }
    }

    pub(super) async fn refresh(&mut self) -> Result<()> {
        let previous = self.selected_session_id().map(str::to_string);
        self.sessions = data::fetch_sessions().await?;
        self.selected = previous
            .and_then(|id| self.sessions.iter().position(|s| s.session_id == id))
            .unwrap_or_else(|| self.selected.min(self.sessions.len().saturating_sub(1)));
        self.sync_pane_titles();
        self.status = format!("{} session(s)", self.sessions.len());
        Ok(())
    }

    fn selected_session_id(&self) -> Option<&str> {
        self.sessions
            .get(self.selected)
            .map(|s| s.session_id.as_str())
    }

    pub(super) async fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        if self.input_mode {
            if input_escape_key(key) {
                self.input_mode = false;
                self.status = "control mode".to_string();
                return Ok(true);
            }
            if let Some(bytes) = pane::encode_key(key) {
                self.forward_bytes(&bytes)?;
            }
            return Ok(true);
        }

        if key.code != KeyCode::Char('K') {
            self.pending_kill = None;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(false),
            KeyCode::Char('r') => self.refresh().await?,
            KeyCode::Up | KeyCode::Char('k') => self.move_selection(-1),
            KeyCode::Down | KeyCode::Char('j') => self.move_selection(1),
            KeyCode::Enter | KeyCode::Char('a') => self.open_selected(true).await?,
            KeyCode::Char('o') => self.open_selected(false).await?,
            KeyCode::Char('K') => self.confirm_or_kill_selected().await?,
            KeyCode::Tab | KeyCode::Right => self.cycle_pane(1),
            KeyCode::BackTab | KeyCode::Left => self.cycle_pane(-1),
            KeyCode::Char('x') | KeyCode::Char('c') => self.close_active_pane(),
            KeyCode::Char(ch) if ('1'..='9').contains(&ch) => {
                self.focus_pane((ch as u8 - b'1') as usize, false);
            }
            _ => {}
        }
        Ok(true)
    }

    pub(super) fn forward_paste(&mut self, text: String) -> Result<()> {
        if self.input_mode {
            self.forward_bytes(text.as_bytes())?;
        }
        Ok(())
    }

    fn forward_bytes(&mut self, bytes: &[u8]) -> Result<()> {
        let Some(idx) = self.active_pane else {
            return Ok(());
        };
        if !self.panes[idx].write_input(bytes)? {
            self.status = format!("{} disconnected", self.panes[idx].title());
            self.input_mode = false;
        }
        Ok(())
    }

    fn move_selection(&mut self, delta: isize) {
        if self.sessions.is_empty() {
            self.selected = 0;
            return;
        }
        let len = self.sessions.len() as isize;
        self.selected = (self.selected as isize + delta).rem_euclid(len) as usize;
    }

    async fn open_selected(&mut self, input: bool) -> Result<()> {
        let Some(row) = self.sessions.get(self.selected).cloned() else {
            self.status = "no sessions".to_string();
            return Ok(());
        };
        let Some(pty_id) = row.resolve_pty_id().await? else {
            self.status = format!("{} has no live PTY endpoint", row.agent);
            return Ok(());
        };
        if let Some(idx) = self.panes.iter().position(|p| p.pty_id() == pty_id) {
            self.focus_pane(idx, input);
            return Ok(());
        }
        let pane = match PtyPane::open(&row, pty_id) {
            Ok(pane) => pane,
            Err(e) => {
                self.status = format!("could not attach {}: {e:#}", row.agent);
                return Ok(());
            }
        };
        self.panes.push(pane);
        self.focus_pane(self.panes.len() - 1, input);
        Ok(())
    }

    async fn confirm_or_kill_selected(&mut self) -> Result<()> {
        let Some(row) = self.sessions.get(self.selected).cloned() else {
            self.status = "no session selected".to_string();
            return Ok(());
        };
        if self.pending_kill.as_deref() != Some(&row.session_id) {
            self.pending_kill = Some(row.session_id.clone());
            self.status = format!("press K again to kill {}", row_title(&row));
            return Ok(());
        }
        self.pending_kill = None;
        self.kill_session(row).await
    }

    async fn kill_session(&mut self, row: SessionRow) -> Result<()> {
        let title = row_title(&row);
        let v = super::super::daemon_call_async(
            "session_kill",
            serde_json::json!({ "session": row.session_id }),
        )
        .await?;
        self.close_panes_for_session(&row.session_id);
        self.refresh().await?;
        let killed = v["killed"].as_bool().unwrap_or(false);
        let ended = v["ended"].as_bool().unwrap_or(false);
        let reason = v["reason"].as_str().unwrap_or("");
        self.status = match (killed, ended, reason.is_empty()) {
            (true, true, _) => format!("killed {title}"),
            (false, true, true) => format!("ended {title}"),
            (_, true, false) => format!("ended {title}; process stop failed: {reason}"),
            (_, false, false) => format!("could not kill {title}: {reason}"),
            _ => format!("could not kill {title}"),
        };
        Ok(())
    }

    fn focus_pane(&mut self, idx: usize, input: bool) {
        if idx < self.panes.len() {
            self.active_pane = Some(idx);
            self.input_mode = input;
            self.status = if input {
                format!(
                    "attached to {}; Esc/Ctrl-G controls",
                    self.panes[idx].title()
                )
            } else {
                format!("focused {}", self.panes[idx].title())
            };
        }
    }

    fn cycle_pane(&mut self, delta: isize) {
        if self.panes.is_empty() {
            self.active_pane = None;
            return;
        }
        let current = self.active_pane.unwrap_or(0) as isize;
        let len = self.panes.len() as isize;
        self.focus_pane((current + delta).rem_euclid(len) as usize, false);
    }

    fn close_active_pane(&mut self) {
        let Some(idx) = self.active_pane else {
            return;
        };
        if idx < self.panes.len() {
            self.panes[idx].shutdown();
            let closed = self.panes.remove(idx).title().to_string();
            self.active_pane = if self.panes.is_empty() {
                None
            } else {
                Some(idx.min(self.panes.len() - 1))
            };
            self.input_mode = false;
            self.status = format!("closed {closed}");
        }
    }

    fn close_panes_for_session(&mut self, session_id: &str) {
        let mut idx = 0;
        while idx < self.panes.len() {
            if self.panes[idx].session_id() == session_id {
                self.panes[idx].shutdown();
                self.panes.remove(idx);
            } else {
                idx += 1;
            }
        }
        self.active_pane = self
            .active_pane
            .filter(|idx| *idx < self.panes.len())
            .or_else(|| (!self.panes.is_empty()).then_some(0));
        self.input_mode = false;
    }

    pub(super) fn poll_panes(&mut self) {
        for pane in &mut self.panes {
            if let Err(e) = pane.poll_output() {
                self.status = format!("{} read failed: {e:#}", pane.title());
            }
        }
    }

    fn sync_pane_titles(&mut self) {
        for pane in &mut self.panes {
            if let Some(row) = self
                .sessions
                .iter()
                .find(|s| s.session_id == pane.session_id())
            {
                pane.refresh_title(row);
            }
        }
    }
}

fn row_title(row: &SessionRow) -> String {
    format!("{} - {}", row.agent, row.title_with_activity())
}

pub(super) fn input_escape_key(key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => !key.modifiers.contains(KeyModifiers::ALT),
        KeyCode::Char(']') | KeyCode::Char('\u{1d}') => {
            key.modifiers.contains(KeyModifiers::CONTROL) || key.code == KeyCode::Char('\u{1d}')
        }
        KeyCode::Char('g') | KeyCode::Char('G') => key.modifiers.contains(KeyModifiers::CONTROL),
        _ => false,
    }
}
