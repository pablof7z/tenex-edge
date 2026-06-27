use super::*;

impl Store {
    pub fn enqueue_chat(&self, row: &ChatInboxRow) -> Result<bool> {
        let changed = self.conn.execute(
            "INSERT INTO chat_inbox
               (chat_event_id, target_session, from_pubkey, from_slug, project, body, created_at, delivered, from_session, mentioned_session)
             VALUES (?1,?2,?3,?4,?5,?6,?7,0,?8,?9)
             ON CONFLICT(chat_event_id, target_session) DO UPDATE SET
               from_session=CASE
                 WHEN chat_inbox.from_session='' THEN excluded.from_session
                 ELSE chat_inbox.from_session
               END,
               mentioned_session=CASE
                 WHEN chat_inbox.mentioned_session='' THEN excluded.mentioned_session
                 ELSE chat_inbox.mentioned_session
               END",
            params![
                row.chat_event_id,
                row.target_session,
                row.from_pubkey,
                row.from_slug,
                row.project,
                row.body,
                row.created_at,
                row.from_session,
                row.mentioned_session,
            ],
        )?;
        Ok(changed > 0)
    }

    /// Idempotently record a local chat history row. This is separate from
    /// `chat_inbox`: the log powers explicit reads, while the inbox remains the
    /// live-only hook injection queue.
    pub fn record_chat(&self, row: &ChatLogRow) -> Result<bool> {
        let changed = self.conn.execute(
            "INSERT INTO chat_messages
               (chat_event_id, from_pubkey, from_slug, host, project, body, created_at, from_session, mentioned_session)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)
             ON CONFLICT(chat_event_id) DO UPDATE SET
               from_session=CASE
                 WHEN chat_messages.from_session='' THEN excluded.from_session
                 ELSE chat_messages.from_session
               END,
               mentioned_session=CASE
                 WHEN chat_messages.mentioned_session='' THEN excluded.mentioned_session
                 ELSE chat_messages.mentioned_session
               END",
            params![
                row.chat_event_id,
                row.from_pubkey,
                row.from_slug,
                row.host,
                row.project,
                row.body,
                row.created_at,
                row.from_session,
                row.mentioned_session,
            ],
        )?;
        self.conn.execute(
            "UPDATE chat_inbox
             SET
               from_session=CASE
                 WHEN from_session='' THEN ?2
                 ELSE from_session
               END,
               mentioned_session=CASE
                 WHEN mentioned_session='' THEN ?3
                 ELSE mentioned_session
               END
             WHERE chat_event_id=?1",
            params![row.chat_event_id, row.from_session, row.mentioned_session,],
        )?;
        Ok(changed > 0)
    }

    /// The originating session recorded locally for a chat event, if any.
    ///
    /// User prompts are signed by the *operator* key, which maps to no session,
    /// so the relay echo can't recover the origin from the signer pubkey. But
    /// `publish_chat_checked`/`rpc_user_prompt` write the origin session into
    /// `chat_messages` synchronously *before* the wire send, so the local row
    /// always precedes the echo. The materializer reads it back here to suppress
    /// self-delivery to the session that produced the prompt.
    pub fn chat_origin_session(&self, chat_event_id: &str) -> Option<String> {
        self.conn
            .query_row(
                "SELECT from_session FROM chat_messages WHERE chat_event_id=?1",
                params![chat_event_id],
                |r| r.get::<_, String>(0),
            )
            .ok()
            .filter(|s| !s.is_empty())
    }

    pub fn list_chat_messages(
        &self,
        project: &str,
        since: u64,
        limit: Option<u64>,
        offset: u64,
        tail: bool,
    ) -> Result<Vec<ChatLogRow>> {
        let limit = limit.unwrap_or(i64::MAX as u64).min(i64::MAX as u64) as i64;
        let offset = offset.min(i64::MAX as u64) as i64;
        let order = if tail {
            "created_at DESC, chat_event_id DESC"
        } else {
            "created_at ASC, chat_event_id ASC"
        };
        let sql = format!(
            "SELECT chat_event_id, from_pubkey, from_slug, host, project, body, created_at, from_session, mentioned_session
             FROM chat_messages
             WHERE project=?1 AND created_at>=?2
             ORDER BY {order}
             LIMIT ?3 OFFSET ?4"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let mut rows: Vec<ChatLogRow> = stmt
            .query_map(params![project, since, limit, offset], row_to_chat_log)?
            .filter_map(|r| r.ok())
            .collect();
        if tail {
            rows.reverse();
        }
        Ok(rows)
    }

    /// Recent project chat lines for tail backfill, newest first.
    /// `project = None` spans all projects. Each row is `(created_at, body,
    /// from_pubkey, project, from_session)` — enough to render a `Msg` event
    /// without a relay round-trip.
    #[allow(clippy::type_complexity)]
    pub fn recent_chat_for_backfill(
        &self,
        project: Option<&str>,
        since: u64,
        limit: u64,
    ) -> Result<Vec<(u64, String, String, String, Option<String>)>> {
        let mut stmt = self.conn.prepare(
            "SELECT created_at, body, from_pubkey, project, from_session
             FROM chat_messages
             WHERE (?1 IS NULL OR project=?1) AND created_at >= ?2
             ORDER BY created_at DESC LIMIT ?3",
        )?;
        let rows = stmt
            .query_map(params![project, since, limit as i64], |r| {
                Ok((
                    r.get::<_, u64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, Option<String>>(4)?,
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(rows)
    }

    /// Read undelivered chat rows without marking them delivered. Used by
    /// mid-turn hook injection so the next turn-start remains authoritative.
    pub fn peek_chat(&self, session_id: &str) -> Result<Vec<ChatInboxRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT chat_event_id, target_session, from_pubkey, from_slug, project, body, created_at, from_session, mentioned_session
             FROM chat_inbox WHERE target_session=?1 AND delivered=0 ORDER BY created_at",
        )?;
        let rows: Vec<ChatInboxRow> = stmt
            .query_map(params![session_id], row_to_chat)?
            .filter_map(|r| r.ok())
            .collect();
        Ok(rows)
    }

    /// Read undelivered chat rows that explicitly mention this session.
    pub fn peek_chat_mentions(&self, session_id: &str) -> Result<Vec<ChatInboxRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT chat_event_id, target_session, from_pubkey, from_slug, project, body, created_at, from_session, mentioned_session
             FROM chat_inbox
             WHERE target_session=?1 AND mentioned_session=?1 AND delivered=0
             ORDER BY created_at",
        )?;
        let rows: Vec<ChatInboxRow> = stmt
            .query_map(params![session_id], row_to_chat)?
            .filter_map(|r| r.ok())
            .collect();
        Ok(rows)
    }

    /// Read explicit mention rows not yet surfaced by hook fallback. They stay
    /// undelivered so tmux / turn-start can still submit the actual prompt.
    pub fn peek_unnotified_chat_mentions(&self, session_id: &str) -> Result<Vec<ChatInboxRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT chat_event_id, target_session, from_pubkey, from_slug, project, body, created_at, from_session, mentioned_session
             FROM chat_inbox
             WHERE target_session=?1 AND mentioned_session=?1 AND delivered=0 AND notified_at=0
             ORDER BY created_at",
        )?;
        let rows: Vec<ChatInboxRow> = stmt
            .query_map(params![session_id], row_to_chat)?
            .filter_map(|r| r.ok())
            .collect();
        Ok(rows)
    }

    /// Return undelivered chat rows for a session and mark them delivered.
    pub fn drain_chat(&self, session_id: &str) -> Result<Vec<ChatInboxRow>> {
        let rows = self.peek_chat(session_id)?;
        self.conn.execute(
            "UPDATE chat_inbox SET delivered=1, delivered_at=?2 WHERE target_session=?1 AND delivered=0",
            params![session_id, crate::util::now_secs()],
        )?;
        Ok(rows)
    }

    /// Mark exactly these chat rows delivered for `session_id`.
    pub fn mark_chat_rows_delivered(
        &self,
        session_id: &str,
        event_ids: &[String],
        delivered_at: u64,
    ) -> Result<()> {
        let mut stmt = self.conn.prepare(
            "UPDATE chat_inbox SET delivered=1, delivered_at=?3
             WHERE target_session=?1 AND chat_event_id=?2 AND delivered=0",
        )?;
        for event_id in event_ids {
            stmt.execute(params![session_id, event_id, delivered_at])?;
        }
        Ok(())
    }

    /// Mark direct mention rows as surfaced by a non-consuming hook fallback.
    pub fn mark_chat_rows_notified(
        &self,
        session_id: &str,
        event_ids: &[String],
        notified_at: u64,
    ) -> Result<()> {
        let mut stmt = self.conn.prepare(
            "UPDATE chat_inbox SET notified_at=?3
             WHERE target_session=?1 AND chat_event_id=?2 AND delivered=0 AND notified_at=0",
        )?;
        for event_id in event_ids {
            stmt.execute(params![session_id, event_id, notified_at])?;
        }
        Ok(())
    }
}
