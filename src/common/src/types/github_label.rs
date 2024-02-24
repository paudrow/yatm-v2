use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GithubLabel {
    pub name: String,
    pub color: String,
    pub description: Option<String>,
}
