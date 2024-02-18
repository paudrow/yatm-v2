use anyhow::{anyhow, Context, Result};
use async_recursion::async_recursion;
use dotenv::dotenv;
use octocrab::models::issues::Issue;
use octocrab::models::IssueState;
use octocrab::params::State;
use octocrab::Octocrab;
use tokio::time::{sleep, Duration};

pub struct Github {
    octocrab: Octocrab,
    owner: String,
    repo: String,
}

impl Github {
    pub fn new(owner: String, repo: String) -> Result<Self> {
        dotenv().ok();

        let token = std::env::var("GITHUB_TOKEN").context("GITHUB_TOKEN not set")?;

        Ok(Self {
            octocrab: Octocrab::builder()
                .personal_token(token)
                .build()
                .context("Failed to create octocrab")?,
            owner,
            repo,
        })
    }

    pub async fn delete_labels(&self) -> Result<()> {
        let labels = self
            .octocrab
            .issues(&self.owner, &self.repo)
            .list_labels_for_repo()
            .send()
            .await
            .context("Failed to list labels")?;

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
                .context("Failed to delete label")?;
        }

        Ok(())
    }

    pub async fn create_labels(&self, labels: Vec<String>) -> Result<()> {
        for label in labels {
            self.octocrab
                .issues(&self.owner, &self.repo)
                .create_label(&label, "000000", "")
                .await
                .context(format!("Failed to create label: {}", &label))?;
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
            .context("Failed to list issues")?;
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
        let issues = self
            .get_issues(Some(State::Open))
            .await
            .context("Failed to get issues")?;

        for issue in issues {
            self.close_issue(issue).await?;
        }
        Ok(())
    }

    pub async fn close_issue(&self, issue: Issue) -> Result<()> {
        self.octocrab
            .issues(&self.owner, &self.repo)
            .update(issue.number)
            .state(IssueState::Closed)
            .send()
            .await
            .context(format!(
                "Failed to delete issue: #{} {}",
                issue.number, issue.title
            ))?;
        Ok(())
    }
}