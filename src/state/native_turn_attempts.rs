//! Durable outcomes for daemon-owned native RPC turns.
//!
//! Presence remains lifecycle-only. This ledger records one exact delivery
//! attempt and its native terminal evidence for operator diagnostics.

use super::*;

const MAX_DIAGNOSTIC_CHARS: usize = 500;
const COLS: &str = "id, pubkey, runtime_generation, delivery_kind, delivery_event_id, \
    native_thread_id, native_turn_id, outcome, error_message, error_details, \
    started_at, finished_at";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeTurnDeliveryKind {
    InboxEvent,
    SpawnPrompt,
}

impl NativeTurnDeliveryKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InboxEvent => "inbox_event",
            Self::SpawnPrompt => "spawn_prompt",
        }
    }

    fn parse(value: &str) -> Result<Self> {
        match value {
            "inbox_event" => Ok(Self::InboxEvent),
            "spawn_prompt" => Ok(Self::SpawnPrompt),
            _ => anyhow::bail!("unknown NativeTurnDeliveryKind value {value:?}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeTurnOutcome {
    Started,
    Completed,
    Failed,
    Interrupted,
    RejectedBeforeStart,
    ChildExited,
    UnknownReconciled,
}

impl NativeTurnOutcome {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Started => "started",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Interrupted => "interrupted",
            Self::RejectedBeforeStart => "rejected_before_start",
            Self::ChildExited => "child_exited",
            Self::UnknownReconciled => "unknown_reconciled",
        }
    }

    fn parse(value: &str) -> Result<Self> {
        match value {
            "started" => Ok(Self::Started),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            "interrupted" => Ok(Self::Interrupted),
            "rejected_before_start" => Ok(Self::RejectedBeforeStart),
            "child_exited" => Ok(Self::ChildExited),
            "unknown_reconciled" => Ok(Self::UnknownReconciled),
            _ => anyhow::bail!("unknown NativeTurnOutcome value {value:?}"),
        }
    }

    pub fn is_failure(self) -> bool {
        !matches!(self, Self::Started | Self::Completed)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeTurnAttempt {
    pub id: i64,
    pub pubkey: String,
    pub runtime_generation: u64,
    pub delivery_kind: NativeTurnDeliveryKind,
    pub delivery_event_id: String,
    pub native_thread_id: String,
    pub native_turn_id: String,
    pub outcome: NativeTurnOutcome,
    pub error_message: String,
    pub error_details: String,
    pub started_at: u64,
    pub finished_at: u64,
}

#[derive(Debug)]
pub struct NewNativeTurnAttempt<'a> {
    pub pubkey: &'a str,
    pub runtime_generation: u64,
    pub delivery_kind: NativeTurnDeliveryKind,
    pub delivery_event_id: &'a str,
    pub native_thread_id: &'a str,
    pub started_at: u64,
}

#[derive(Debug)]
pub struct FinishNativeTurnAttempt<'a> {
    pub id: i64,
    pub pubkey: &'a str,
    pub runtime_generation: u64,
    pub native_turn_id: &'a str,
    pub outcome: NativeTurnOutcome,
    pub error_message: &'a str,
    pub error_details: &'a str,
    pub finished_at: u64,
}

fn row_to_attempt(row: &rusqlite::Row) -> rusqlite::Result<NativeTurnAttempt> {
    let delivery_kind = row.get::<_, String>(3)?;
    let outcome = row.get::<_, String>(7)?;
    Ok(NativeTurnAttempt {
        id: row.get(0)?,
        pubkey: row.get(1)?,
        runtime_generation: row.get(2)?,
        delivery_kind: NativeTurnDeliveryKind::parse(&delivery_kind).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Text, error.into())
        })?,
        delivery_event_id: row.get(4)?,
        native_thread_id: row.get(5)?,
        native_turn_id: row.get(6)?,
        outcome: NativeTurnOutcome::parse(&outcome).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(7, rusqlite::types::Type::Text, error.into())
        })?,
        error_message: row.get(8)?,
        error_details: row.get(9)?,
        started_at: row.get(10)?,
        finished_at: row.get(11)?,
    })
}

fn diagnostic(value: &str) -> String {
    crate::secret_scrub::scrub(value)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(MAX_DIAGNOSTIC_CHARS)
        .collect()
}

