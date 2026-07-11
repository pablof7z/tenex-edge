use super::*;

#[test]
fn explicit_work_topic_appears_only_after_distillation_pause_expires() {
    let store = seed_store();
    let mut rec = session(&store);
    rec.work_topic = "Researching MCP improvements around resource allocation".into();
    rec.work_topic_set_at = 100;

    let paused = render_fabric_context(
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
        !paused.contains("Current visible work topic"),
        "got: {paused}"
    );

    let visible_at = 100 + crate::work_topic::DISTILL_PAUSE_SECS;
    let visible = render_fabric_context(&store, input(Some(&rec), "root", 0, visible_at, true))
        .expect("explicit context should render");
    assert!(
        visible.contains("You are @coder, running on laptop. Current visible work topic: \"Researching MCP improvements around resource allocation\""),
        "got: {visible}"
    );
    assert!(
        visible.contains("[if your work topic drifted you can update it]"),
        "got: {visible}"
    );

    let captured = capture_inputs(&store, &input(Some(&rec), "root", 0, visible_at, true));
    let reconciled = render_view_text(&assemble::assemble_view(&captured, 0, visible_at));
    assert_eq!(reconciled, visible);
}
