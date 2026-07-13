mod render;
#[cfg(test)]
mod tests;

use super::SessionChoice;
use anyhow::{Context, Result};
use crossterm::{
    cursor::Show,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute, terminal,
};
use ratatui::{backend::CrosstermBackend, layout::Rect, Terminal, TerminalOptions, Viewport};
use std::{collections::BTreeSet, io};

const CHROME_ROWS: u16 = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PickerExit {
    Submit,
    Cancel,
}

#[derive(Debug)]
struct PickerState {
    choices: Vec<SessionChoice>,
    visible: Vec<usize>,
    selected: BTreeSet<usize>,
    query: String,
    cursor: usize,
    offset: usize,
}

impl PickerState {
    fn new(choices: Vec<SessionChoice>) -> Self {
        let visible = (0..choices.len()).collect();
        Self {
            choices,
            visible,
            selected: BTreeSet::new(),
            query: String::new(),
            cursor: 0,
            offset: 0,
        }
    }

    fn handle_key(&mut self, key: KeyEvent, rows: usize) -> Option<PickerExit> {
        if key.kind == KeyEventKind::Release {
            return None;
        }
        match key.code {
            KeyCode::Esc => return Some(PickerExit::Cancel),
            KeyCode::Enter => return Some(PickerExit::Submit),
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return Some(PickerExit::Cancel);
            }
            KeyCode::Up => self.move_up(1),
            KeyCode::Down => self.move_down(1),
            KeyCode::PageUp => self.move_up(rows.max(1)),
            KeyCode::PageDown => self.move_down(rows.max(1)),
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = self.visible.len().saturating_sub(1),
            KeyCode::Char(' ') => self.toggle_current(),
            KeyCode::Right => self.selected.extend(self.visible.iter().copied()),
            KeyCode::Left => self.selected.clear(),
            KeyCode::Backspace => {
                self.query.pop();
                self.refilter();
            }
            KeyCode::Char(character)
                if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                self.query.push(character);
                self.refilter();
            }
            _ => {}
        }
        self.ensure_visible(rows);
        None
    }

    fn move_up(&mut self, amount: usize) {
        if self.visible.is_empty() {
            return;
        }
        self.cursor = if amount == 1 && self.cursor == 0 {
            self.visible.len() - 1
        } else {
            self.cursor.saturating_sub(amount)
        };
    }

    fn move_down(&mut self, amount: usize) {
        if self.visible.is_empty() {
            return;
        }
        let last = self.visible.len() - 1;
        self.cursor = if amount == 1 && self.cursor == last {
            0
        } else {
            self.cursor.saturating_add(amount).min(last)
        };
    }

    fn toggle_current(&mut self) {
        let Some(choice) = self.visible.get(self.cursor).copied() else {
            return;
        };
        if !self.selected.remove(&choice) {
            self.selected.insert(choice);
        }
    }

    fn refilter(&mut self) {
        let mut scored = self
            .choices
            .iter()
            .enumerate()
            .filter_map(|(index, choice)| {
                choice
                    .row
                    .fuzzy_score(&self.query)
                    .map(|score| (index, score))
            })
            .collect::<Vec<_>>();
        scored.sort_by(|(left_index, left_score), (right_index, right_score)| {
            right_score
                .cmp(left_score)
                .then_with(|| left_index.cmp(right_index))
        });
        self.visible = scored.into_iter().map(|(index, _)| index).collect();
        self.cursor = 0;
        self.offset = 0;
    }

    fn ensure_visible(&mut self, rows: usize) {
        if rows == 0 || self.visible.is_empty() {
            self.offset = 0;
            return;
        }
        if self.cursor < self.offset {
            self.offset = self.cursor;
        } else if self.cursor >= self.offset + rows {
            self.offset = self.cursor + 1 - rows;
        }
        self.offset = self.offset.min(self.visible.len().saturating_sub(rows));
    }

    fn window(&self, rows: usize) -> impl Iterator<Item = (usize, usize, &SessionChoice)> {
        let end = (self.offset + rows).min(self.visible.len());
        self.visible[self.offset..end]
            .iter()
            .enumerate()
            .map(move |(relative, &choice)| (self.offset + relative, choice, &self.choices[choice]))
    }

    fn take_selected(self) -> Vec<SessionChoice> {
        self.choices
            .into_iter()
            .enumerate()
            .filter_map(|(index, choice)| self.selected.contains(&index).then_some(choice))
            .collect()
    }
}

struct RawMode;

impl RawMode {
    fn enter() -> Result<Self> {
        terminal::enable_raw_mode().context("enabling raw terminal mode")?;
        Ok(Self)
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        let _ = execute!(io::stdout(), Show);
    }
}

pub(super) fn select(
    choices: Vec<SessionChoice>,
    header: String,
    terminal_height: u16,
) -> Result<Option<Vec<SessionChoice>>> {
    let height = viewport_height(terminal_height);
    let _raw_mode = RawMode::enter()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Inline(height),
        },
    )
    .context("creating inline session picker")?;
    terminal.hide_cursor()?;

    let mut state = PickerState::new(choices);
    let mut last_area = Rect::new(0, 0, 0, height);
    let interaction = interaction_loop(&mut terminal, &mut state, &header, &mut last_area);
    let cleanup = cleanup_terminal(&mut terminal, last_area);
    drop(terminal);
    cleanup?;

    match interaction? {
        PickerExit::Submit => Ok(Some(state.take_selected())),
        PickerExit::Cancel => Ok(None),
    }
}

fn interaction_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &mut PickerState,
    header: &str,
    last_area: &mut Rect,
) -> Result<PickerExit> {
    loop {
        let rows = option_rows(last_area.height);
        state.ensure_visible(rows);
        *last_area = terminal
            .draw(|frame| render::draw(frame, state, header))
            .context("drawing session picker")?
            .area;
        let Event::Key(key) = event::read().context("reading session picker input")? else {
            continue;
        };
        if let Some(exit) = state.handle_key(key, option_rows(last_area.height)) {
            return Ok(exit);
        }
    }
}

fn cleanup_terminal(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    area: Rect,
) -> Result<()> {
    let clear = (area.width > 0).then(|| terminal.clear()).transpose();
    let position = terminal.set_cursor_position((0, area.y));
    let cursor = terminal.show_cursor();
    clear.context("clearing session picker")?;
    position.context("restoring terminal cursor position")?;
    cursor.context("showing terminal cursor")?;
    Ok(())
}

fn option_rows(viewport_height: u16) -> usize {
    usize::from(viewport_height.saturating_sub(CHROME_ROWS))
}

fn viewport_height(terminal_height: u16) -> u16 {
    let half = terminal_height / 2;
    if half > CHROME_ROWS {
        half
    } else {
        terminal_height.max(1)
    }
}