impl Store {
    pub fn start_native_turn_attempt(&self, row: &NewNativeTurnAttempt<'_>) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO native_turn_attempts
                (pubkey, runtime_generation, delivery_kind, delivery_event_id,
                 native_thread_id, outcome, started_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 'started', ?6)",
            params![
                row.pubkey,
                row.runtime_generation,
                row.delivery_kind.as_str(),
                row.delivery_event_id,
                row.native_thread_id,
                row.started_at,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn finish_native_turn_attempt(&self, row: &FinishNativeTurnAttempt<'_>) -> Result<bool> {
        if row.outcome == NativeTurnOutcome::Started {
            anyhow::bail!("a native turn attempt cannot finish as started");
        }
        let changed = self.conn.execute(
            "UPDATE native_turn_attempts
                SET native_turn_id=?1, outcome=?2, error_message=?3,
                    error_details=?4, finished_at=?5
              WHERE id=?6 AND pubkey=?7 AND runtime_generation=?8
                AND outcome='started' AND finished_at=0",
            params![
                row.native_turn_id,
                row.outcome.as_str(),
                diagnostic(row.error_message),
                diagnostic(row.error_details),
                row.finished_at,
                row.id,
                row.pubkey,
                row.runtime_generation,
            ],
        )?;
        Ok(changed == 1)
    }

    pub fn latest_native_turn_attempt(
        &self,
        pubkey: &str,
        runtime_generation: u64,
    ) -> Result<Option<NativeTurnAttempt>> {
        Ok(self
            .conn
            .query_row(
                &format!(
                    "SELECT {COLS} FROM native_turn_attempts
                     WHERE pubkey=?1 AND runtime_generation=?2 AND outcome!='started'
                     ORDER BY id DESC LIMIT 1"
                ),
                params![pubkey, runtime_generation],
                row_to_attempt,
            )
            .optional()?)
    }

    pub fn reconcile_open_native_turn_attempts(&self, now: u64) -> Result<usize> {
        Ok(self.conn.execute(
            "UPDATE native_turn_attempts
                SET outcome='unknown_reconciled',
                    error_message='daemon restarted before native terminal evidence',
                    finished_at=?1
              WHERE outcome='started' AND finished_at=0",
            params![now],
        )?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_generation_outcome_is_bounded_redacted_and_at_most_once() {
        let store = Store::open_memory().unwrap();
        let id = store
            .start_native_turn_attempt(&NewNativeTurnAttempt {
                pubkey: "pk",
                runtime_generation: 7,
                delivery_kind: NativeTurnDeliveryKind::InboxEvent,
                delivery_event_id: "event",
                native_thread_id: "thread",
                started_at: 10,
            })
            .unwrap();
        let token = "sk-abcdefghijklmnopqrstuvwxyz123456";
        let message = format!("rejected {token} {}", "x".repeat(600));
        let finish = FinishNativeTurnAttempt {
            id,
            pubkey: "pk",
            runtime_generation: 7,
            native_turn_id: "turn",
            outcome: NativeTurnOutcome::Failed,
            error_message: &message,
            error_details: "",
            finished_at: 20,
        };
        assert!(store.finish_native_turn_attempt(&finish).unwrap());
        assert!(!store.finish_native_turn_attempt(&finish).unwrap());

        let row = store.latest_native_turn_attempt("pk", 7).unwrap().unwrap();
        assert_eq!(row.outcome, NativeTurnOutcome::Failed);
        assert!(!row.error_message.contains(token));
        assert!(row.error_message.chars().count() <= MAX_DIAGNOSTIC_CHARS);
        assert!(store.latest_native_turn_attempt("pk", 8).unwrap().is_none());
    }

    #[test]
    fn restart_reconciles_only_unfinished_attempts() {
        let store = Store::open_memory().unwrap();
        store
            .start_native_turn_attempt(&NewNativeTurnAttempt {
                pubkey: "pk",
                runtime_generation: 1,
                delivery_kind: NativeTurnDeliveryKind::SpawnPrompt,
                delivery_event_id: "",
                native_thread_id: "thread",
                started_at: 1,
            })
            .unwrap();
        assert_eq!(store.reconcile_open_native_turn_attempts(9).unwrap(), 1);
        let row = store.latest_native_turn_attempt("pk", 1).unwrap().unwrap();
        assert_eq!(row.outcome, NativeTurnOutcome::UnknownReconciled);
        assert_eq!(row.finished_at, 9);
    }
}
