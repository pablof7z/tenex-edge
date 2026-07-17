use std::process::Command;

fn contextual_help(args: &[&str], agent: bool) -> String {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mosaico"));
    command.args(args);
    if agent {
        command.env("MOSAICO_AGENT", "test-agent");
    } else {
        command.env_remove("MOSAICO_AGENT");
    }
    let output = command.output().expect("run mosaico help");

    assert!(output.status.success(), "help failed: {output:?}");
    String::from_utf8(output.stdout).expect("help is UTF-8")
}

#[test]
fn bare_invocation_matches_top_level_human_help() {
    let bare = contextual_help(&[], false);
    let explicit = contextual_help(&["--help"], false);

    assert_eq!(bare, explicit);
    assert!(bare.contains("  sessions"));
    assert!(bare.contains("  agents"));
    assert!(!bare.contains("  mgmt"));
    assert!(!bare.contains("  publish"));
}

#[test]
fn agent_help_hides_operator_agent_management() {
    let help = contextual_help(&["--help"], true);

    assert!(help.contains("  my"));
    assert!(!help.contains("  agents"));
    assert!(!help.contains("  mgmt"));
}
