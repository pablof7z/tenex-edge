pub(in crate::daemon::server::probe::validate) struct RecipientTarget {
    pub(super) message_prefix: String,
    pub(super) recipient_pubkey: String,
    pub(super) target_session: Option<String>,
}

pub(in crate::daemon::server::probe::validate) fn recipient_target(
    target: &str,
) -> Option<RecipientTarget> {
    colon_target(target, "recipient:")
        .or_else(|| colon_target(target, "delivery:"))
        .or_else(|| path_target(target, "recipient/"))
        .or_else(|| path_target(target, "delivery/"))
}

fn colon_target(target: &str, prefix: &str) -> Option<RecipientTarget> {
    let rest = target.strip_prefix(prefix)?;
    let mut parts = rest.splitn(3, ':');
    let message_prefix = parts.next()?;
    let recipient_pubkey = parts.next()?;
    let target_session = parts.next();
    build_target(message_prefix, recipient_pubkey, target_session)
}

fn path_target(target: &str, prefix: &str) -> Option<RecipientTarget> {
    let rest = target.strip_prefix(prefix)?;
    let mut parts = rest.splitn(3, '/');
    let message_prefix = parts.next()?;
    let recipient_pubkey = parts.next()?;
    let target_session = parts.next();
    build_target(message_prefix, recipient_pubkey, target_session)
}

fn build_target(
    message_prefix: &str,
    recipient_pubkey: &str,
    target_session: Option<&str>,
) -> Option<RecipientTarget> {
    (!message_prefix.trim().is_empty() && !recipient_pubkey.trim().is_empty()).then(|| {
        RecipientTarget {
            message_prefix: message_prefix.to_string(),
            recipient_pubkey: recipient_pubkey.to_string(),
            target_session: target_session
                .filter(|session| !session.trim().is_empty())
                .map(str::to_string),
        }
    })
}
