use anyhow::{Context, Result};
use dotenv::dotenv;
use octocrab::Octocrab;

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

    pub async fn get_issues(&self) -> Result<octocrab::Page<octocrab::models::issues::Issue>> {
        let issues = self
            .octocrab
            .issues(&self.owner, &self.repo)
            .list()
            .state(octocrab::params::State::Open)
            .send()
            .await
            .context("Failed to list issues")?;
        Ok(issues)
    }

    pub async fn create_issue(
        &self,
        title: String,
        body: String,
        labels: Vec<String>,
    ) -> Result<()> {
        self.octocrab
            .issues(&self.owner, &self.repo)
            .create(&title)
            .body(&body)
            .labels(labels)
            .send()
            .await
            .context(format!("Failed to create issue: {}", &title))?;
        Ok(())
    }

    pub async fn close_all_issues(&self) -> Result<()> {
        let issues = self
            .octocrab
            .issues(&self.owner, &self.repo)
            .list()
            .state(octocrab::params::State::Open)
            .send()
            .await
            .context("Failed to list issues")?;

        for issue in issues {
            self.octocrab
                .issues(&self.owner, &self.repo)
                .update(issue.number)
                .state(octocrab::models::IssueState::Closed)
                .send()
                .await
                .context(format!("Failed to delete issue: {}", issue.number))?;
        }
        Ok(())
    }
}
