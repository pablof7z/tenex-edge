pub(super) struct DurableReservationGuard;

impl Drop for DurableReservationGuard {
    fn drop(&mut self) {
        release_durable_reservation();
    }
}

fn release_durable_reservation() {
    let Ok(reservation) = std::env::var("TENEX_EDGE_DURABLE_RESERVATION") else {
        return;
    };
    std::thread::spawn(move || {
        let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        else {
            return;
        };
        runtime.block_on(async move {
            let Ok(mut client) = crate::daemon::client::Client::connect_running().await else {
                return;
            };
            client
                .call(
                    "agent_launch_release",
                    serde_json::json!({ "durable_reservation": reservation }),
                )
                .await
                .ok();
        });
    })
    .join()
    .ok();
}
