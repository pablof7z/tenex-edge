use super::*;
use std::io::Write;

#[test]
fn last_assistant_text_returns_final_assistant_message() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("t.jsonl");
    let mut f = File::create(&p).unwrap();
    writeln!(
        f,
        r#"{{"type":"user","message":{{"role":"user","content":"summarize the outage"}}}}"#
    )
    .unwrap();
    writeln!(f, r#"{{"type":"assistant","message":{{"role":"assistant","content":[{{"type":"text","text":"first pass"}},{{"type":"tool_use","name":"Read","input":{{}}}}]}}}}"#).unwrap();
    writeln!(f, r#"{{"type":"user","message":{{"role":"user","content":[{{"type":"tool_result","content":"noise"}}]}}}}"#).unwrap();
    writeln!(f, r#"{{"type":"assistant","message":{{"role":"assistant","content":[{{"type":"text","text":"Root cause was a retry storm; backoff added."}}]}}}}"#).unwrap();

    let out = read_last_assistant_text(&p, 5000).unwrap();
    assert_eq!(out, "Root cause was a retry storm; backoff added.");
}

#[test]
fn last_assistant_text_none_without_assistant_output() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("only-user.jsonl");
    let mut f = File::create(&p).unwrap();
    writeln!(
        f,
        r#"{{"type":"user","message":{{"role":"user","content":"hello"}}}}"#
    )
    .unwrap();
    // An assistant turn that is nothing but a tool call has no text to publish.
    writeln!(f, r#"{{"type":"assistant","message":{{"role":"assistant","content":[{{"type":"tool_use","name":"Bash","input":{{}}}}]}}}}"#).unwrap();

    assert!(read_last_assistant_text(&p, 5000).is_none());
}
