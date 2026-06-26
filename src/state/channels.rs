use super::Store;
use anyhow::Result;
use rusqlite::{params, OptionalExtension};

impl Store {
    /// Returns true if `project` is a known root (non-subgroup) project.
    pub fn is_root_project(&self, project: &str) -> bool {
        self.conn
            .query_row(
                "SELECT 1 FROM project_meta WHERE project=?1 AND (parent='' OR parent IS NULL) \
                 UNION \
                 SELECT 1 FROM owned_groups WHERE project=?1 AND is_session_room=0",
                params![project],
                |_| Ok(()),
            )
            .is_ok()
    }

    /// Returns `true` if `project` is known locally — either it has a
    /// `project_meta` row (seen on relay) or an `owned_groups` row (we created it).
    pub fn channel_exists(&self, project: &str) -> bool {
        self.conn
            .query_row(
                "SELECT 1 FROM project_meta WHERE project = ?1 \
                 UNION SELECT 1 FROM owned_groups WHERE project = ?1",
                params![project],
                |_| Ok(()),
            )
            .is_ok()
    }

    /// Mark a group as a per-session room (issue #6). Idempotent. This records
    /// local routing metadata but does not by itself claim relay ownership; call
    /// `mark_group_owned` separately when management credentials exist.
    pub fn mark_session_room(&self, project: &str, parent: &str, ts: u64) -> Result<()> {
        self.conn.execute(
            "INSERT INTO owned_groups (project, created_at, owns_group, is_session_room, room_parent)
             VALUES (?1, ?2, 0, 1, ?3)
             ON CONFLICT(project) DO UPDATE SET is_session_room=1, room_parent=?3",
            params![project, ts, parent],
        )?;
        Ok(())
    }

