use super::*;

pub(super) fn status_replay_seed_session_id(fact: &InputFact) -> Option<&str> {
    match fact {
        InputFact::StatusDrive(StatusDrive::SessionStarted(_)) => None,
        InputFact::StatusDrive(StatusDrive::TurnStarted { session_id, .. })
        | InputFact::StatusDrive(StatusDrive::TurnEnded { session_id, .. })
        | InputFact::StatusDrive(StatusDrive::DistillCompleted { session_id, .. })
        | InputFact::StatusDrive(StatusDrive::TitleSet { session_id, .. })
        | InputFact::StatusDrive(StatusDrive::ChannelsChanged { session_id, .. })
        | InputFact::StatusDrive(StatusDrive::Tick { session_id, .. })
        | InputFact::StatusDrive(StatusDrive::SessionEnded { session_id, .. })
        | InputFact::StatusDrive(StatusDrive::SessionRevoked { session_id, .. }) => {
            Some(session_id)
        }
        _ => None,
    }
}
