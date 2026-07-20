#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PickerView {
    Inspector,
    Briefs,
    Index,
}

impl PickerView {
    pub(super) fn number(self) -> u8 {
        match self {
            Self::Inspector => 1,
            Self::Briefs => 2,
            Self::Index => 3,
        }
    }

    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Inspector => "Inspector",
            Self::Briefs => "Briefs",
            Self::Index => "Index",
        }
    }
}
