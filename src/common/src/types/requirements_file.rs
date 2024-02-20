use crate::types::Requirement;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RequirementsFile {
    pub requirements: Vec<Requirement>,
}

impl RequirementsFile {
    pub fn default() -> Self {
        RequirementsFile {
            requirements: vec![Requirement::default()],
        }
    }
}