    /// The work-root project a per-session room is nested under (set at mint).
    /// `None` if `project` is not a known per-session room. Materializer-safe.
    pub fn session_room_parent(&self, project: &str) -> Result<Option<String>> {
        let parent: rusqlite::Result<String> = self.conn.query_row(
            "SELECT room_parent FROM owned_groups WHERE project=?1 AND is_session_room=1",
            params![project],
            |r| r.get::<_, String>(0),
        );
        match parent {
            Ok(p) if !p.is_empty() => Ok(Some(p)),
            Ok(_) => Ok(None),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Per-session rooms directly nested under a work-root project.
    pub fn session_rooms_under(&self, parent: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT project FROM owned_groups
             WHERE is_session_room=1 AND room_parent=?1
             ORDER BY project",
        )?;
        let rows = stmt.query_map(params![parent], |r| r.get::<_, String>(0))?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// Top-level project that owns `scope`. `scope` may already be a root
    /// project, a per-session room, or a NIP-29 subgroup/task channel.
    pub fn work_root_for_scope(&self, scope: &str) -> Result<String> {
        let mut current = scope.to_string();
        let mut seen = std::collections::BTreeSet::new();
        for _ in 0..64 {
            if !seen.insert(current.clone()) {
                break;
            }
            let parent = match self.session_room_parent(&current)? {
                Some(parent) => Some(parent),
                None => self.group_parent(&current)?,
            };
            match parent {
                Some(parent) if !parent.is_empty() => current = parent,
                _ => break,
            }
        }
        Ok(current)
    }

    /// The display label for a channel: its kind:39000 `name`, falling back to
    /// the raw id when unknown/empty.
    fn channel_label(&self, id: &str) -> String {
        let name: rusqlite::Result<String> = self.conn.query_row(
            "SELECT name FROM project_meta WHERE project=?1",
            params![id],
            |r| r.get::<_, String>(0),
        );
        match name {
            Ok(n) if !n.is_empty() => n,
            _ => id.to_string(),
        }
    }

    /// The known human title for a channel from NIP-29 metadata. Unlike
    /// `group_display_name`, this intentionally does not fall back to `about`;
    /// hook awareness should not manufacture titles from descriptions.
    pub fn channel_title(&self, id: &str) -> Result<Option<String>> {
        let title = self
            .conn
            .query_row(
                "SELECT name FROM project_meta WHERE project=?1",
                params![id],
                |r| r.get::<_, String>(0),
            )
            .optional()?
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        Ok(title)
    }

    /// Latest non-empty local or peer work title for a channel. This is the
    /// fallback for per-session rooms whose display name is the distilled
    /// session title rather than a relay-authored group name.
    pub fn latest_channel_work_title(&self, id: &str) -> Result<Option<String>> {
        self.conn
            .query_row(
                "SELECT title FROM (
                   SELECT title, updated_at FROM session_state
                   WHERE project=?1 AND title <> ''
                   UNION ALL
                   SELECT title, updated_at FROM presence_state
                   WHERE project=?1 AND title <> ''
                 )
                 ORDER BY updated_at DESC LIMIT 1",
                params![id],
                |r| r.get::<_, String>(0),
            )
            .optional()
            .map_err(Into::into)
    }

    /// The channel breadcrumb from the top-level project down to `leaf`, as
    /// `(id, label)` pairs in root→leaf order. Walks the `parent` chain
    /// (`group_parent`); a top-level project returns a single element. Bounded
    /// against cycles/corrupt data.
    pub fn channel_breadcrumb(&self, leaf: &str) -> Result<Vec<(String, String)>> {
        let mut chain: Vec<(String, String)> = Vec::new();
        let mut cur = leaf.to_string();
        for _ in 0..64 {
            chain.push((cur.clone(), self.channel_label(&cur)));
            match self.group_parent(&cur)? {
                Some(p) => cur = p,
                None => break,
            }
        }
        chain.reverse();
        Ok(chain)
    }

    /// Descendant channels of `root` (excluding `root` itself) in preorder, with
    /// `depth` relative to `root` (direct children = depth 1). Drives the
    /// "Subchannels:" block. Source: the `project_meta` parent→children tree.
    pub fn subchannels_of(&self, root: &str) -> Result<Vec<(String, String, usize)>> {
        // Build the parent→children adjacency from the metadata table once.
        let meta = self.list_group_metadata()?; // (id, about, name, parent)
        let mut children: std::collections::BTreeMap<String, Vec<(String, String)>> =
            std::collections::BTreeMap::new();
        for (id, _about, name, parent) in &meta {
            if !parent.is_empty() {
                let label = if name.is_empty() {
                    id.clone()
                } else {
                    name.clone()
                };
                children
                    .entry(parent.clone())
                    .or_default()
                    .push((id.clone(), label));
            }
        }
        // Deterministic ordering: sort each sibling list by channel id so the
        // rendered subchannel tree is stable across runs (metadata row order is
        // otherwise arbitrary).
        for kids in children.values_mut() {
            kids.sort_by(|a, b| a.0.cmp(&b.0));
        }
        let mut out: Vec<(String, String, usize)> = Vec::new();
        // Preorder DFS, bounded depth to guard against cycles.
        fn walk(
            children: &std::collections::BTreeMap<String, Vec<(String, String)>>,
            node: &str,
            depth: usize,
            out: &mut Vec<(String, String, usize)>,
        ) {
            if depth > 32 {
                return;
            }
            if let Some(kids) = children.get(node) {
                for (id, label) in kids {
                    out.push((id.clone(), label.clone(), depth));
                    walk(children, id, depth + 1, out);
                }
            }
        }
        walk(&children, root, 1, &mut out);
        Ok(out)
    }

    /// Count distinct channels with any session activity (local or peer) at or
    /// after `cutoff`, excluding the channels in `exclude` (typically the current
    /// channel + its rendered subtree). Drives the "N other active channels in
    /// the past 24h" tail.
    pub fn count_active_channels_since(&self, cutoff: u64, exclude: &[String]) -> Result<u64> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT project FROM (
               SELECT project FROM session_state      WHERE last_seen>=?1
               UNION
               SELECT project FROM presence_state      WHERE last_seen>=?1
             )",
        )?;
        let excl: std::collections::HashSet<&str> = exclude.iter().map(|s| s.as_str()).collect();
        let rows = stmt.query_map(params![cutoff], |r| r.get::<_, String>(0))?;
        let mut n: u64 = 0;
        for row in rows {
            let project = row?;
            if !excl.contains(project.as_str()) {
                n += 1;
            }
        }
        Ok(n)
    }

    /// Channels with semantic activity at or after `cutoff`, newest first.
    /// Heartbeat-only presence is intentionally ignored.
    pub fn semantic_active_channels_since(
        &self,
        cutoff: u64,
        exclude: &[String],
    ) -> Result<Vec<(String, u64)>> {
        let mut stmt = self.conn.prepare(
            "SELECT project, MAX(latest) AS latest FROM (
               SELECT project, MAX(created_at) AS latest
               FROM chat_messages
               WHERE created_at>=?1
               GROUP BY project
               UNION ALL
               SELECT project, MAX(updated_at) AS latest
               FROM session_state
               WHERE updated_at>=?1 AND (title<>'' OR activity<>'')
               GROUP BY project
               UNION ALL
               SELECT project, MAX(updated_at) AS latest
               FROM presence_state
               WHERE updated_at>=?1 AND (title<>'' OR activity<>'')
               GROUP BY project
               UNION ALL
               SELECT project, MAX(updated_at) AS latest
               FROM project_meta
               WHERE updated_at>=?1 AND name<>''
               GROUP BY project
             )
             GROUP BY project
             ORDER BY latest DESC, project ASC",
        )?;
        let excl: std::collections::HashSet<&str> = exclude.iter().map(|s| s.as_str()).collect();
        let rows = stmt.query_map(params![cutoff], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, u64>(1)?))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (project, latest) = row?;
            if !excl.contains(project.as_str()) {
                out.push((project, latest));
            }
        }
        Ok(out)
    }

    /// True if `project` is a per-session room (minted at session birth), as
    /// opposed to a project group or a task room.
    pub fn is_session_room(&self, project: &str) -> Result<bool> {
        let n: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM owned_groups WHERE project=?1 AND is_session_room=1",
            params![project],
            |r| r.get(0),
        )?;
        Ok(n > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_room_flag_roundtrip() {
        let s = Store::open_memory().unwrap();
        // A plain owned group is not a session room.
        s.mark_group_owned("proj", 100).unwrap();
        assert!(!s.is_session_room("proj").unwrap());
        // Marking a room sets the flag + records its work-root parent. It is
        // known locally, but is not owned until relay-management code claims it.
        s.mark_session_room("proj-room", "proj", 100).unwrap();
        assert!(s.is_session_room("proj-room").unwrap());
        assert!(!s.is_group_owned("proj-room").unwrap());
        assert!(s.channel_exists("proj-room"));
        s.mark_group_owned("proj-room", 100).unwrap();
        assert!(s.is_group_owned("proj-room").unwrap());
        assert_eq!(
            s.session_room_parent("proj-room").unwrap().as_deref(),
            Some("proj")
        );
        // Idempotent (parent preserved).
        s.mark_session_room("proj-room", "proj", 200).unwrap();
        assert!(s.is_session_room("proj-room").unwrap());
        // Unknown group is not a session room and has no parent.
        assert!(!s.is_session_room("nope").unwrap());
        assert_eq!(s.session_room_parent("nope").unwrap(), None);
    }

    #[test]
    fn work_root_for_scope_walks_owned_rooms_and_group_parents() {
        let s = Store::open_memory().unwrap();
        s.upsert_group_metadata("proj", "Proj", "", 100).unwrap();
        s.upsert_group_metadata("task-room", "Task", "proj", 100)
            .unwrap();
        s.mark_session_room("session-room", "task-room", 100)
            .unwrap();

        assert_eq!(s.work_root_for_scope("proj").unwrap(), "proj");
        assert_eq!(s.work_root_for_scope("task-room").unwrap(), "proj");
        assert_eq!(s.work_root_for_scope("session-room").unwrap(), "proj");
    }

    // ── channel hierarchy helpers (channel-context block) ────────────────

    fn seed_channel_tree(s: &Store) {
        // proj
        //  ├─ research        (#research)
        //  │   └─ comparison  (#competitive-comparison)
        //  └─ planning        (#planning)
        s.upsert_group_metadata("proj", "myproject", "", 1).unwrap();
        s.upsert_group_metadata("research", "research", "proj", 1)
            .unwrap();
        s.upsert_group_metadata("comparison", "competitive-comparison", "research", 1)
            .unwrap();
        s.upsert_group_metadata("planning", "planning", "proj", 1)
            .unwrap();
    }

    #[test]
    fn channel_breadcrumb_walks_parent_chain_to_root() {
        let s = Store::open_memory().unwrap();
        seed_channel_tree(&s);
        // Leaf → root→leaf order with display labels.
        assert_eq!(
            s.channel_breadcrumb("comparison").unwrap(),
            vec![
                ("proj".into(), "myproject".into()),
                ("research".into(), "research".into()),
                ("comparison".into(), "competitive-comparison".into()),
            ]
        );
        // A top-level project is a single crumb.
        assert_eq!(
            s.channel_breadcrumb("proj").unwrap(),
            vec![("proj".into(), "myproject".into())]
        );
        // Unknown id falls back to itself.
        assert_eq!(
            s.channel_breadcrumb("ghost").unwrap(),
            vec![("ghost".into(), "ghost".into())]
        );
    }

    #[test]
    fn subchannels_of_returns_preorder_descendants_with_depth() {
        let s = Store::open_memory().unwrap();
        seed_channel_tree(&s);
        // From the root: direct children at depth 1, the grandchild at depth 2,
        // preorder (BTreeMap orders children by id: comparison's parent is
        // research; planning and research are children of proj, ordered by id).
        let subs = s.subchannels_of("proj").unwrap();
        assert_eq!(
            subs,
            vec![
                ("planning".into(), "planning".into(), 1),
                ("research".into(), "research".into(), 1),
                ("comparison".into(), "competitive-comparison".into(), 2),
            ]
        );
        // A leaf channel has no subchannels.
        assert!(s.subchannels_of("comparison").unwrap().is_empty());
    }
}
