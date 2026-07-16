//! The plain receipt the daemon persists for the `explain` CLI. It is built
//! from the same inputs and rendered text as the snapshot, so explanation and
//! output cannot drift apart.

/// The full-vs-delta shape, decided purely by the `cursor` input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    /// `cursor == 0`: full members and visible descendant tree.
    Full,
    /// `cursor > 0`: changed descendants and recent presence only.
    Delta,
}

impl Shape {
    fn from_cursor(cursor: i64) -> Self {
        if cursor == 0 {
            Shape::Full
        } else {
            Shape::Delta
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Shape::Full => "full",
            Shape::Delta => "delta",
        }
    }
}

/// Whether this render established a baseline, changed it, or left it intact.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameKind {
    /// First snapshot for this session.
    Baseline,
    /// A change delta over the previous snapshot.
    Delta,
    /// Nothing was emitted (inputs unchanged).
    Unchanged,
}

impl FrameKind {
    fn as_str(self) -> &'static str {
        match self {
            FrameKind::Baseline => "baseline",
            FrameKind::Delta => "delta",
            FrameKind::Unchanged => "unchanged",
        }
    }
}

/// Receipt for one hook-context render. Everything here is calculated during
/// the render itself, so it can answer why the snapshot changed without a
/// second derivation that could diverge from the rendered bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HookContextReceipt {
    /// Instrumentation surface tag (`hook_context`).
    pub surface: &'static str,
    /// Hook kind label the daemon assigns (`turn_start` / `turn_check`).
    pub kind: String,
    /// The session this snapshot was rendered for.
    pub pubkey: String,
    /// The explicit seen-cursor input.
    pub cursor: i64,
    /// The explicit wall-clock input.
    pub now: i64,
    /// Full vs delta, attributable to the `cursor` input.
    pub shape: Shape,
    /// The relationship between this render and the previous snapshot.
    pub frame: FrameKind,
    /// Whether a snapshot was injected (non-empty and/or forced).
    pub emitted: bool,
    /// Rendered snapshot byte length (0 when suppressed).
    pub bytes: usize,
    /// Changed input labels (e.g. `presence`, `cursor`, `now`) that explain why
    /// the snapshot changed.
    pub input_causes: Vec<String>,
}

impl HookContextReceipt {
    pub(crate) fn new(
        kind: &str,
        pubkey: &str,
        cursor: i64,
        now: i64,
        frame: FrameKind,
        text: Option<&str>,
        input_causes: Vec<String>,
    ) -> Self {
        Self {
            surface: "hook_context",
            kind: kind.to_string(),
            pubkey: pubkey.to_string(),
            cursor,
            now,
            shape: Shape::from_cursor(cursor),
            frame,
            emitted: text.is_some(),
            bytes: text.map(str::len).unwrap_or(0),
            input_causes,
        }
    }

    /// Serialize for the hook-call log / `receipts` store. Carries `kind` so the
    /// debug loader keeps labelling turn_start / turn_check calls.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "surface": self.surface,
            "kind": self.kind,
            "pubkey": self.pubkey,
            "cursor": self.cursor,
            "now": self.now,
            "shape": self.shape.as_str(),
            "frame": self.frame.as_str(),
            "why_shape": format!(
                "cursor {} 0 → {} render",
                if self.cursor == 0 { "==" } else { ">" },
                self.shape.as_str()
            ),
            "input_causes": self.input_causes,
            "output": {
                "emitted": self.emitted,
                "bytes": self.bytes,
            },
        })
    }
}
