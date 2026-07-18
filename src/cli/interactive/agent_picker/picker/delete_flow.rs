use super::{DeleteScope, PickerAction, PickerState};
use crossterm::event::{KeyCode, KeyEvent};

/// Delete confirmation state, shown inline in the footer rather than an
/// overlay dialog. `Nothing` and `ChooseScope` are dismissed by any key that
/// doesn't advance them; `Confirm` requires `y`/`d` to proceed, `esc` cancels.
#[derive(Debug)]
pub(super) enum PendingDelete {
    Nothing { index: usize },
    ChooseScope { index: usize },
    Confirm { plan: Vec<(usize, DeleteScope)> },
}

impl PickerState {
    /// With no multi-selection, mirrors a single row's exact deletable
    /// targets (asking which to delete when both a configured entry and a
    /// native profile exist). With a multi-selection, every selected row
    /// that has anything to delete is queued with `DeleteScope::Both` —
    /// bulk delete doesn't offer per-row target picking.
    pub(super) fn begin_delete(&mut self) {
        if self.selected.is_empty() {
            let Some(index) = self.current() else {
                return;
            };
            let row = &self.rows[index];
            self.pending_delete = Some(match (row.has_configured, row.has_native_profile) {
                (false, false) => PendingDelete::Nothing { index },
                (true, false) => PendingDelete::Confirm {
                    plan: vec![(index, DeleteScope::Agent)],
                },
                (false, true) => PendingDelete::Confirm {
                    plan: vec![(index, DeleteScope::Profile)],
                },
                (true, true) => PendingDelete::ChooseScope { index },
            });
            return;
        }
        let plan = self
            .selected
            .iter()
            .filter(|&&index| {
                self.rows[index].has_configured || self.rows[index].has_native_profile
            })
            .map(|&index| (index, DeleteScope::Both))
            .collect::<Vec<_>>();
        self.pending_delete = Some(if plan.is_empty() {
            PendingDelete::Nothing {
                index: *self
                    .selected
                    .iter()
                    .next()
                    .expect("checked non-empty above"),
            }
        } else {
            PendingDelete::Confirm { plan }
        });
    }

    pub(super) fn handle_pending_delete(
        &mut self,
        pending: PendingDelete,
        key: KeyEvent,
    ) -> Option<PickerAction> {
        match pending {
            PendingDelete::Nothing { .. } => {}
            PendingDelete::ChooseScope { index } => match key.code {
                KeyCode::Char('a') => {
                    self.pending_delete = Some(PendingDelete::Confirm {
                        plan: vec![(index, DeleteScope::Agent)],
                    });
                }
                KeyCode::Char('p') => {
                    self.pending_delete = Some(PendingDelete::Confirm {
                        plan: vec![(index, DeleteScope::Profile)],
                    });
                }
                KeyCode::Char('b') => {
                    self.pending_delete = Some(PendingDelete::Confirm {
                        plan: vec![(index, DeleteScope::Both)],
                    });
                }
                KeyCode::Esc => {}
                _ => self.pending_delete = Some(PendingDelete::ChooseScope { index }),
            },
            PendingDelete::Confirm { plan } => match key.code {
                KeyCode::Char('y')
                | KeyCode::Char('Y')
                | KeyCode::Char('d')
                | KeyCode::Char('D') => {
                    self.selected.clear();
                    return Some(PickerAction::Delete(plan));
                }
                KeyCode::Esc => {}
                _ => self.pending_delete = Some(PendingDelete::Confirm { plan }),
            },
        }
        None
    }
}
