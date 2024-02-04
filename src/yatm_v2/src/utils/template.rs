use askama::Template;

#[derive(Template, Clone)]
#[template(path = "content.md")]
struct GithubIssueTemplate {
    name: String,
}

fn main() {
    let template = GithubIssueTemplate {
        name: "world".to_string(),
    };
    println!("{}", template.render().unwrap());
}