use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CollaboratorPermission {
    #[serde(alias = "read")]
    Pull,
    Triage,
    #[serde(alias = "write")]
    Push,
    Maintain,
    Admin,
}

impl CollaboratorPermission {
    pub fn as_api_str(&self) -> &'static str {
        match self {
            CollaboratorPermission::Pull => "pull",
            CollaboratorPermission::Triage => "triage",
            CollaboratorPermission::Push => "push",
            CollaboratorPermission::Maintain => "maintain",
            CollaboratorPermission::Admin => "admin",
        }
    }
}

impl fmt::Display for CollaboratorPermission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_api_str())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct GithubCollaborator {
    pub username: String,
    pub permission: CollaboratorPermission,
}

fn default_true() -> bool {
    true
}

fn is_false(v: &bool) -> bool {
    !*v
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct GithubRepoSettings {
    #[serde(default)]
    pub create_if_missing: bool,
    /// Reapply repository settings and collaborators on existing repositories on each run.
    /// Defaults to `true` whenever `repo_settings` is specified.
    #[serde(default = "default_true")]
    pub reapply_settings_after_creation: bool,
    #[serde(default = "default_true")]
    pub private: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Collaborator permissions to configure on the target repository.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub collaborators: Vec<GithubCollaborator>,
    /// Whether to remove direct collaborators and pending invitations not listed in `collaborators`.
    #[serde(default, skip_serializing_if = "is_false")]
    pub strict_collaborators: bool,
}

impl Default for GithubRepoSettings {
    fn default() -> Self {
        Self {
            create_if_missing: false,
            reapply_settings_after_creation: true,
            private: true,
            description: None,
            collaborators: Vec::new(),
            strict_collaborators: false,
        }
    }
}
