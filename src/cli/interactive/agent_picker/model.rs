use crate::session::Harness;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::cli) struct AgentProvenance {
    pub(in crate::cli) label: String,
    pub(in crate::cli) harness: Harness,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::cli) enum DeleteScope {
    Agent,
    Profile,
    Both,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::cli) struct AgentPickerRow {
    pub(in crate::cli) name: String,
    pub(in crate::cli) description: String,
    pub(in crate::cli) status: Option<AgentProvenance>,
    pub(in crate::cli) has_configured: bool,
    pub(in crate::cli) has_native_profile: bool,
}

impl AgentPickerRow {
    pub(super) fn source_label(&self) -> &'static str {
        if self.has_configured {
            "configured"
        } else if self.has_native_profile {
            "installed profile"
        } else {
            "built in"
        }
    }

    pub(super) fn source_short_label(&self) -> &'static str {
        if self.has_configured {
            "config"
        } else if self.has_native_profile {
            "profile"
        } else {
            "core"
        }
    }

    pub(super) fn harness_label(&self) -> &str {
        self.status
            .as_ref()
            .map(|value| value.label.as_str())
            .unwrap_or("Unknown")
    }

    pub(super) fn clean_description(&self) -> String {
        let decoded = self.description.replace("\\n", "\n").replace("\\t", " ");
        let end = ["<example>", "<examples>", "<instructions>"]
            .into_iter()
            .filter_map(|marker| decoded.find(marker))
            .min()
            .unwrap_or(decoded.len());
        let primary = &decoded[..end];
        let clean = crate::agent_about::for_injection(primary).replace(" -- ", " — ");
        if clean.is_empty() {
            format!("{} agent", self.source_label())
        } else {
            clean
        }
    }

    pub(super) fn description_summary(&self) -> String {
        let clean = self.clean_description();
        clean
            .split_once(". ")
            .map(|(first, _)| format!("{first}."))
            .unwrap_or(clean)
    }

    pub(super) fn fuzzy_score(&self, input: &str) -> Option<i64> {
        if input.is_empty() {
            return Some(0);
        }
        let matcher = SkimMatcherV2::default().ignore_case();
        [
            (self.name.as_str(), 4_000),
            (self.description.as_str(), 2_000),
            (
                self.status
                    .as_ref()
                    .map(|value| value.label.as_str())
                    .unwrap_or_default(),
                750,
            ),
        ]
        .into_iter()
        .filter_map(|(field, priority)| {
            let score = matcher.fuzzy_match(field, input)?;
            let exact = i64::from(field.to_lowercase().contains(&input.to_lowercase())) * 10_000;
            Some(score + exact + priority)
        })
        .max()
    }
}
