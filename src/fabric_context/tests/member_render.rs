//! `<members>` rendering: role + `@codename@host` per member, with the legacy
//! (`people`) and pure (`assemble`) paths proven byte-identical.

use crate::fabric_context::{assemble, capture_inputs, render_fabric_context, render_view_text};
use crate::state::Status;

use super::{input, seed_store, session, OTHER_PK};

/// A member whose session is known (a live status carries its session id) renders
/// as `@<codename>` with its relay role; the pure and legacy paths agree.
#[test]
fn member_row_shows_role_and_codename_for_peer_session() {
    let store = seed_store();
    let rec = session(&store);
    store
        .upsert_profile_with_agent_slug(
            OTHER_PK, "reviewer", "reviewer", "reviewer", "laptop", false, 2,
        )
        .unwrap();
    store
        .upsert_status(&Status {
            pubkey: OTHER_PK.into(),
            session_id: "peer-sess".into(),
            channel_h: "root".into(),
            slug: "reviewer".into(),
            title: "Reviewing".into(),
            activity: String::new(),
            busy: false,
            last_seen: 90,
            updated_at: 90,
            expiration: 500,
        })
        .unwrap();

    let codename = crate::util::friendly_short_code("peer-sess");
    let text = render_fabric_context(&store, input(Some(&rec), "root", 0, 100, true))
        .expect("context should render");
    // Self keeps its slug ref; the peer session renders under its codename, both
    // carrying their relay role.
    assert!(
        text.contains("<member ref=\"@coder\" agentSlug=\"coder\" role=\"member\""),
        "got: {text}"
    );
    assert!(
        text.contains(&format!(
            "<member ref=\"@{codename}\" agentSlug=\"reviewer\" role=\"member\""
        )),
        "got: {text}"
    );

    // Parity: the pure capture→assemble path renders byte-identically.
    let captured = capture_inputs(&store, &input(Some(&rec), "root", 0, 100, true));
    let trellis = render_view_text(&assemble::assemble_view(&captured, 0, 100));
    assert_eq!(trellis, text);
}
