use super::*;

#[test]
fn explicit_work_topic_appears_immediately_as_current_title() {
    let store = seed_store();
    let mut rec = session(&store);
    rec.work_topic = "Researching MCP improvements around resource allocation".into();
    rec.work_topic_set_at = 100;

    let visible = render_fabric_context(
        &store,
        input(
            Some(&rec),
            "root",
            0,
            100 + crate::work_topic::DISTILL_PAUSE_SECS - 1,
            true,
        ),
    )
    .expect("explicit context should render");
    assert!(
        visible.contains("Agent: coder · Session: @coder · Backend: laptop\n  Current title: \"Researching MCP improvements around resource allocation\""),
        "got: {visible}"
    );
    assert!(
        visible.contains("[if your title drifted you can update it]"),
        "got: {visible}"
    );

    let captured = capture_inputs(
        &store,
        &input(
            Some(&rec),
            "root",
            0,
            100 + crate::work_topic::DISTILL_PAUSE_SECS - 1,
            true,
        ),
    );
    let reconciled = render_view_text(&assemble::assemble_view(
        &captured,
        0,
        100 + crate::work_topic::DISTILL_PAUSE_SECS - 1,
    ));
    assert_eq!(reconciled, visible);
}
