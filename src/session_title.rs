use anyhow::{bail, Result};

pub(crate) const MAX_WORDS: usize = 15;

pub(crate) fn normalize(title: &str) -> Result<String> {
    let words = title.split_whitespace().collect::<Vec<_>>();
    if words.is_empty() {
        bail!("session title must not be empty");
    }
    if words.len() > MAX_WORDS {
        bail!("session title must be at most {MAX_WORDS} words");
    }
    Ok(words.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_and_limits_titles_by_words() {
        assert_eq!(
            normalize("  Researching   MCP allocation ").unwrap(),
            "Researching MCP allocation"
        );
        assert!(normalize("").is_err());
        assert!(normalize(&vec!["word"; MAX_WORDS + 1].join(" ")).is_err());
    }
}
