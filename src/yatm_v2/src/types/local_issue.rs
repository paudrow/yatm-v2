use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocalIssue {
    pub labels: Vec<String>,
    pub title: String,
    pub text_body: String,
}
