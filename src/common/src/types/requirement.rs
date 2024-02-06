use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Requirement {
    pub name: String,
    pub description: String,
    pub steps: Vec<Step>,
    pub labels: Option<Vec<String>>,
    pub links: Option<Vec<Link>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Link {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Step {
    pub action: Vec<Action>,
    pub expect: Vec<Expect>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Action {
    StdIn(Terminal),
    Image(String),
    Describe(String),
    Url(Link),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Expect {
    StdOut(Terminal),
    StdErr(Terminal),
    Image(String),
    Describe(String),
    Url(Link),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Terminal {
    pub number: u8,
    pub text: String,
}
