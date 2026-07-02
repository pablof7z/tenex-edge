use nostr_sdk::prelude::*;
use regex::Regex;
use std::sync::OnceLock;

static SCRUB_PATTERNS: OnceLock<Vec<Regex>> = OnceLock::new();

/// Credential-detection regexes, ordered most-specific first. A pattern that
/// fails to compile silently disables scrubbing for that credential class (a
/// fail-open leak risk), so the `all_scrub_patterns_compile` test asserts every
/// source here compiles — a broken pattern fails the build loudly rather than
/// at runtime.
const SECRET_PATTERN_SOURCES: &[&str] = &[
    // AWS access key IDs
    r"(?:AKIA|ASIA|AGPA|AIDA|AROA|AIPA|ANPA|ANVA|APKA)[0-9A-Z]{16}",
    // GitHub tokens (classic PATs, fine-grained, OAuth, runner, server)
    r"gh[pousr]_[A-Za-z0-9]{36,255}",
    // Slack tokens
    r"xox[baprs]-[A-Za-z0-9\-]{10,255}",
    // Google API keys
    r"AIza[0-9A-Za-z\-_]{35}",
    // Anthropic API keys
    r"sk-ant-[A-Za-z0-9\-_]{20,255}",
    // OpenAI / generic sk- keys (after more-specific patterns)
    r"sk-[A-Za-z0-9]{20,255}",
    // Nostr secret keys (bech32 nsec)
    r"nsec1[a-z0-9]{58}",
    // Ollama cloud API keys: <32-hex>.<20-64 alphanumeric>
    r"[a-f0-9]{32}\.[A-Za-z0-9]{20,64}",
    // PEM private key blocks
    r"-----BEGIN (?:RSA |EC |OPENSSH |PGP |DSA )?PRIVATE KEY-----",
];

fn secret_patterns() -> &'static Vec<Regex> {
    SCRUB_PATTERNS.get_or_init(|| {
        SECRET_PATTERN_SOURCES
            .iter()
            .filter_map(|p| match Regex::new(p) {
                Ok(re) => Some(re),
                Err(e) => {
                    // Runtime stays safe (we skip the broken pattern), but the
                    // `all_scrub_patterns_compile` test guarantees we never ship one.
                    eprintln!("[tenex-edge] scrub: failed to compile pattern {p:?}: {e}");
                    None
                }
            })
            .collect()
    })
}

/// Replace detected credential spans with `[REDACTED]`. Fail-open: always
/// returns a valid string; compile errors on individual patterns are skipped.
fn scrub_secrets(input: &str) -> String {
    let mut out = input.to_string();
    for re in secret_patterns() {
        let replaced = re.replace_all(&out, "[REDACTED]");
        if let std::borrow::Cow::Owned(s) = replaced {
            out = s;
        }
    }
    out
}

