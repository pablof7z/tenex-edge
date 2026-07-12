use super::*;

async fn acquire_and_release(client: &mut Client, slug: &str, context: &str) {
    let reserved = client
        .call(
            "agent_launch_preflight",
            serde_json::json!({ "agent": slug }),
        )
        .await
        .unwrap_or_else(|error| panic!("{context}: {error:#}"));
    client
        .call(
            "agent_launch_release",
            serde_json::json!({
                "durable_reservation": reserved["durable_reservation"],
            }),
        )
        .await
        .unwrap();
}

pub(super) async fn assert_supervisor_releases_reservations(
    home: &Home,
    client: &mut Client,
    slug: &str,
) {
    let running = run_cli(
        home,
        &[
            "launch",
            slug,
            "--workspace",
            "tmp",
            "--command",
            "/bin/sh -c 'sleep 2'",
        ],
    );
    assert!(
        running.status.success(),
        "{}",
        String::from_utf8_lossy(&running.stderr)
    );
    client
        .call(
            "agent_launch_preflight",
            serde_json::json!({ "agent": slug }),
        )
        .await
        .expect_err("reservation remains exclusive while no-hook child is alive");
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    acquire_and_release(
        client,
        slug,
        "normal child exit did not release reservation",
    )
    .await;

    let failed = run_cli(
        home,
        &[
            "launch",
            slug,
            "--workspace",
            "tmp",
            "--command",
            "/definitely/missing",
        ],
    );
    assert!(
        failed.status.success(),
        "supervisor process failed to start"
    );
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    acquire_and_release(
        client,
        slug,
        "failed child spawn did not release reservation",
    )
    .await;
}
