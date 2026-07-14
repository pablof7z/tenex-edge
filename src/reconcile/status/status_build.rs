use super::StatusCommand;
use crate::domain::{AgentRef, Status};

pub(super) fn to_status(cmd: &StatusCommand, ttl_secs: u64, now: u64, expiring: bool) -> Status {
    Status {
        agent: AgentRef::new(cmd.pubkey.clone(), cmd.slug.clone()),
        channels: cmd.channels.clone(),
        host: cmd.host.clone(),
        title: cmd.title.clone(),
        activity: if expiring {
            String::new()
        } else {
            cmd.activity.clone()
        },
        busy: !expiring && cmd.busy,
        rel_cwd: cmd.rel_cwd.clone(),
        expires_at: Some(if expiring {
            now
        } else {
            now.saturating_add(ttl_secs)
        }),
        dispatch_event: cmd.dispatch_event.clone(),
    }
}
