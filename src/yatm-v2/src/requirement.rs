use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Requirement {
    pub name: String,
    pub description: String,
    pub labels: Vec<String>,
    pub steps: Vec<Step>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Step {
    pub action: Vec<Action>,
    pub expect: Vec<Expect>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Action {
    StdIn(Terminal),
    Image(String),
    Describe(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Expect {
    StdOut(Terminal),
    StdErr(Terminal),
    Image(String),
    Describe(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Terminal {
    pub number: u8,
    pub text: String,
}
