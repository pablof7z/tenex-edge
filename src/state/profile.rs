/// kind:0 metadata for any pubkey.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Profile {
    pub pubkey: String,
    pub name: String,
    pub slug: String,
    pub agent_slug: String,
    pub host: String,
    pub is_backend: bool,
    pub agents: Vec<(String, String)>,
    pub workspaces: Vec<String>,
    pub updated_at: u64,
}
