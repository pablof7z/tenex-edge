use crate::state::Store;

pub(crate) fn p_tag_pubkeys(tags_json: &str) -> Vec<String> {
    let Ok(tags) = serde_json::from_str::<Vec<Vec<String>>>(tags_json) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for tag in tags {
        if tag.first().is_some_and(|t| t == "p") {
            if let Some(pubkey) = tag.get(1).filter(|p| !p.is_empty()) {
                if !out.iter().any(|seen| seen == pubkey) {
                    out.push(pubkey.clone());
                }
            }
        }
    }
    out
}

pub(super) fn mentions_pubkey(tags_json: &str, pubkey: &str) -> bool {
    if pubkey.is_empty() {
        return false;
    }
    p_tag_pubkeys(tags_json).iter().any(|p| p == pubkey)
}

/// A chat event is backend↔party traffic when its author OR any directed `p`-tag
/// recipient is a backend — either this daemon's own management key
/// (`backend_pubkey`, reliable on a cold cache) or a pubkey whose cached kind:0
/// declares `is_backend` (covers remote backends). Such traffic is excluded from
/// ambient `<chatter>`, symmetric with the roster's backend exclusion in
/// `assemble::member_rows`.
pub(crate) fn is_backend_traffic(
    store: &Store,
    backend_pubkey: &str,
    author: &str,
    tags_json: &str,
) -> bool {
    if is_backend_pubkey(store, backend_pubkey, author) {
        return true;
    }
    p_tag_pubkeys(tags_json)
        .iter()
        .any(|pk| is_backend_pubkey(store, backend_pubkey, pk))
}

pub(crate) fn is_backend_pubkey(store: &Store, backend_pubkey: &str, pubkey: &str) -> bool {
    (!backend_pubkey.is_empty() && pubkey == backend_pubkey) || is_backend(store, pubkey)
}

fn is_backend(store: &Store, pubkey: &str) -> bool {
    store
        .get_profile(pubkey)
        .ok()
        .flatten()
        .map(|p| p.is_backend)
        .unwrap_or(false)
}
