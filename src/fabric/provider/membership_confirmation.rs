use super::Nip29Provider;
use crate::fabric::group_management::GroupMutationOutcome;
use crate::util::now_secs;

impl Nip29Provider {
    pub(crate) async fn grant_member_confirmed(
        &self,
        channel: &str,
        pubkey: &str,
    ) -> GroupMutationOutcome {
        self.confirm_role_grant(channel, pubkey, false).await
    }

    pub(crate) async fn grant_admin_confirmed(
        &self,
        channel: &str,
        pubkey: &str,
    ) -> GroupMutationOutcome {
        self.confirm_role_grant(channel, pubkey, true).await
    }

    pub(crate) async fn remove_member_confirmed(
        &self,
        channel: &str,
        pubkey: &str,
    ) -> GroupMutationOutcome {
        for attempt in 0..6u32 {
            let outcome = self.nip29_remove_member_outcome(channel, pubkey).await;
            match self.try_fetch_group_state(channel).await {
                Ok((_, roles, members)) => {
                    if !members.contains(pubkey) && !roles.contains_key(pubkey) {
                        self.with_store(|s| {
                            if let Err(e) = s.remove_channel_member(channel, pubkey) {
                                tracing::error!(
                                    channel,
                                    pubkey,
                                    error = %e,
                                    "remove_member_confirmed: local mirror remove failed after confirmed relay removal"
                                );
                            }
                        });
                        return GroupMutationOutcome::Confirmed;
                    }
                }
                Err(e) => {
                    tracing::error!(
                        channel,
                        pubkey,
                        attempt,
                        error = %e,
                        "remove_member_confirmed: relay read-back failed; cannot confirm removal"
                    );
                }
            }
            if outcome.is_rejected() {
                return GroupMutationOutcome::Rejected;
            }
            tokio::time::sleep(std::time::Duration::from_millis(250 * (attempt as u64 + 1))).await;
        }
        GroupMutationOutcome::Unconfirmed
    }

    async fn confirm_role_grant(
        &self,
        channel: &str,
        pubkey: &str,
        want_admin: bool,
    ) -> GroupMutationOutcome {
        for attempt in 0..6u32 {
            let outcome = if want_admin {
                self.nip29_add_admin_outcome(channel, pubkey).await
            } else {
                self.nip29_add_member_outcome(channel, pubkey).await
            };
            // Confirm ONLY on a relay state we actually OBSERVED. A read-back
            // failure must never be promoted to "grant confirmed".
            match self.try_fetch_group_state(channel).await {
                Ok((_, roles, members)) => {
                    let present = if want_admin {
                        roles.get(pubkey).map(String::as_str) == Some("admin")
                    } else {
                        members.contains(pubkey) || roles.contains_key(pubkey)
                    };
                    if present {
                        let role = if want_admin { "admin" } else { "member" };
                        self.with_store(|s| {
                            if let Err(e) = s.upsert_channel_member(channel, pubkey, role, now_secs())
                            {
                                tracing::error!(
                                    channel,
                                    pubkey,
                                    role,
                                    error = %e,
                                    "confirm_role_grant: local mirror write failed after confirmed relay grant"
                                );
                            }
                        });
                        return GroupMutationOutcome::Confirmed;
                    }
                }
                Err(e) => {
                    tracing::error!(
                        channel,
                        pubkey,
                        attempt,
                        error = %e,
                        "confirm_role_grant: relay read-back failed; cannot confirm grant"
                    );
                }
            }
            if outcome.is_rejected() {
                return GroupMutationOutcome::Rejected;
            }
            tokio::time::sleep(std::time::Duration::from_millis(250 * (attempt as u64 + 1))).await;
        }
        GroupMutationOutcome::Unconfirmed
    }
}
