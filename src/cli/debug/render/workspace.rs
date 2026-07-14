use super::*;

pub(super) fn pane_title(pane: &SessionPane) -> Line<'static> {
    let session = if pane.agent.is_empty() {
        pane.short.as_str()
    } else {
        pane.agent.as_str()
    };
    let channels = if pane.channels.is_empty() {
        pane.root.clone()
    } else {
        pane.channels.join(", ")
    };
    Line::from(vec![
        Span::raw(format!("{session} / ")),
        Span::styled(
            pane.root.clone(),
            Style::default().fg(crate::console_style::workspace_ratatui_color(&pane.root)),
        ),
        Span::raw(format!(" / {channels}")),
    ])
}
