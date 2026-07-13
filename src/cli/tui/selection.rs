use super::app::App;
use super::data::SessionRow;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct KillTarget {
    pub(super) session_id: String,
    pub(super) label: String,
}

impl App {
    pub(super) fn visible_indices(&self) -> Vec<usize> {
        if self.search_query.is_empty() {
            return (0..self.sessions.len()).collect();
        }
        let matcher = SkimMatcherV2::default().ignore_case();
        let mut matches = self
            .sessions
            .iter()
            .enumerate()
            .filter_map(|(idx, row)| {
                matcher
                    .fuzzy_match(&row.search_text(), &self.search_query)
                    .map(|score| (idx, score))
            })
            .collect::<Vec<_>>();
        matches.sort_by(|(a_idx, a_score), (b_idx, b_score)| {
            b_score.cmp(a_score).then_with(|| a_idx.cmp(b_idx))
        });
        matches.into_iter().map(|(idx, _)| idx).collect()
    }

    pub(super) fn selected_row(&self) -> Option<&SessionRow> {
        self.visible_indices()
            .into_iter()
            .find(|idx| *idx == self.selected)
            .and_then(|idx| self.sessions.get(idx))
    }

    pub(super) fn selected_view_index(&self) -> Option<usize> {
        self.visible_indices()
            .iter()
            .position(|idx| *idx == self.selected)
    }

    pub(super) fn ensure_selection_visible(&mut self) {
        let visible = self.visible_indices();
        if !visible.contains(&self.selected) {
            self.selected = visible.first().copied().unwrap_or(0);
        }
    }

    pub(super) fn move_selection(&mut self, delta: isize) {
        let visible = self.visible_indices();
        if visible.is_empty() {
            self.selected = 0;
            return;
        }
        let current = visible
            .iter()
            .position(|idx| *idx == self.selected)
            .unwrap_or(0) as isize;
        let next = (current + delta).rem_euclid(visible.len() as isize) as usize;
        self.selected = visible[next];
    }

    pub(super) fn toggle_selected(&mut self) {
        let Some(id) = self.selected_row().map(|row| row.session_id.clone()) else {
            return;
        };
        if !self.marked.remove(&id) {
            self.marked.insert(id);
        }
        self.status = format!("{} selected", self.marked.len());
    }

    pub(super) fn toggle_all_visible(&mut self) {
        let ids = self
            .visible_indices()
            .into_iter()
            .map(|idx| self.sessions[idx].session_id.clone())
            .collect::<Vec<_>>();
        if ids.iter().all(|id| self.marked.contains(id)) {
            for id in ids {
                self.marked.remove(&id);
            }
        } else {
            self.marked.extend(ids);
        }
        self.status = format!("{} selected", self.marked.len());
    }

    pub(super) fn prune_marked(&mut self) {
        self.marked
            .retain(|id| self.sessions.iter().any(|row| &row.session_id == id));
    }

    pub(super) fn kill_targets(&self) -> Vec<KillTarget> {
        let ids = if self.marked.is_empty() {
            self.selected_row()
                .map(|row| vec![row.session_id.clone()])
                .unwrap_or_default()
        } else {
            self.marked.iter().cloned().collect()
        };
        self.sessions
            .iter()
            .filter(|row| ids.contains(&row.session_id))
            .map(|row| KillTarget {
                session_id: row.session_id.clone(),
                label: format!("@{} - {}", row.handle, row.title_with_activity()),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn row(id: &str, handle: &str, workspace: &str, title: &str) -> SessionRow {
        SessionRow {
            session_id: id.to_string(),
            handle: handle.to_string(),
            workspace: workspace.to_string(),
            title: title.to_string(),
            ..SessionRow::default()
        }
    }

    #[test]
    fn fuzzy_search_matches_across_projection_fields() {
        let mut app = App::new(Duration::from_secs(2));
        app.sessions = vec![
            row("s1", "opal-codex", "tenex-edge", "shipping tui"),
            row("s2", "river-claude", "skills", "writing docs"),
        ];
        app.search_query = "opltui".to_string();

        assert_eq!(app.visible_indices(), vec![0]);
    }

    #[test]
    fn toggle_all_only_changes_visible_rows() {
        let mut app = App::new(Duration::from_secs(2));
        app.sessions = vec![
            row("s1", "opal-codex", "tenex-edge", "shipping tui"),
            row("s2", "river-claude", "skills", "writing docs"),
        ];
        app.search_query = "skills".to_string();

        app.toggle_all_visible();
        assert_eq!(app.marked.into_iter().collect::<Vec<_>>(), vec!["s2"]);
    }
}
