use super::*;

pub(in crate::daemon::server) fn select_session_signer(
    state: &Arc<DaemonState>,
    session_id: &str,
    agent_pubkey: &str,
    agent_slug: &str,
    project: &str,
    harness_kind: &str,
    anchor: &str,
) -> Result<session_signer::SessionSigner> {
    let existing_session_pubkey = state.with_store(|s| s.session_pubkey_for_session(session_id));
    let tenex_keys = match state.cfg.session_ikm_nsec() {
        Some(nsec) => Some(
            Keys::parse(nsec)
                .context("parsing tenexPrivateKey for transient session signer derivation")?,
        ),
        None => None,
    };
    let signer = {
        let mut reservations = state.session_signers.lock().unwrap();
        let mut session_keys = state.session_keys.lock().unwrap();
        session_signer::select_and_reserve(
            &mut reservations,
            &mut session_keys,
            session_signer::SignerRequest {
                session_id,
                agent_pubkey,
                agent_slug,
                project,
                harness_kind,
                anchor,
                existing_session_pubkey,
                tenex_secret: tenex_keys.as_ref().map(Keys::secret_key),
            },
        )?
    };
    if let Some(session_pubkey) = signer.transient_pubkey() {
        if let Err(e) = state.with_store(|s| {
            s.upsert_session_pubkey(
                session_pubkey,
                session_id,
                agent_pubkey,
                agent_slug,
                now_secs(),
            )
        }) {
            state.release_session_signer(session_id, agent_pubkey, project);
            return Err(e);
        }
    }
    Ok(signer)
}

pub(in crate::daemon::server) async fn admit_transient_signer(
    state: &Arc<DaemonState>,
    project: &str,
    session_pubkey: &str,
) -> Result<()> {
    let add = state.provider.nip29_add_member(project, session_pubkey);
    let accepted = tokio::time::timeout(std::time::Duration::from_secs(8), add)
        .await
        .unwrap_or(false);
    if !accepted {
        anyhow::bail!(
            "NIP-29 admission failed for transient signer {} in {project}",
            pubkey_short(session_pubkey)
        );
    }
    state.with_store(|s| s.upsert_group_member(project, session_pubkey, "member", now_secs()))?;
    Ok(())
}
