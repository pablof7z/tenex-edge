const EVENT_LIMITATION: &str = "event validation can prove local materialization; Trellis explanation is available only when a receipt records this event as an artifact";

pub(super) struct EventOutcome<'a> {
    pub(super) requested_id: &'a str,
    pub(super) resolved_id: &'a str,
    pub(super) found: bool,
    pub(super) receipt_count: usize,
    pub(super) message: Option<&'a crate::state::Message>,
    pub(super) relay_event: Option<&'a crate::state::RelayEvent>,
    pub(super) quarantine_found: bool,
    pub(super) outbox_failed: bool,
    pub(super) outbox_pending: bool,
    pub(super) outbox_published: bool,
}

pub(super) fn summary(outcome: &EventOutcome<'_>) -> String {
    if outcome.quarantine_found {
        return format!(
            "event `{}` is quarantined before normal materialization",
            outcome.resolved_id
        );
    }
    if outcome.receipt_count > 0 {
        if outcome.outbox_published {
            return format!(
                "event `{}` has {} Trellis receipt(s) and published outbox evidence",
                outcome.resolved_id, outcome.receipt_count
            );
        }
        return format!(
            "event `{}` has {} Trellis receipt(s)",
            outcome.resolved_id, outcome.receipt_count
        );
    }
    if let Some(message) = outcome.message {
        return format!(
            "event `{}` is a chat message with sync_state `{}` in channel `{}`",
            message.message_id, message.sync_state, message.channel_h
        );
    }
    if let Some(event) = outcome.relay_event {
        return format!(
            "event `{}` is cached as relay kind {} in channel `{}`",
            event.id, event.kind, event.channel_h
        );
    }
    if outcome.outbox_published {
        return format!(
            "event `{}` is published in the outbox ledger",
            outcome.resolved_id
        );
    }
    if outcome.outbox_pending {
        return format!(
            "event `{}` is pending in the outbox ledger",
            outcome.resolved_id
        );
    }
    format!(
        "event `{}` is not locally materialized",
        outcome.requested_id
    )
}

pub(super) fn reason(outcome: &EventOutcome<'_>) -> &'static str {
    if !outcome.found {
        return "no Trellis receipt, outbox row, canonical message row, or relay event row matched this event id prefix";
    }
    if outcome.quarantine_found {
        return "relay event is quarantined and has not been admitted to canonical event/message state";
    }
    if outcome.outbox_failed {
        return "outbox row records a failed relay publish outcome";
    }
    if let Some(message) = outcome.message {
        if message.error.as_deref().is_some_and(|s| !s.is_empty())
            || super::is_failed_state(&message.sync_state)
        {
            return "canonical message row records a failed/rejected sync state";
        }
        if super::is_provisional_state(&message.sync_state) || message.native_event_id.is_none() {
            return "canonical message row exists, but relay/native event materialization is not proven";
        }
    }
    if outcome.outbox_pending && !outcome.outbox_published {
        return "outbox row exists, but relay acceptance is still pending";
    }
    if outcome.receipt_count > 0 {
        if outcome.outbox_published {
            return "Trellis receipts explain this event artifact and outbox evidence proves relay publish completion";
        }
        return "Trellis receipts explain this event artifact";
    }
    if outcome.outbox_published {
        return "outbox evidence proves relay publish completion, but no Trellis receipt explains this event";
    }
    if outcome.message.is_some() {
        return "canonical message row proves local chat materialization, but no Trellis receipt explains this event";
    }
    if outcome.relay_event.is_some() {
        return "raw relay cache proves local materialization, but no Trellis receipt explains this event";
    }
    EVENT_LIMITATION
}
