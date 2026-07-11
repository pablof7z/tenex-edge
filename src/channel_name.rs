use anyhow::Result;

pub(crate) fn validate_child(name: &str, parent_is_workspace_root: bool) -> Result<()> {
    if parent_is_workspace_root && name.eq_ignore_ascii_case("general") {
        anyhow::bail!("general is the workspace root channel and cannot also be a direct child");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn general_is_reserved_only_directly_below_workspace_root() {
        assert!(validate_child("general", true).is_err());
        assert!(validate_child("GENERAL", true).is_err());
        assert!(validate_child("general", false).is_ok());
        assert!(validate_child("reviews", true).is_ok());
    }
}
