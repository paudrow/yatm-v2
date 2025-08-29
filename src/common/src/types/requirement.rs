use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Requirement {
    pub name: String,
    pub shortname: Option<String>,
    pub description: String,
    pub steps: Vec<Step>,
    pub labels: Option<Vec<String>>,
    pub links: Option<Vec<Link>>,
}

impl Requirement {
    pub fn default() -> Self {
        Requirement {
            name: "name".to_string(),
            shortname: Some("shortname".to_string()),
            description: "description".to_string(),
            steps: vec![Step {
                name: Some("step name".to_string()),
                description: Some("step description".to_string()),
                action: vec![
                    Action::Describe("action".to_string()),
                    Action::StdIn(Terminal {
                        number: 1,
                        text: "echo 'hi'".to_string(),
                    }),
                    Action::Url(Link {
                        name: "Google".to_string(),
                        url: "www.google.com".to_string(),
                    }),
                    Action::Image("https://placekitten.com/200/300".to_string()),
                ],
                expect: vec![
                    Expect::Describe("expect".to_string()),
                    Expect::StdOut(Terminal {
                        number: 1,
                        text: "hi".to_string(),
                    }),
                    Expect::StdErr(Terminal {
                        number: 1,
                        text: "error".to_string(),
                    }),
                    Expect::Url(Link {
                        name: "Google".to_string(),
                        url: "www.google.com".to_string(),
                    }),
                    Expect::Image("https://placekitten.com/200/300".to_string()),
                ],
            }],
            labels: Some(vec!["label".to_string()]),
            links: Some(vec![Link {
                name: "Google".to_string(),
                url: "www.google.com".to_string(),
            }]),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Link {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Step {
    pub name: Option<String>,
    pub description: Option<String>,
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
