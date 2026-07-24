use anyhow::{bail, Result};

pub(super) enum Action<'a> {
    Reply,
    Tag(&'a str),
}

pub(super) fn reject(sender_pubkey: &str, target_pubkey: &str, action: Action<'_>) -> Result<()> {
    if sender_pubkey != target_pubkey {
        return Ok(());
    }
    match action {
        Action::Reply => bail!(
            "you are trying to reply to your own message; replying to yourself is not allowed. \
             This is probably a mistake; did you mean to reply to someone else's message?"
        ),
        Action::Tag(label) => bail!(
            "you are trying to --tag yourself ({label}); tagging yourself is not allowed. \
             This is probably a mistake; did you mean to tag a different agent?"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn another_identity_remains_a_valid_reply_or_tag_target() {
        assert!(reject("sender", "recipient", Action::Reply).is_ok());
        assert!(reject("sender", "recipient", Action::Tag("recipient")).is_ok());
    }
}
