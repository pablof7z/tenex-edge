use anyhow::Result;

pub(crate) fn validate_child(name: &str, workspace_root: Option<&str>) -> Result<()> {
    if name.trim().is_empty() || name.contains(['.', '/']) {
        anyhow::bail!("channel names must be one non-empty dotted-path segment");
    }
    if workspace_root.is_some_and(|root| name.eq_ignore_ascii_case(root)) {
        anyhow::bail!(
            "{name} is already the workspace root channel and cannot also be a direct child"
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_name_is_reserved_only_directly_below_its_root() {
        assert!(validate_child("mosaico", Some("mosaico")).is_err());
        assert!(validate_child("MOSAICO", Some("mosaico")).is_err());
        assert!(validate_child("mosaico", None).is_ok());
        assert!(validate_child("reviews", Some("mosaico")).is_ok());
        assert!(validate_child("epic/reviews", Some("mosaico")).is_err());
        assert!(validate_child("epic.reviews", Some("mosaico")).is_err());
    }
}
