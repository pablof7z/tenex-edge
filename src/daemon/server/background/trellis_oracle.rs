//! Sampled Trellis oracle task. It checks live graph snapshots periodically and
//! stamps the latest all-commit ledger row per covered surface with the outcome.

use super::super::*;

const SAMPLE_EVERY: Duration = Duration::from_secs(60);

pub fn spawn_trellis_oracle_sampler(state: Arc<DaemonState>) {
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(SAMPLE_EVERY);
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            tick.tick().await;
            let report = probe::oracle_report(&state);
            state.with_store(|s| {
                for surface in &report.surfaces {
                    if let Err(e) = s.record_oracle_sample(
                        surface.surface,
                        surface.status,
                        surface.error.as_deref(),
                    ) {
                        tracing::warn!(
                            surface = surface.surface,
                            error = %e,
                            "trellis oracle sample was not recorded"
                        );
                    }
                }
            });
        }
    });
}
