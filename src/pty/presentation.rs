//! Typed presentation-state protocol for a live PTY supervisor.

use super::meta;
use serde::de::DeserializeOwned;
use std::fmt;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PresentationSnapshot {
    pub(crate) attached_clients: u64,
    pub(crate) attachment_epoch: u64,
    pub(crate) changed_at: u64,
}

impl PresentationSnapshot {
    pub(crate) fn is_headless(self) -> bool {
        self.attached_clients == 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PresentationObservation {
    Managed(PresentationSnapshot),
    AdoptedPreUpgrade { headless: bool },
}

impl PresentationObservation {
    pub(crate) fn is_headless(self) -> bool {
        match self {
            Self::Managed(snapshot) => snapshot.is_headless(),
            Self::AdoptedPreUpgrade { headless } => headless,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum ConditionalKillOutcome {
    Killed { presentation: PresentationSnapshot },
    PresentationChanged { presentation: PresentationSnapshot },
}

#[derive(Debug)]
pub(crate) struct PresentationUnavailable {
    detail: String,
}

impl PresentationUnavailable {
    fn new(detail: impl Into<String>) -> Self {
        Self {
            detail: detail.into(),
        }
    }
}

impl fmt::Display for PresentationUnavailable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PTY presentation unavailable: {}", self.detail)
    }
}

impl std::error::Error for PresentationUnavailable {}

/// Return the supervisor's current presentation state. Failure is distinct from
/// a reachable supervisor reporting zero attached clients.
pub(crate) fn presentation_snapshot(
    id_or_path: &str,
) -> Result<PresentationSnapshot, PresentationUnavailable> {
    request(id_or_path, "PRESENTATION\n", Duration::from_millis(100))
}

pub(crate) fn presentation_observation(
    id_or_path: &str,
) -> Result<PresentationObservation, PresentationUnavailable> {
    match presentation_snapshot(id_or_path) {
        Ok(snapshot) => Ok(PresentationObservation::Managed(snapshot)),
        Err(managed_error) if is_pre_token_endpoint(id_or_path) => {
            let headless = pre_upgrade_headless(id_or_path).map_err(|legacy_error| {
                PresentationUnavailable::new(format!(
                    "managed probe failed ({managed_error}); pre-upgrade adoption failed ({legacy_error})"
                ))
            })?;
            super::meta::adopt_pre_token_supervisor(id_or_path).map_err(|error| {
                PresentationUnavailable::new(format!(
                    "pre-upgrade ownership adoption failed: {error:#}"
                ))
            })?;
            Ok(PresentationObservation::AdoptedPreUpgrade { headless })
        }
        Err(error) => Err(error),
    }
}

/// Kill only if the supervisor still has no attached clients at `expected_epoch`.
/// The supervisor evaluates the predicate and confirms child exit while holding
/// the same attachment-state lock used by attach and detach transitions.
pub(crate) fn kill_if_headless_at(
    id_or_path: &str,
    expected_epoch: u64,
) -> Result<ConditionalKillOutcome, PresentationUnavailable> {
    match request(
        id_or_path,
        &format!("KILL_IF_HEADLESS {expected_epoch}\n"),
        Duration::from_secs(4),
    ) {
        Ok(outcome) => Ok(outcome),
        Err(_) if is_pre_token_endpoint(id_or_path) => {
            kill_pre_upgrade_if_headless(id_or_path, expected_epoch)
        }
        Err(error) => Err(error),
    }
}

fn is_pre_token_endpoint(id: &str) -> bool {
    super::meta::read_all_metadata()
        .into_iter()
        .find(|metadata| metadata.id == id)
        .is_some_and(|metadata| metadata.instance_token.is_empty() && !metadata.socket.is_empty())
}

fn pre_upgrade_headless(id_or_path: &str) -> Result<bool, PresentationUnavailable> {
    let response: String = request_line(id_or_path, "OUTPUT_MODE\n", Duration::from_millis(100))?;
    match response.trim() {
        "headless" => Ok(true),
        "headed" => Ok(false),
        other => Err(PresentationUnavailable::new(format!(
            "invalid pre-upgrade presentation {other:?}"
        ))),
    }
}

fn kill_pre_upgrade_if_headless(
    id: &str,
    expected_epoch: u64,
) -> Result<ConditionalKillOutcome, PresentationUnavailable> {
    super::meta::adopt_pre_token_supervisor(id).map_err(|error| {
        PresentationUnavailable::new(format!("pre-upgrade ownership adoption failed: {error:#}"))
    })?;
    let original = meta::resolve_socket(id);
    let quarantine = original.with_extension(format!("adopting-{}", std::process::id()));
    std::fs::rename(&original, &quarantine).map_err(|error| {
        PresentationUnavailable::new(format!("quarantining pre-upgrade socket failed: {error}"))
    })?;
    let restore = || {
        std::fs::rename(&quarantine, &original).map_err(|error| {
            PresentationUnavailable::new(format!("restoring pre-upgrade socket failed: {error}"))
        })
    };
    if !pre_upgrade_headless(quarantine.to_string_lossy().as_ref())? {
        restore()?;
        return Ok(ConditionalKillOutcome::PresentationChanged {
            presentation: PresentationSnapshot {
                attached_clients: 1,
                attachment_epoch: expected_epoch.saturating_add(1),
                changed_at: crate::util::now_secs(),
            },
        });
    }
    let kill_result = send_without_response(quarantine.to_string_lossy().as_ref(), "KILL\n");
    if let Err(error) = kill_result {
        let _ = restore();
        return Err(error);
    }
    for _ in 0..20 {
        if UnixStream::connect(&quarantine).is_err() {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    if UnixStream::connect(&quarantine).is_ok() {
        super::meta::terminate_owned_supervisor(id).map_err(|error| {
            PresentationUnavailable::new(format!("pre-upgrade termination failed: {error:#}"))
        })?;
    }
    let _ = std::fs::remove_file(&quarantine);
    Ok(ConditionalKillOutcome::Killed {
        presentation: PresentationSnapshot {
            attached_clients: 0,
            attachment_epoch: expected_epoch,
            changed_at: crate::util::now_secs(),
        },
    })
}

fn request<T: DeserializeOwned>(
    id_or_path: &str,
    command: &str,
    timeout: Duration,
) -> Result<T, PresentationUnavailable> {
    let path = meta::resolve_socket(id_or_path);
    let mut stream = UnixStream::connect(&path)
        .map_err(|error| PresentationUnavailable::new(format!("{}: {error}", path.display())))?;
    stream
        .write_all(command.as_bytes())
        .and_then(|_| stream.flush())
        .map_err(|error| PresentationUnavailable::new(format!("request write failed: {error}")))?;
    stream
        .set_read_timeout(Some(timeout))
        .map_err(|error| PresentationUnavailable::new(format!("read timeout failed: {error}")))?;
    let mut response = String::new();
    let read = BufReader::new(stream)
        .read_line(&mut response)
        .map_err(|error| PresentationUnavailable::new(format!("response read failed: {error}")))?;
    if read == 0 {
        return Err(PresentationUnavailable::new(
            "supervisor closed without a response",
        ));
    }
    serde_json::from_str(response.trim()).map_err(|error| {
        PresentationUnavailable::new(format!("invalid supervisor response: {error}"))
    })
}

fn request_line(
    id_or_path: &str,
    command: &str,
    timeout: Duration,
) -> Result<String, PresentationUnavailable> {
    let path = meta::resolve_socket(id_or_path);
    let mut stream = UnixStream::connect(&path)
        .map_err(|error| PresentationUnavailable::new(format!("{}: {error}", path.display())))?;
    stream
        .write_all(command.as_bytes())
        .and_then(|_| stream.flush())
        .map_err(|error| PresentationUnavailable::new(format!("request write failed: {error}")))?;
    stream
        .set_read_timeout(Some(timeout))
        .map_err(|error| PresentationUnavailable::new(format!("read timeout failed: {error}")))?;
    let mut response = String::new();
    let read = BufReader::new(stream)
        .read_line(&mut response)
        .map_err(|error| PresentationUnavailable::new(format!("response read failed: {error}")))?;
    if read == 0 {
        return Err(PresentationUnavailable::new(
            "supervisor closed without a response",
        ));
    }
    Ok(response)
}

fn send_without_response(id_or_path: &str, command: &str) -> Result<(), PresentationUnavailable> {
    let path = meta::resolve_socket(id_or_path);
    let mut stream = UnixStream::connect(&path)
        .map_err(|error| PresentationUnavailable::new(format!("{}: {error}", path.display())))?;
    stream
        .write_all(command.as_bytes())
        .and_then(|_| stream.flush())
        .map_err(|error| PresentationUnavailable::new(format!("request write failed: {error}")))
}

#[cfg(test)]
#[path = "presentation/tests.rs"]
mod tests;
