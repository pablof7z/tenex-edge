use super::clients::{snapshot, Clients};
use crate::pty::PresentationSnapshot;

pub(super) struct SessionExitGuard {
    pty_id: String,
    clients: Clients,
    child_success: Option<bool>,
    child_exit_code: Option<u32>,
    presentation: Option<PresentationSnapshot>,
}

impl SessionExitGuard {
    pub(super) fn new(pty_id: String, clients: Clients) -> Self {
        Self {
            pty_id,
            clients,
            child_success: None,
            child_exit_code: None,
            presentation: None,
        }
    }

    pub(super) fn record_child_exit(
        &mut self,
        status: &portable_pty::ExitStatus,
        presentation: PresentationSnapshot,
    ) {
        self.child_success = Some(status.success());
        self.child_exit_code = Some(status.exit_code());
        self.presentation = Some(presentation);
    }
}

impl Drop for SessionExitGuard {
    fn drop(&mut self) {
        let report = crate::pty::SupervisorExitReport {
            pty_id: self.pty_id.clone(),
            child_success: self.child_success,
            child_exit_code: self.child_exit_code,
            presentation: self.presentation.unwrap_or_else(|| snapshot(&self.clients)),
            recorded_at: crate::util::now_secs(),
        };
        if let Err(error) = crate::pty::persist_exit_report(&report) {
            eprintln!("[mosaico pty supervisor] could not persist exit report: {error:#}");
        }
        notify_daemon(report);
    }
}

fn notify_daemon(report: crate::pty::SupervisorExitReport) {
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
            if client
                .call("pty_supervisor_exit", notification_params(&report))
                .await
                .is_ok_and(|result| result["accepted"].as_bool() == Some(true))
            {
                crate::pty::remove_exit_report(&report.pty_id);
            }
        });
    })
    .join()
    .ok();
}

fn notification_params(report: &crate::pty::SupervisorExitReport) -> serde_json::Value {
    serde_json::to_value(report).expect("serializing supervisor exit report")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_notification_carries_child_and_presentation_facts() {
        let params = notification_params(&crate::pty::SupervisorExitReport {
            pty_id: "pty-1".into(),
            child_success: Some(false),
            child_exit_code: Some(17),
            presentation: PresentationSnapshot {
                attached_clients: 2,
                attachment_epoch: 9,
                changed_at: 8,
            },
            recorded_at: 10,
        });
        assert_eq!(params["pty_id"], "pty-1");
        assert_eq!(params["child_success"], false);
        assert_eq!(params["child_exit_code"], 17);
        assert_eq!(params["presentation"]["attached_clients"], 2);
        assert_eq!(params["presentation"]["attachment_epoch"], 9);
        assert_eq!(params["presentation"]["changed_at"], 8);
    }
}
