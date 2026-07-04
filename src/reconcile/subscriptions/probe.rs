//! Probe-facing, non-mutating causality queries over the live subscription graph
//! (frontier design §4.3 why + §4.4 state). Read-only: it reports the current
//! owner set + refcount of a channel's REQ and why its latest command was
//! emitted, resolved through the label registry — no relay I/O, no mutation.

use std::collections::BTreeSet;

use trellis_core::{ResourceCommandCause, ResourceKey, ScopeId};

use crate::reconcile::labels::key_path;

use super::keys::{sub_key, Space};
use super::SubscriptionReconciler;

/// One live subscription resource: its path, refcount, and owning scopes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubStateRow {
    pub resource_key: String,
    pub refcount: usize,
    pub owners: Vec<String>,
}

/// Plain, Trellis-free explanation of a channel `#h` REQ's live state + last change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelWhy {
    /// Human resource path, e.g. `sub/h/general`.
    pub resource_key: String,
    /// How many scopes currently own the REQ (the authoritative refcount).
    pub refcount: usize,
    /// The owning scopes, by debug name (`daemon-subs`, `session-<id>`).
    pub owners: Vec<String>,
    /// The latest command operation, or `None` if none was ever emitted.
    pub last_kind: Option<String>,
    /// What produced the latest command, or `None` if none was ever emitted.
    pub cause: Option<String>,
}

impl SubscriptionReconciler {
    /// The live graph revision.
    pub fn revision(&self) -> u64 {
        self.graph.revision().get()
    }

    /// Explain a channel's `#h` REQ: its live owner set + refcount, and why its
    /// latest command was emitted. Always returns a value (the refcount/owners are
    /// live even for a never-commanded key); `last_kind`/`cause` are `None` when no
    /// command has been emitted — reported honestly rather than faked.
    pub fn explain_channel(&self, h: &str) -> ChannelWhy {
        let key = sub_key(Space::ChannelH, h);
        let owners = self
            .graph
            .resource_owners(&key)
            .map(|scopes| scopes.iter().map(|s| self.scope_label(*s)).collect())
            .unwrap_or_default();
        let refcount = self.owner_count(&key);
        let (last_kind, cause) = match self.why_command(&key) {
            Some(why) => (
                Some(format!("{:?}", why.kind)),
                Some(self.cause_label(&why.cause)),
            ),
            None => (None, None),
        };
        ChannelWhy {
            resource_key: key_path(&key),
            refcount,
            owners,
            last_kind,
            cause,
        }
    }

    /// Every live subscription resource with its owner set + refcount, in stable
    /// key order. Live keys are the union of the daemon scope's and each session
    /// scope's resource inventories (a shared key is owned by several scopes).
    pub fn state_rows(&self) -> Vec<SubStateRow> {
        let mut keys: BTreeSet<ResourceKey> = BTreeSet::new();
        for scope in std::iter::once(self.daemon_scope).chain(self.session_scopes()) {
            if let Ok(inv) = self.graph.scope_resource_inventory(scope) {
                keys.extend(inv.resources);
            }
        }
        keys.into_iter()
            .map(|key| {
                let owners = self
                    .graph
                    .resource_owners(&key)
                    .map(|scopes| scopes.iter().map(|s| self.scope_label(*s)).collect())
                    .unwrap_or_default();
                SubStateRow {
                    resource_key: key_path(&key),
                    refcount: self.owner_count(&key),
                    owners,
                }
            })
            .collect()
    }

    /// The scope ids of every currently-alive session.
    fn session_scopes(&self) -> impl Iterator<Item = ScopeId> + '_ {
        self.sessions.values().map(|n| n.scope)
    }

    fn cause_label(&self, cause: &ResourceCommandCause) -> String {
        match cause {
            ResourceCommandCause::Planner { collection } => format!(
                "planner: {}",
                self.labels()
                    .label_of(*collection)
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("node:{}", collection.get()))
            ),
            ResourceCommandCause::ScopeClosed { scope } => {
                format!("scope-closed: {}", self.scope_label(*scope))
            }
        }
    }

    fn scope_label(&self, scope: ScopeId) -> String {
        self.graph
            .scope_meta(scope)
            .map(|m| m.debug_name().to_string())
            .unwrap_or_else(|| format!("scope:{}", scope.get()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reconcile::CoverageSnapshot;
    use std::collections::{BTreeMap, BTreeSet};

    fn set<const N: usize>(items: [&str; N]) -> BTreeSet<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    /// A channel owned by two scopes (daemon + a session) reports refcount 2 and
    /// its opening command, labeled.
    #[test]
    fn explain_channel_reports_owners_and_cause() {
        let mut r = SubscriptionReconciler::new().unwrap();
        let mut sessions = BTreeMap::new();
        sessions.insert("s1".to_string(), set(["general"]));
        let snap = CoverageSnapshot {
            daemon_channels: set(["general"]),
            addressed_pubkeys: BTreeSet::new(),
            archived_channels: BTreeSet::new(),
            sessions,
        };
        r.sync(&snap).unwrap();
        r.assert_oracle().unwrap();

        let why = r.explain_channel("general");
        assert_eq!(why.resource_key, "sub/h/general");
        assert_eq!(why.refcount, 2, "daemon scope + session scope both own it");
        assert_eq!(why.owners.len(), 2);
        assert_eq!(why.last_kind.as_deref(), Some("Open"));
        assert!(
            why.cause
                .as_deref()
                .unwrap_or_default()
                .starts_with("planner:"),
            "cause names the planner: {:?}",
            why.cause
        );
    }

    /// An uncovered channel has refcount 0 and no live audit.
    #[test]
    fn explain_channel_uncovered_is_empty() {
        let r = SubscriptionReconciler::new().unwrap();
        let why = r.explain_channel("nope");
        assert_eq!(why.refcount, 0);
        assert!(why.owners.is_empty());
        assert!(why.last_kind.is_none());
        assert!(why.cause.is_none());
    }
}
