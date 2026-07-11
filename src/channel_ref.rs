use crate::state::Store;

const MAX_CHANNEL_REF_DEPTH: usize = 32;

/// Full, agent-facing channel path for reply instructions. The workspace's
/// durable root `h` is an address namespace; its root channel is always named
/// `general`, so every path starts `workspace.general`.
pub(crate) fn full_channel_ref(store: &Store, channel_h: &str) -> String {
    let mut parts = Vec::new();
    let mut cur = channel_h.to_string();
    let mut workspace = channel_h.to_string();
    for _ in 0..MAX_CHANNEL_REF_DEPTH {
        let Some(channel) = store.get_channel(&cur).ok().flatten() else {
            if parts.is_empty() {
                return format!("{channel_h}.general");
            }
            parts.push(cur);
            break;
        };
        if channel.parent.is_empty() {
            workspace = channel.channel_h;
            break;
        }
        parts.push(
            channel
                .human_name()
                .map(str::to_string)
                .unwrap_or_else(|| channel.channel_h.clone()),
        );
        cur = channel.parent;
    }
    parts.reverse();
    let mut reference = vec![workspace, "general".to_string()];
    reference.extend(parts);
    reference.join(".")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_channel_ref_walks_to_workspace_root() {
        let store = Store::open_memory().unwrap();
        store
            .upsert_channel("root-h", "workspace", "", "", 1)
            .unwrap();
        store
            .upsert_channel("child-h", "channel", "", "root-h", 2)
            .unwrap();
        store
            .upsert_channel("qa-h", "qa", "", "child-h", 3)
            .unwrap();

        assert_eq!(
            full_channel_ref(&store, "qa-h"),
            "root-h.general.channel.qa"
        );
    }

    #[test]
    fn full_channel_ref_falls_back_to_unknown_h() {
        let store = Store::open_memory().unwrap();

        assert_eq!(full_channel_ref(&store, "opaque"), "opaque.general");
    }

    #[test]
    fn workspace_root_is_general() {
        let store = Store::open_memory().unwrap();
        store
            .upsert_channel("workspace", "general", "", "", 1)
            .unwrap();

        assert_eq!(full_channel_ref(&store, "workspace"), "workspace.general");
    }
}
