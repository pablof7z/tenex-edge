use trellis_core::ResourceCommand;

pub(crate) fn command_plans_match<C: PartialEq>(
    preview: &[ResourceCommand<C>],
    committed: &[ResourceCommand<C>],
) -> bool {
    preview.len() == committed.len()
        && preview
            .iter()
            .zip(committed)
            .all(|(left, right)| command_matches(left, right))
}

fn command_matches<C: PartialEq>(
    preview: &ResourceCommand<C>,
    committed: &ResourceCommand<C>,
) -> bool {
    match (preview, committed) {
        (
            ResourceCommand::Open {
                key: left_key,
                command: left_command,
                ..
            },
            ResourceCommand::Open {
                key: right_key,
                command: right_command,
                ..
            },
        )
        | (
            ResourceCommand::Replace {
                key: left_key,
                command: left_command,
                ..
            },
            ResourceCommand::Replace {
                key: right_key,
                command: right_command,
                ..
            },
        )
        | (
            ResourceCommand::Refresh {
                key: left_key,
                command: left_command,
                ..
            },
            ResourceCommand::Refresh {
                key: right_key,
                command: right_command,
                ..
            },
        ) => left_key == right_key && left_command == right_command,
        (
            ResourceCommand::Close { key: left_key, .. },
            ResourceCommand::Close { key: right_key, .. },
        ) => left_key == right_key,
        _ => false,
    }
}
