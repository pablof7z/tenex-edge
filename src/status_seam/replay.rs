use super::*;

pub(super) fn status_replay_seed_pubkey(fact: &InputFact) -> Option<&str> {
    match fact {
        InputFact::StatusDrive(StatusDrive::SessionStarted(_)) => None,
        InputFact::StatusDrive(StatusDrive::TurnStarted { pubkey, .. })
        | InputFact::StatusDrive(StatusDrive::TurnEnded { pubkey, .. })
        | InputFact::StatusDrive(StatusDrive::TitleSet { pubkey, .. })
        | InputFact::StatusDrive(StatusDrive::ChannelsChanged { pubkey, .. })
        | InputFact::StatusDrive(StatusDrive::Tick { pubkey, .. })
        | InputFact::StatusDrive(StatusDrive::SessionEnded { pubkey, .. })
        | InputFact::StatusDrive(StatusDrive::SessionRevoked { pubkey, .. }) => Some(pubkey),
        _ => None,
    }
}
