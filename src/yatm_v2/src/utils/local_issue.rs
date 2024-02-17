use anyhow::{Context, Result};
use askama::Template;
use common::types::{Action, Expect, Link, Step, TestCase};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocalIssue {
    pub labels: Vec<String>,
    pub title: String,
    pub text_body: String,
}
