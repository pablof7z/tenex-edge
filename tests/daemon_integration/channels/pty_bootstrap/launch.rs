use super::*;

fn launch_no_hook(home: &Home, agent: &str, channel: &str, command: &str) {
    let work_dir = home.dir.path().join(channel);
    add_workspace_mapping(home, channel, &work_dir);
    let out = run_cli_with_env_in_dir(
        home,
        &[
            "launch",
            agent,
            "--workspace",
            channel,
            "--command",
            command,
        ],
        &[],
        &work_dir,
    );
    assert!(
        out.status.success(),
        "launch failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn launch_command_bootstraps_session_without_child_session_start_hook() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let home = Home::new();
    write_config(&home, false);

    let channel = unique_session("launch-bootstrap");
    let agent = "launch-no-hook-agent";
    launch_no_hook(
        &home,
        agent,
        &channel,
        "sh -lc 'while true; do sleep 1; done'",
    );

    let rec = wait_for_alive(&home, agent, &channel);
    let pty_id = Store::open(&home.store_path())
        .unwrap()
        .aliases_for_session(&rec.session_id)
        .unwrap()
        .into_iter()
        .find(|alias| alias.external_id_kind == "pty_session")
        .map(|alias| alias.external_id)
        .expect("launch must register its PTY alias before returning");

    let _ = tenex_edge::pty::kill(&pty_id);
    stop_daemon(&home);
}

#[test]
fn supervisor_exit_retires_the_bootstrapped_session() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let home = Home::new();
    write_config(&home, false);

    let channel = unique_session("launch-exit");
    let agent = "launch-exit-agent";
    launch_no_hook(&home, agent, &channel, "sh -lc 'sleep 1'");
    let rec = wait_for_alive(&home, agent, &channel);

    assert!(
        wait_until(Duration::from_secs(10), || {
            Store::open(&home.store_path())
                .and_then(|store| store.get_session(&rec.session_id))
                .ok()
                .flatten()
                .is_some_and(|session| !session.alive)
        }),
        "supervisor exit did not retire session {}",
        rec.session_id
    );
    stop_daemon(&home);
}
