use anyhow::{bail, Result};

pub(crate) const DISTILL_PAUSE_SECS: u64 = 30 * 60;
pub(crate) const MAX_WORDS: usize = 15;

pub(crate) fn normalize(topic: &str) -> Result<String> {
    let words = topic.split_whitespace().collect::<Vec<_>>();
    if words.is_empty() {
        bail!("session title must not be empty");
    }
    if words.len() > MAX_WORDS {
        bail!("session title must be at most {MAX_WORDS} words");
    }
    Ok(words.join(" "))
}

pub(crate) fn suppresses_distillation(topic: &str, set_at: u64, now: u64) -> bool {
    !topic.is_empty() && now < set_at.saturating_add(DISTILL_PAUSE_SECS)
}

pub(crate) fn is_visible(topic: &str, _set_at: u64, _now: u64) -> bool {
    !topic.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_and_limits_topics_by_words() {
        assert_eq!(
            normalize("  Researching   MCP allocation ").unwrap(),
            "Researching MCP allocation"
        );
        assert!(normalize("").is_err());
        assert!(normalize(&vec!["word"; MAX_WORDS + 1].join(" ")).is_err());
    }

    #[test]
    fn explicit_topic_is_visible_while_pausing_distillation() {
        let topic = "Researching MCP improvements around resource allocation";
        assert!(suppresses_distillation(
            topic,
            100,
            100 + DISTILL_PAUSE_SECS - 1
        ));
        assert!(is_visible(topic, 100, 100 + DISTILL_PAUSE_SECS - 1));
        assert!(!suppresses_distillation(
            topic,
            100,
            100 + DISTILL_PAUSE_SECS
        ));
        assert!(is_visible(topic, 100, 100 + DISTILL_PAUSE_SECS));
    }
}
