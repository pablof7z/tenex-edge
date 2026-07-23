use super::{render_agent_who, render_agent_who_from_aggregation, AgentWhoInput};
use crate::state::{Status, Store};
use std::collections::BTreeSet;

fn advertise_host(
    store: &Store,
    pubkey: &str,
    host: &str,
    agents: &[(&str, &str)],
    workspaces: &[&str],
    updated_at: u64,
) {
    let agents = agents
        .iter()
        .map(|(slug, about)| ((*slug).to_string(), (*about).to_string()))
        .collect::<Vec<_>>();
    let workspaces = workspaces
        .iter()
        .map(|workspace| (*workspace).to_string())
        .collect::<Vec<_>>();
    store
        .upsert_profile_snapshot(
            pubkey,
            host,
            host,
            "",
            host,
            true,
            &agents,
            &workspaces,
            updated_at,
        )
        .unwrap();
    for workspace in workspaces {
        store
            .upsert_channel_member(&workspace, pubkey, "admin", updated_at)
            .unwrap();
    }
}

fn seed() -> Store {
    let store = Store::open_memory().unwrap();
    store
        .upsert_channel("alpha", "alpha", "Alpha workspace", "", 1)
        .unwrap();
    store
        .upsert_channel("beta", "beta", "Beta workspace", "", 1)
        .unwrap();
    store
        .upsert_channel("joined", "planning", "Plan work", "alpha", 1)
        .unwrap();
    store
        .upsert_channel("compact", "small-talk", "Chit chat", "alpha", 1)
        .unwrap();
    store.upsert_workspace("alpha", "/work/alpha", 1).unwrap();
    store
        .upsert_profile_with_agent_slug(
            "self-pk",
            "quill-peak-369-codex",
            "quill-peak-369-codex",
            "codex",
            "laptop",
            false,
            1,
        )
        .unwrap();
    store
        .upsert_profile("human-pk", "Pablo", "Pablo", "", false, 1)
        .unwrap();
    store
        .upsert_profile("backend-pk", "remote", "remote", "remote", true, 1)
        .unwrap();
    for channel in ["alpha", "joined"] {
        store
            .upsert_channel_member(channel, "self-pk", "member", 1)
            .unwrap();
    }
    store
        .upsert_channel_member("beta", "self-pk", "member", 1)
        .unwrap();
    store
        .upsert_channel_member("alpha", "human-pk", "admin", 1)
        .unwrap();
    store
        .upsert_channel_member("alpha", "backend-pk", "admin", 1)
        .unwrap();
    store
        .upsert_channel_member("compact", "human-pk", "member", 1)
        .unwrap();
    store
        .upsert_status(&Status {
            pubkey: "self-pk".into(),
            channel_h: "alpha".into(),
            slug: "quill-peak-369-codex".into(),
            title: "Implement awareness".into(),
            activity: String::new(),
            state: crate::session_state::SessionState::Idle,
            state_since: 99,
            last_seen: 99,
            updated_at: 99,
            expiration: 200,
        })
        .unwrap();
    advertise_host(
        &store,
        "remote-pk",
        "remoteBackend1",
        &[("claude", "Remote review")],
        &["alpha", "beta"],
        1,
    );
    store
}

fn render(expand_beta: bool) -> String {
    let store = seed();
    let roots = vec!["alpha".to_string(), "beta".to_string()];
    let mut expanded_workspaces = BTreeSet::from(["alpha".to_string()]);
    if expand_beta {
        expanded_workspaces.insert("beta".to_string());
    }
    render_agent_who(
        &store,
        AgentWhoInput {
            roots: &roots,
            self_name: "quill-peak-369-codex",
            self_pubkey: "self-pk",
            local_host: "laptop",
            backend_pubkey: "backend-pk",
            now: 100,
            headless: false,
            expanded_workspaces: &expanded_workspaces,
        },
    )
    .unwrap()
}

