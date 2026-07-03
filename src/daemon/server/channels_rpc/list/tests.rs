use super::*;

fn channel(id: &str, name: &str, about: &str, parent: &str) -> crate::state::Channel {
    crate::state::Channel {
        channel_h: id.to_string(),
        name: name.to_string(),
        about: about.to_string(),
        parent: parent.to_string(),
        created_at: 1,
        updated_at: 1,
    }
}

#[test]
fn channel_list_rooms_hides_archived_channels() {
    let rooms = channel_list_rooms(
        vec![
            channel("root", "root", "", ""),
            channel("active", "active", "current", "root"),
            channel("archived", "archived", "[ARCHIVED] done", "root"),
            channel("child", "child", "nested", "archived"),
        ],
        "root",
    );

    let ids = rooms
        .iter()
        .filter_map(|row| row["child_h"].as_str())
        .collect::<Vec<_>>();
    assert_eq!(ids, vec!["active"]);
}
