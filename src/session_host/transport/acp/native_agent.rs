use crate::agent_catalog::NativeAgentActivation;
use crate::session::Harness;

pub(super) fn claude_selector(
    activation: Option<&NativeAgentActivation>,
    harness: Harness,
) -> Option<&str> {
    match (activation, harness) {
        (Some(NativeAgentActivation::NativeSelector { name }), Harness::ClaudeCode) => {
            Some(name.as_str())
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_claude_native_selectors_reach_adapter_metadata() {
        let activation = NativeAgentActivation::NativeSelector {
            name: "reviewer".into(),
        };
        assert_eq!(
            claude_selector(Some(&activation), Harness::ClaudeCode),
            Some("reviewer")
        );
        assert_eq!(claude_selector(Some(&activation), Harness::Opencode), None);
        assert_eq!(claude_selector(None, Harness::ClaudeCode), None);
    }
}
