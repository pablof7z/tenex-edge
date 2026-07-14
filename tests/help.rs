use std::process::Command;

fn human_help(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_mosaico"))
        .args(args)
        .env_remove("MOSAICO_AGENT")
        .output()
        .expect("run mosaico help");

    assert!(output.status.success(), "help failed: {output:?}");
    String::from_utf8(output.stdout).expect("help is UTF-8")
}

#[test]
fn bare_invocation_matches_top_level_human_help() {
    let bare = human_help(&[]);
    let explicit = human_help(&["--help"]);

    assert_eq!(bare, explicit);
    assert!(bare.contains("  sessions"));
    assert!(bare.contains("  mgmt"));
    assert!(!bare.contains("  publish"));
}
