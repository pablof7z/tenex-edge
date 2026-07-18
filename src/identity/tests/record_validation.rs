use super::*;

#[test]
fn loading_pre_keyless_agent_migrates_redundant_keys_atomically() {
    let dir = tempfile::tempdir().unwrap();
    load_or_create(dir.path(), "coder", "codex", None, 1).unwrap();
    let path = dir.path().join("agents/coder.json");
    let keys = Keys::generate();
    let mut config: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    config["secret_key"] = serde_json::json!(keys.secret_key().to_secret_hex());
    config["public_key"] = serde_json::json!(keys.public_key().to_hex());
    std::fs::write(&path, serde_json::to_string_pretty(&config).unwrap()).unwrap();

    let loaded = load(dir.path(), "coder").unwrap();
    assert!(loaded.per_session_key);
    assert!(loaded.keys.is_none());
    let migrated: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert!(migrated.get("secret_key").is_none());
    assert!(migrated.get("public_key").is_none());
}

#[test]
fn loading_agent_requires_explicit_identity_mode() {
    let dir = tempfile::tempdir().unwrap();
    load_or_create(dir.path(), "coder", "codex", None, 1).unwrap();
    let path = dir.path().join("agents/coder.json");
    let mut config: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    config.as_object_mut().unwrap().remove("perSessionKey");
    std::fs::write(&path, serde_json::to_string_pretty(&config).unwrap()).unwrap();

    let error = load(dir.path(), "coder").unwrap_err();
    assert!(
        format!("{error:#}").contains("missing field `perSessionKey`"),
        "unexpected error: {error:#}"
    );
}
