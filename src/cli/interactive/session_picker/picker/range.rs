use crate::cli::interactive::session_picker::data::SessionRow;

#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum HistoryRange {
    #[default]
    Live,
    Hours3,
    Hours12,
    Day1,
    Days2,
    Week1,
    Days30,
    All,
}

impl HistoryRange {
    const STEPS: [Self; 8] = [
        Self::Live,
        Self::Hours3,
        Self::Hours12,
        Self::Day1,
        Self::Days2,
        Self::Week1,
        Self::Days30,
        Self::All,
    ];

    pub(super) fn expand(&mut self) {
        let index = Self::STEPS
            .iter()
            .position(|step| step == self)
            .unwrap_or(0);
        *self = Self::STEPS[(index + 1).min(Self::STEPS.len() - 1)];
    }

    pub(super) fn narrow(&mut self) {
        let index = Self::STEPS
            .iter()
            .position(|step| step == self)
            .unwrap_or(0);
        *self = Self::STEPS[index.saturating_sub(1)];
    }

    pub(super) fn includes(self, row: &SessionRow, now: u64) -> bool {
        if row.running {
            return true;
        }
        match self.max_age_secs() {
            None => self == Self::All,
            Some(max_age) => row.last_seen > 0 && now.saturating_sub(row.last_seen) <= max_age,
        }
    }

    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Live => "Live",
            Self::Hours3 => "3h",
            Self::Hours12 => "12h",
            Self::Day1 => "1d",
            Self::Days2 => "2d",
            Self::Week1 => "1w",
            Self::Days30 => "30d",
            Self::All => "All",
        }
    }

    fn max_age_secs(self) -> Option<u64> {
        const HOUR: u64 = 60 * 60;
        const DAY: u64 = 24 * HOUR;
        match self {
            Self::Live | Self::All => None,
            Self::Hours3 => Some(3 * HOUR),
            Self::Hours12 => Some(12 * HOUR),
            Self::Day1 => Some(DAY),
            Self::Days2 => Some(2 * DAY),
            Self::Week1 => Some(7 * DAY),
            Self::Days30 => Some(30 * DAY),
        }
    }
}