#[test]
fn both_renderers_use_one_immutable_capture() {
    let store = seed();
    let roots = vec!["alpha".to_string(), "beta".to_string()];
    let expanded = BTreeSet::from(["alpha".to_string()]);
    let aggregation = crate::who_aggregation::WhoAggregation::load(&store, 100).unwrap();
    let render_view = || {
        render_agent_who_from_aggregation(
            &aggregation,
            AgentWhoInput {
                roots: &roots,
                self_name: "quill-peak-369-codex",
                self_pubkey: "self-pk",
                local_host: "laptop",
                backend_pubkey: "backend-pk",
                now: 100,
                headless: false,
                expanded_workspaces: &expanded,
            },
        )
    };
    let before_view = render_view().unwrap();
    let before_snapshot =
        crate::who_snapshot::build_who_snapshot(&aggregation, Some("alpha"), 100, "laptop")
            .unwrap();

    store.upsert_workspace("alpha", "/mutated", 101).unwrap();
    store
        .upsert_profile("human-pk", "Changed", "Changed", "", false, 101)
        .unwrap();
    store.remove_channel_member("alpha", "human-pk").unwrap();

    assert_eq!(render_view().unwrap(), before_view);
    assert_eq!(
        crate::who_snapshot::build_who_snapshot(&aggregation, Some("alpha"), 100, "laptop")
            .unwrap(),
        before_snapshot
    );
}

#[test]
fn lists_global_agents_and_compacts_other_workspaces() {
    let xml = render(false);
    assert!(
        xml.contains("<self name=\"@quill-peak-369-codex\" host=\"laptop\" headless=\"off\" />"),
        "{xml}"
    );
    assert!(xml.contains("<host name=\"remoteBackend1\">"), "{xml}");
    assert!(!xml.contains("<host name=\"remote\">"), "{xml}");
    assert!(
        xml.contains("<agent ref=\"claude@remoteBackend1\" about=\"Remote review\" />"),
        "{xml}"
    );
    assert!(
        xml.contains(
            "<workspace name=\"alpha\" about=\"Alpha workspace\" members=\"2\" hosts=\"remoteBackend1\""
        ),
        "{xml}"
    );
    assert!(xml.contains(
        "<workspace name=\"beta\" about=\"Beta workspace\" members=\"1\" hosts=\"remoteBackend1\" />"
    ));
    assert!(!xml.contains(" path="), "{xml}");
    assert!(!xml.contains(" channel=\"alpha\""), "{xml}");
}

#[test]
fn agent_about_is_compact_and_bounded() {
    let store = seed();
    let long_about = format!("Routes\\narchitecture work {}", "carefully ".repeat(40));
    advertise_host(
        &store,
        "long-pk",
        "laptop",
        &[("architect", &long_about)],
        &["alpha"],
        2,
    );
    let roots = vec!["alpha".to_string()];
    let xml = render_agent_who(
        &store,
        AgentWhoInput {
            roots: &roots,
            self_name: "quill-peak-369-codex",
            self_pubkey: "self-pk",
            local_host: "laptop",
            backend_pubkey: "backend-pk",
            now: 100,
            headless: false,
            expanded_workspaces: &BTreeSet::from(["alpha".to_string()]),
        },
    )
    .unwrap();
    let start = xml.find("<agent ref=\"architect@laptop\"").unwrap();
    let row = &xml[start..xml[start..].find(" />").map(|end| start + end).unwrap()];
    let about = row
        .split_once("about=\"")
        .unwrap()
        .1
        .split_once('"')
        .unwrap()
        .0;

    assert!(!about.contains("\\n"), "{about}");
    assert!(about.chars().count() <= crate::agent_about::AGENT_ABOUT_MAX_CHARS);
    assert!(about.ends_with('…'), "{about}");
}

#[test]
fn workspace_carries_root_members_and_membership_gated_children() {
    let xml = render(false);
    assert!(
        xml.contains(
            "<workspace name=\"alpha\" about=\"Alpha workspace\" members=\"2\" hosts=\"remoteBackend1\""
        ),
        "{xml}"
    );
    assert!(xml.contains("<human name=\"@Pablo\" state=\"offline\""));
    assert!(xml.contains(
        "<agent name=\"@quill-peak-369-codex\" state=\"idle\" status=\"Implement awareness\""
    ));
    assert!(
        xml.contains("members=\"2\" hosts=\"remoteBackend1\">\n      <members>"),
        "{xml}"
    );
    assert!(xml.contains(
        "<channel name=\"small-talk\" id=\"alpha.small-talk\" members=\"1\" about=\"Chit chat\" />"
    ));
    assert!(!xml.contains("general"), "synthetic root leaked: {xml}");
    assert!(!xml.contains("@remote\" state="), "backend leaked: {xml}");
}

#[test]
fn exact_session_joined_workspace_set_controls_expansion() {
    let xml = render(true);
    assert!(xml.contains(
        "<workspace name=\"beta\" about=\"Beta workspace\" members=\"1\" hosts=\"remoteBackend1\">"
    ));
    assert!(xml.contains("<agent name=\"@quill-peak-369-codex\" state=\"offline\""));
}
