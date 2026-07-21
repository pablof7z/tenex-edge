use super::*;

fn auth() -> AuthState {
    AuthState {
        public_url: "https://mosaico.example".into(),
        resource_url: "https://mosaico.example/mcp".into(),
        secret: b"test-secret".to_vec(),
        whitelisted: Arc::new(Vec::new()),
        codes: Arc::new(Mutex::new(HashMap::new())),
        challenges: Arc::new(Mutex::new(HashMap::new())),
    }
}

#[test]
fn actor_correlation_is_keyed_stable_and_contains_no_raw_identifiers() {
    let auth = auth();
    let first = auth.redact_actor_key(&["openai-v1", "subject", "conversation-one"]);
    let repeat = auth.redact_actor_key(&["openai-v1", "subject", "conversation-one"]);
    let second = auth.redact_actor_key(&["openai-v1", "subject", "conversation-two"]);
    assert_eq!(first, repeat);
    assert_ne!(first, second);
    assert!(!first.contains("subject"));
    assert!(!first.contains("conversation"));
}

#[test]
fn metadata_separates_authorization_server_from_mcp_resource() {
    let auth = auth();
    assert_eq!(
        auth.protected_resource(),
        json!({
            "resource": "https://mosaico.example/mcp",
            "authorization_servers": ["https://mosaico.example"],
            "scopes_supported": SCOPES,
            "resource_documentation": "https://github.com/pablof7z/mosaico",
        })
    );
    assert_eq!(
        auth.authorization_server()["issuer"],
        "https://mosaico.example"
    );
}

#[test]
fn tokens_are_audience_bound_to_mcp_resource() {
    let auth = auth();
    let code = AuthCode {
        client_id: "client".into(),
        redirect_uri: "https://client.example/callback".into(),
        code_challenge: "challenge".into(),
        resource: auth.resource_url.clone(),
        scope: default_scope(),
        pubkey: "pubkey".into(),
        expires_at: crate::util::now_secs() + 60,
    };
    let token = auth.issue_token(&code).expect("issue token");
    let claims = auth.verify_token(&token).expect("verify token");
    assert_eq!(claims.iss, "https://mosaico.example");
    assert_eq!(claims.aud, "https://mosaico.example/mcp");

    let origin_token = auth
        .issue_token(&AuthCode {
            resource: auth.public_url.clone(),
            ..code
        })
        .expect("issue origin-audience token");
    assert!(auth.verify_token(&origin_token).is_err());
}

#[test]
fn tokens_do_not_expire() {
    let auth = auth();
    let code = AuthCode {
        client_id: "client".into(),
        redirect_uri: "https://client.example/callback".into(),
        code_challenge: "challenge".into(),
        resource: auth.resource_url.clone(),
        scope: default_scope(),
        pubkey: "pubkey".into(),
        expires_at: crate::util::now_secs() + 60,
    };
    let token = auth.issue_token(&code).expect("issue token");
    // The signed payload must carry no expiry claim.
    let payload = token.split('.').nth(1).expect("token payload segment");
    let decoded = URL_SAFE_NO_PAD.decode(payload).expect("decode payload");
    let claims: Value = serde_json::from_slice(&decoded).expect("parse claims");
    assert!(claims.get("exp").is_none(), "tokens must not carry exp");
    // And verification never rejects on time.
    auth.verify_token(&token)
        .expect("non-expiring token verifies");
}
