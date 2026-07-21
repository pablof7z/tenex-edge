use super::append_command_args;

#[test]
fn extra_args_reach_pty_and_rpc_commands() {
    let mut command = vec!["codex".to_string(), "app-server".to_string()];
    let mut rpc_argv = command.clone();
    let extra_args = vec!["--yolo".to_string(), "--model=x".to_string()];

    append_command_args(&mut command, Some(&mut rpc_argv), &extra_args);

    let expected = ["codex", "app-server", "--yolo", "--model=x"];
    assert_eq!(command, expected);
    assert_eq!(rpc_argv, expected);
}
