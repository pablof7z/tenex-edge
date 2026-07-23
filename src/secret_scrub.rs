//! Shared credential scrubbing for diagnostics and outbound public content.

use regex::Regex;
use std::sync::OnceLock;

static PATTERNS: OnceLock<Vec<Regex>> = OnceLock::new();

const SOURCES: &[&str] = &[
    r"(?:AKIA|ASIA|AGPA|AIDA|AROA|AIPA|ANPA|ANVA|APKA)[0-9A-Z]{16}",
    r"gh[pousr]_[A-Za-z0-9]{36,255}",
    r"xox[baprs]-[A-Za-z0-9\-]{10,255}",
    r"AIza[0-9A-Za-z\-_]{35}",
    r"sk-ant-[A-Za-z0-9\-_]{20,255}",
    r"sk-[A-Za-z0-9]{20,255}",
    r"nsec1[a-z0-9]{58}",
    r"[a-f0-9]{32}\.[A-Za-z0-9]{20,64}",
    r"-----BEGIN (?:RSA |EC |OPENSSH |PGP |DSA )?PRIVATE KEY-----",
];

fn patterns() -> &'static Vec<Regex> {
    PATTERNS.get_or_init(|| {
        SOURCES
            .iter()
            .map(|source| Regex::new(source).expect("validated credential scrub pattern"))
            .collect()
    })
}

pub(crate) fn scrub(input: &str) -> String {
    patterns().iter().fold(input.to_string(), |text, pattern| {
        pattern.replace_all(&text, "[REDACTED]").into_owned()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_pattern_compiles() {
        assert_eq!(patterns().len(), SOURCES.len());
    }

    #[test]
    fn credentials_are_removed_from_diagnostics() {
        let token = "sk-abcdefghijklmnopqrstuvwxyz123456";
        let result = scrub(&format!("provider rejected {token}"));
        assert_eq!(result, "provider rejected [REDACTED]");
    }
}
