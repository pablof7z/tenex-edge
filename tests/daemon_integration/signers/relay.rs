use serde_json::Value;
use std::process::Command;

pub(super) fn status_authors_on_relay(relay: &str, channel: &str) -> Vec<String> {
    status_events_on_relay(relay, channel)
        .into_iter()
        .filter_map(|event| {
            event
                .get("pubkey")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .collect()
}

pub(super) fn status_evidence_on_relay(
    relay: &str,
    channel: &str,
) -> Vec<(String, String, String)> {
    status_events_on_relay(relay, channel)
        .into_iter()
        .filter_map(|event| {
            let author = event.get("pubkey")?.as_str()?.to_string();
            let tags = event.get("tags")?.as_array()?;
            let tag_value = |name: &str| {
                tags.iter().find_map(|tag| {
                    let parts = tag.as_array()?;
                    (parts.first()?.as_str()? == name)
                        .then(|| parts.get(1)?.as_str().map(str::to_string))
                        .flatten()
                })
            };
            Some((
                author,
                tag_value("h").unwrap_or_default(),
                tag_value("d").unwrap_or_default(),
            ))
        })
        .collect()
}

pub(super) fn relay_has_status_authors(relay: &str, channel: &str, expected: &[&str]) -> bool {
    let authors = status_authors_on_relay(relay, channel);
    expected
        .iter()
        .all(|author| authors.iter().any(|observed| observed.as_str() == *author))
}

fn status_events_on_relay(relay: &str, channel: &str) -> Vec<Value> {
    let output = Command::new("nak")
        .args(["req", "-k", "30315", "-h", channel, "-l", "20", relay])
        .output()
        .expect("run nak req");
    assert!(
        output.status.success(),
        "nak req failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let value: Value = serde_json::from_str(line).ok()?;
            if value.get("pubkey").is_some() {
                Some(value)
            } else {
                value.get(2).cloned()
            }
        })
        .collect()
}
