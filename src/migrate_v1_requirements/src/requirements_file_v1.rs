use common::types::Link;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RequirementsFileV1 {
    pub requirements: Vec<RequirementV1>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequirementV1 {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<Link>>,
    pub checks: Vec<CheckV1>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckV1 {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#try: Option<Vec<TryV1>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expect: Option<Vec<ExpectV1>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TryV1 {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminal: Option<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExpectV1 {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminal: Option<u8>,
}