/// Scrub content in-place on an `UnsignedEvent`. Resets `id` to `None` when
/// content changed so the signing step recomputes the event ID over the
/// scrubbed content (required — nostr-sdk validates id vs content on sign).
pub(super) fn scrub_unsigned(unsigned: &mut UnsignedEvent) {
    if unsigned.content.is_empty() {
        return;
    }
    let scrubbed = scrub_secrets(&unsigned.content);
    if scrubbed != unsigned.content {
        eprintln!(
            "[tenex-edge] redacted secret(s) from outgoing kind:{} event",
            unsigned.kind.as_u16()
        );
        unsigned.content = scrubbed;
        unsigned.id = None; // force id recompute over scrubbed content at sign time
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Fail-loud guard: every credential pattern MUST compile. At runtime a
    /// broken pattern is silently skipped (fail-open), so this build-time
    /// assertion is the only thing standing between a typo and a disabled
    /// credential class.
    #[test]
    fn all_scrub_patterns_compile() {
        for p in SECRET_PATTERN_SOURCES {
            assert!(
                Regex::new(p).is_ok(),
                "scrub pattern failed to compile: {p:?}"
            );
        }
        // And the live set actually built one regex per source (no silent drops).
        assert_eq!(secret_patterns().len(), SECRET_PATTERN_SOURCES.len());
    }

    #[test]
    fn redacts_aws_key() {
        let input = "my key is AKIAIOSFODNN7EXAMPLE right there";
        let out = scrub_secrets(input);
        assert!(out.contains("[REDACTED]"), "should redact AWS key");
        assert!(!out.contains("AKIAIOSFODNN7EXAMPLE"), "should not leak key");
    }

    #[test]
    fn redacts_github_pat() {
        let token = "ghp_0123456789abcdefghijklmnopqrstuvwxyz";
        let input = format!("use this token: {token}");
        let out = scrub_secrets(&input);
        assert!(out.contains("[REDACTED]"));
        assert!(!out.contains(token));
    }

    #[test]
    fn redacts_anthropic_key() {
        let key = "sk-ant-api03-abc123def456ghi789jkl012mno345pqr";
        let out = scrub_secrets(key);
        assert!(out.contains("[REDACTED]"));
        assert!(!out.contains("sk-ant-"));
    }

    #[test]
    fn redacts_openai_key() {
        let key = "sk-abcdefghijklmnopqrstuvwxyz123456";
        let out = scrub_secrets(key);
        assert!(out.contains("[REDACTED]"));
        assert!(!out.contains(key));
    }

    #[test]
    fn redacts_nsec() {
        // bech32-encoded nostr secret key: "nsec1" + 58 lowercase alphanumeric chars
        let nsec = format!("nsec1{}", "a".repeat(58));
        let input = format!("my key: {nsec}");
        let out = scrub_secrets(&input);
        assert!(out.contains("[REDACTED]"), "should redact nsec; got: {out}");
        assert!(!out.contains("nsec1"), "should not leak nsec prefix");
    }

    #[test]
    fn redacts_ollama_key() {
        let key = "e1a1e08cbfbc4bbf9b702162cbdbd0f6.qQmkiYini5T9hrtMgiYalWb2";
        let input = format!("my ollama key is {key}");
        let out = scrub_secrets(&input);
        assert!(
            out.contains("[REDACTED]"),
            "should redact ollama key; got: {out}"
        );
        assert!(!out.contains(key));
    }

    #[test]
    fn does_not_redact_plain_text() {
        let input = "please refactor the akimbo module and fix the bug";
        let out = scrub_secrets(input);
        assert_eq!(out, input, "ordinary text must pass through unchanged");
    }

    #[test]
    fn empty_content_unchanged() {
        assert_eq!(scrub_secrets(""), "");
    }

    #[tokio::test]
    async fn scrub_unsigned_then_sign_verifies() {
        let keys = Keys::generate();
        let token = "ghp_0123456789abcdefghijklmnopqrstuvwxyz";
        let raw_content = format!("leak {token} in prompt");

        let builder = EventBuilder::new(Kind::from(1u16), raw_content);
        let mut unsigned = builder.build(keys.public_key());
        scrub_unsigned(&mut unsigned);

        assert!(unsigned.content.contains("[REDACTED]"), "content scrubbed");
        assert!(!unsigned.content.contains(token), "token absent");

        // Sign and verify: nostr-sdk recomputes the id from scrubbed content
        // on sign, so verify() must succeed (proves id reset was applied).
        let signed = keys.sign_event(unsigned).await.expect("signing");
        assert!(
            signed.verify().is_ok(),
            "signature valid over scrubbed content"
        );
        assert!(
            !signed.content.contains(token),
            "token absent from signed event"
        );
    }

    #[tokio::test]
    async fn scrub_unsigned_noop_on_empty_content() {
        let keys = Keys::generate();
        // Kind 9000 put-user events have empty content; it must be untouched.
        let builder = EventBuilder::new(Kind::from(9000u16), "");
        let mut unsigned = builder.build(keys.public_key());
        let original_id = unsigned.id;
        scrub_unsigned(&mut unsigned);
        assert_eq!(
            unsigned.id, original_id,
            "id must not be reset when content is empty"
        );
    }
}
