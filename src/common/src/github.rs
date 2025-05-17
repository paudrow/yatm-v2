use crate::types::GithubLabel;
use anyhow::Ok;
use anyhow::{anyhow, Context, Result};
use async_recursion::async_recursion;
use dotenv::dotenv;
use octocrab::models::issues::Issue;
use octocrab::models::IssueState;
use octocrab::models::Label;
use octocrab::params::State;
use octocrab::Octocrab;
use tokio::time::{sleep, Duration};

pub struct Github {
    octocrab: Octocrab,
    owner: String,
    repo: String,
}

impl Github {
    pub fn new(owner: &String, repo: &String) -> Result<Self> {
        dotenv().ok();

        let token = std::env::var("GITHUB_TOKEN").context("GITHUB_TOKEN not set")?;

        Ok(Self {
            octocrab: Octocrab::builder()
                .personal_token(token)
                .build()
                .context("Failed to create octocrab")?,
            owner: owner.clone(),
            repo: repo.clone(),
        })
    }

    pub async fn get_labels(&self) -> Result<Vec<Label>> {
        let mut page: u32 = 1;
        let mut labels: Vec<Label> = Vec::new();
        loop {
            let page_labels = self.get_labels_helper(page).await?;
            if page_labels.is_empty() {
                break;
            }
            labels.extend(page_labels);
            page += 1;
        }
        Ok(labels)
    }
    async fn get_labels_helper(&self, page: u32) -> Result<Vec<Label>> {
        let page = self
            .octocrab
            .issues(&self.owner, &self.repo)
            .list_labels_for_repo()
            .per_page(100)
            .page(page)
            .send()
            .await
            .context(format!(
                "Failed to list labels for {}/{}",
                self.owner, self.repo
            ))?;
        Ok(page.items)
    }

    pub async fn delete_labels(&self) -> Result<()> {
        let labels = self.get_labels().await?;

        for label in labels {
            let label_name = percent_encoding::utf8_percent_encode(
                &label.name,
                percent_encoding::NON_ALPHANUMERIC,
            )
            .to_string();
            self.octocrab
                .issues(&self.owner, &self.repo)
                .delete_label(&label_name)
                .await
                .context(format!(
                    "Failed to delete label '{}' in {}/{}",
                    &label_name, self.owner, self.repo
                ))?;
        }

        Ok(())
    }

    pub async fn create_labels(&self, labels: Vec<GithubLabel>) -> Result<()> {
        let existing_labels = self.get_labels().await?;

        for label in &labels {
            if existing_labels.iter().any(|l| l.name == label.name) {
                println!("Skipping '{}' because it already exists", label.name);
                continue;
            }
            self.octocrab
                .issues(&self.owner, &self.repo)
                .create_label(
                    &label.name,
                    &label.color,
                    label.description.clone().unwrap_or_default(),
                )
                .await
                .context(format!(
                    "Failed to create label '{}' in {}/{}",
                    &label.name, self.owner, self.repo
                ))?;
            println!("Created label '{}'", label.name);
        }
        Ok(())
    }

    pub async fn get_issues(&self, state: Option<State>) -> Result<Vec<Issue>> {
        let mut page: u32 = 1;
        let mut issues: Vec<Issue> = Vec::new();
        loop {
            let page_issues = self.get_issues_helper(page, state).await?;
            if page_issues.is_empty() {
                break;
            }
            issues.extend(page_issues);
            page += 1;
        }
        Ok(issues)
    }

    async fn get_issues_helper(&self, page: u32, state: Option<State>) -> Result<Vec<Issue>> {
        let page = self
            .octocrab
            .issues(&self.owner, &self.repo)
            .list()
            .per_page(100)
            .state(state.unwrap_or(State::Open))
            .page(page)
            .send()
            .await
            .context(format!(
                "Failed to list issues in {}/{}",
                self.owner, self.repo
            ))?;
        Ok(page.items)
    }

    #[async_recursion]
    pub async fn create_issue(
        &self,
        title: String,
        body: String,
        labels: Vec<String>,
    ) -> Result<()> {
        let result = self
            .octocrab
            .issues(&self.owner, &self.repo)
            .create(&title)
            .body(&body)
            .labels(labels.clone())
            .send()
            .await;

        if result.is_ok() {
            return Ok(());
        }

        let error = result.unwrap_err();
        if let octocrab::Error::GitHub { source, .. } = &error {
            if source
                .message
                .contains("You have exceeded a secondary rate limit")
            {
                println!("Secondary rate limit exceeded, waiting 60 seconds and retrying");
                sleep(Duration::from_secs(60)).await;
                // Retry the request by calling the function recursively.
                return self.create_issue(title, body, labels).await;
            }
        }
        Err(anyhow!(error))
    }

    pub async fn close_all_issues(&self) -> Result<()> {
        let issues = self.get_issues(Some(State::Open)).await.context(format!(
            "Failed to get issues in {}/{}",
            self.owner, self.repo
        ))?;

        for issue in issues {
            self.close_issue(&issue).await?;
        }
        Ok(())
    }

    pub async fn close_issue(&self, issue: &Issue) -> Result<()> {
        self.octocrab
            .issues(&self.owner, &self.repo)
            .update(issue.number)
            .state(IssueState::Closed)
            .send()
            .await
            .context(format!(
                "Failed to delete issue #{} {} in {}/{}",
                issue.number, issue.title, self.owner, self.repo
            ))?;
        Ok(())
    }

    pub fn prepend_github_info(&self, content: &String) -> String {
        let header = format!(
            "# Github Target\n\nrepository: [{}/{}](https://github.com/{}/{})",
            self.owner, self.repo, self.owner, self.repo
        );
        if header.is_empty() {
            return content.clone();
        }
        format!("{}\n\n{}", header, content)
    }
}

