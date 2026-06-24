use crate::identity;
use anyhow::{bail, Result};
use nostr_sdk::prelude::{Keys, SecretKey};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct SignerSlot {
    agent_pubkey: String,
    project: String,
}

impl SignerSlot {
    fn new(agent_pubkey: &str, project: &str) -> Self {
        Self {
            agent_pubkey: agent_pubkey.to_string(),
            project: project.to_string(),
        }
    }
}

pub(super) type SignerReservations = HashMap<SignerSlot, String>;

pub(super) struct SignerRequest<'a> {
    pub session_id: &'a str,
    pub agent_pubkey: &'a str,
    pub agent_slug: &'a str,
    pub project: &'a str,
    pub harness_kind: &'a str,
    pub anchor: &'a str,
    pub existing_session_pubkey: Option<String>,
    pub tenex_secret: Option<&'a SecretKey>,
}

pub(super) enum SessionSigner {
    Durable,
    Transient { keys: Keys, pubkey: String },
}

impl SessionSigner {
    pub(super) fn session_keys(&self) -> Option<Keys> {
        match self {
            Self::Durable => None,
            Self::Transient { keys, .. } => Some(keys.clone()),
        }
    }

    pub(super) fn transient_pubkey(&self) -> Option<&str> {
        match self {
            Self::Durable => None,
            Self::Transient { pubkey, .. } => Some(pubkey.as_str()),
        }
    }
}

pub(super) fn select_and_reserve(
    reservations: &mut SignerReservations,
    session_keys: &mut HashMap<String, Keys>,
    req: SignerRequest<'_>,
) -> Result<SessionSigner> {
    if req.existing_session_pubkey.is_some() {
        return reserve_transient(session_keys, req);
    }

    let slot = SignerSlot::new(req.agent_pubkey, req.project);
    match reservations.get(&slot) {
        None => {
            reservations.insert(slot, req.session_id.to_string());
            Ok(SessionSigner::Durable)
        }
        Some(owner) if owner == req.session_id => Ok(SessionSigner::Durable),
        Some(_) => reserve_transient(session_keys, req),
    }
}

pub(super) fn release(
    reservations: &mut SignerReservations,
    session_keys: &mut HashMap<String, Keys>,
    session_id: &str,
    agent_pubkey: &str,
    project: &str,
) -> Option<Keys> {
    let slot = SignerSlot::new(agent_pubkey, project);
    if reservations.get(&slot).map(String::as_str) == Some(session_id) {
        reservations.remove(&slot);
    }
    session_keys.remove(session_id)
}

fn reserve_transient(
    session_keys: &mut HashMap<String, Keys>,
    req: SignerRequest<'_>,
) -> Result<SessionSigner> {
    let Some(secret) = req.tenex_secret else {
        bail!("cannot derive transient signer without tenexPrivateKey");
    };
    let keys = identity::derive_session_keys(
        secret,
        req.project,
        req.agent_slug,
        req.harness_kind,
        req.anchor,
    );
    let pubkey = keys.public_key().to_hex();
    if let Some(existing) = req.existing_session_pubkey.as_deref() {
        if existing != pubkey {
            bail!(
                "stored session pubkey {} does not match rederived pubkey {}",
                crate::util::pubkey_short(existing),
                crate::util::pubkey_short(&pubkey)
            );
        }
    }
    session_keys.insert(req.session_id.to_string(), keys.clone());
    Ok(SessionSigner::Transient { keys, pubkey })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_secret() -> SecretKey {
        SecretKey::from_slice(&[0x22; 32]).unwrap()
    }

    fn request<'a>(
        session_id: &'a str,
        project: &'a str,
        secret: &'a SecretKey,
    ) -> SignerRequest<'a> {
        SignerRequest {
            session_id,
            agent_pubkey: "durable-pubkey",
            agent_slug: "claude",
            project,
            harness_kind: "codex",
            anchor: session_id,
            existing_session_pubkey: None,
            tenex_secret: Some(secret),
        }
    }

    #[test]
    fn first_session_in_group_uses_durable_signer() {
        let secret = test_secret();
        let mut reservations = SignerReservations::new();
        let mut session_keys = HashMap::new();

        let signer = select_and_reserve(
            &mut reservations,
            &mut session_keys,
            request("s1", "hello", &secret),
        )
        .unwrap();

        assert!(matches!(signer, SessionSigner::Durable));
        assert!(session_keys.is_empty());
    }

    #[test]
    fn second_same_agent_same_group_uses_transient_signer() {
        let secret = test_secret();
        let mut reservations = SignerReservations::new();
        let mut session_keys = HashMap::new();

        select_and_reserve(
            &mut reservations,
            &mut session_keys,
            request("s1", "hello", &secret),
        )
        .unwrap();
        let signer = select_and_reserve(
            &mut reservations,
            &mut session_keys,
            request("s2", "hello", &secret),
        )
        .unwrap();

        assert!(matches!(signer, SessionSigner::Transient { .. }));
        assert!(session_keys.contains_key("s2"));
    }

    #[test]
    fn same_agent_in_different_groups_uses_durable_signer() {
        let secret = test_secret();
        let mut reservations = SignerReservations::new();
        let mut session_keys = HashMap::new();

        select_and_reserve(
            &mut reservations,
            &mut session_keys,
            request("s1", "hello", &secret),
        )
        .unwrap();
        let signer = select_and_reserve(
            &mut reservations,
            &mut session_keys,
            request("s2", "other", &secret),
        )
        .unwrap();

        assert!(matches!(signer, SessionSigner::Durable));
        assert!(!session_keys.contains_key("s2"));
    }

    #[test]
    fn resumed_transient_session_reuses_stored_pubkey() {
        let secret = test_secret();
        let mut reservations = SignerReservations::new();
        let mut session_keys = HashMap::new();
        let keys = identity::derive_session_keys(&secret, "hello", "claude", "codex", "s2");
        let pubkey = keys.public_key().to_hex();
        let mut req = request("s2", "hello", &secret);
        req.existing_session_pubkey = Some(pubkey.clone());

        let signer = select_and_reserve(&mut reservations, &mut session_keys, req).unwrap();

        assert_eq!(signer.transient_pubkey(), Some(pubkey.as_str()));
        assert_eq!(
            session_keys.get("s2").unwrap().public_key().to_hex(),
            pubkey
        );
    }

    #[test]
    fn reservation_mutex_shape_prevents_two_durable_winners() {
        use std::sync::{Arc, Mutex};
        use std::thread;

        let reservations = Arc::new(Mutex::new(SignerReservations::new()));
        let session_keys = Arc::new(Mutex::new(HashMap::new()));
        let winners = Arc::new(Mutex::new(Vec::new()));

        thread::scope(|scope| {
            for session_id in ["s1", "s2"] {
                let reservations = Arc::clone(&reservations);
                let session_keys = Arc::clone(&session_keys);
                let winners = Arc::clone(&winners);
                scope.spawn(move || {
                    let secret = test_secret();
                    let mut reservations = reservations.lock().unwrap();
                    let mut session_keys = session_keys.lock().unwrap();
                    let signer = select_and_reserve(
                        &mut reservations,
                        &mut session_keys,
                        request(session_id, "hello", &secret),
                    )
                    .unwrap();
                    winners
                        .lock()
                        .unwrap()
                        .push(matches!(signer, SessionSigner::Durable));
                });
            }
        });

        let durable_count = winners
            .lock()
            .unwrap()
            .iter()
            .filter(|durable| **durable)
            .count();
        assert_eq!(durable_count, 1);
    }
}
