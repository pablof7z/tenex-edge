use anyhow::{Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use url::Url;

#[derive(Clone)]
pub(super) struct AuthCode {
    pub(super) client_id: String,
    pub(super) redirect_uri: String,
    pub(super) code_challenge: String,
    pub(super) resource: String,
    pub(super) scope: String,
    pub(super) pubkey: String,
    pub(super) expires_at: u64,
}

#[derive(Deserialize, Clone)]
pub(super) struct AuthorizeParams {
    response_type: String,
    pub(super) client_id: String,
    pub(super) redirect_uri: String,
    pub(super) state: Option<String>,
    pub(super) code_challenge: String,
    code_challenge_method: String,
    resource: Option<String>,
    pub(super) scope: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct AuthorizeForm {
    pub(super) nsec: String,
    response_type: String,
    client_id: String,
    redirect_uri: String,
    state: Option<String>,
    code_challenge: String,
    code_challenge_method: String,
    resource: Option<String>,
    scope: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct TokenForm {
    pub(super) grant_type: String,
    pub(super) code: String,
    pub(super) redirect_uri: String,
    pub(super) client_id: String,
    code_verifier: String,
    resource: Option<String>,
}

impl AuthorizeParams {
    pub(super) fn validate(&self, default_resource: &str) -> Result<()> {
        anyhow::ensure!(self.response_type == "code", "response_type must be code");
        anyhow::ensure!(
            self.code_challenge_method == "S256",
            "PKCE S256 is required"
        );
        anyhow::ensure!(
            self.resource_url(default_resource) == default_resource,
            "resource does not match this MCP server"
        );
        Url::parse(&self.redirect_uri).context("redirect_uri must be an absolute URL")?;
        Ok(())
    }

    pub(super) fn resource_url(&self, default_resource: &str) -> String {
        self.resource
            .clone()
            .unwrap_or_else(|| default_resource.to_string())
    }

    pub(super) fn login_fields(&self) -> Vec<(String, String)> {
        vec![
            ("response_type".into(), self.response_type.clone()),
            ("client_id".into(), self.client_id.clone()),
            ("redirect_uri".into(), self.redirect_uri.clone()),
            ("state".into(), self.state.clone().unwrap_or_default()),
            ("code_challenge".into(), self.code_challenge.clone()),
            (
                "code_challenge_method".into(),
                self.code_challenge_method.clone(),
            ),
            ("resource".into(), self.resource.clone().unwrap_or_default()),
            ("scope".into(), self.scope.clone().unwrap_or_default()),
        ]
    }
}

impl AuthorizeForm {
    pub(super) fn params(&self) -> AuthorizeParams {
        AuthorizeParams {
            response_type: self.response_type.clone(),
            client_id: self.client_id.clone(),
            redirect_uri: self.redirect_uri.clone(),
            state: self.state.clone(),
            code_challenge: self.code_challenge.clone(),
            code_challenge_method: self.code_challenge_method.clone(),
            resource: self.resource.clone(),
            scope: self.scope.clone(),
        }
    }
}

pub(super) fn validate_token_request(
    form: &TokenForm,
    code: &AuthCode,
    default_resource: &str,
) -> Result<()> {
    anyhow::ensure!(code.expires_at > crate::util::now_secs(), "code expired");
    anyhow::ensure!(form.client_id == code.client_id, "client_id mismatch");
    anyhow::ensure!(
        form.redirect_uri == code.redirect_uri,
        "redirect_uri mismatch"
    );
    anyhow::ensure!(
        form.resource.as_deref().unwrap_or(default_resource) == code.resource,
        "resource mismatch"
    );
    let expected = URL_SAFE_NO_PAD.encode(Sha256::digest(form.code_verifier.as_bytes()));
    anyhow::ensure!(expected == code.code_challenge, "PKCE verifier mismatch");
    Ok(())
}
