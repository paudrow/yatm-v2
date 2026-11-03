use crate::types::{GithubCollaborator, GithubLabel, GithubRepoSettings};
use anyhow::Ok;
use anyhow::{anyhow, Context, Result};
use async_recursion::async_recursion;
use dotenv::dotenv;
use octocrab::models::issues::Issue;
use octocrab::models::IssueState;
use octocrab::models::Label;
use octocrab::params::State;
use octocrab::Octocrab;
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};

const USERNAME_ENCODE_SET: &percent_encoding::AsciiSet =
    &percent_encoding::NON_ALPHANUMERIC.remove(b'-').remove(b'_');

#[derive(Serialize)]
struct CreateRepoRequest<'a> {
    name: &'a str,
    private: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'a str>,
}

#[derive(Serialize)]
struct UpdateRepoRequest<'a> {
    private: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'a str>,
}

#[derive(Serialize)]
struct AddCollaboratorRequest<'a> {
    permission: &'a str,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct RemoteRepoState {
    #[serde(default)]
    pub private: bool,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DirectCollaborator {
    pub login: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RepoInvitationInvitee {
    pub login: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RepoInvitation {
    pub id: u64,
    pub invitee: Option<RepoInvitationInvitee>,
}

pub struct Github {
    octocrab: Option<Octocrab>,
    owner: String,
    repo: String,
}

impl Github {
    pub fn new(owner: &String, repo: &String) -> Result<Self> {
        dotenv().ok();

        let token = std::env::var("GITHUB_TOKEN")
            .ok()
            .filter(|token| !token.trim().is_empty());
        let api_url = std::env::var("GITHUB_API_URL")
            .ok()
            .filter(|url| !url.trim().is_empty());

        Self::new_with_options(owner, repo, token, api_url)
    }

    pub fn new_with_options(
        owner: &str,
        repo: &str,
        token: Option<String>,
        base_uri: Option<String>,
    ) -> Result<Self> {
        let octocrab = match token {
            Some(token) => {
                let mut builder = Octocrab::builder().personal_token(token);
                if let Some(uri) = base_uri {
                    builder = builder
                        .base_uri(uri)
                        .context("Failed to configure GITHUB_API_URL on octocrab")?;
                }
                Some(builder.build().context("Failed to create octocrab")?)
            }
            None => None,
        };

        Ok(Self {
            octocrab,
            owner: owner.to_string(),
            repo: repo.to_string(),
        })
    }

    fn get_octocrab(&self) -> Result<&Octocrab> {
        self.octocrab.as_ref().context("GITHUB_TOKEN not set")
    }

    fn is_not_found(err: &octocrab::Error) -> bool {
        matches!(
            err,
            octocrab::Error::GitHub { source, .. } if source.status_code.as_u16() == 404
        )
    }

    fn is_already_exists_error(err: &octocrab::Error) -> bool {
        if let octocrab::Error::GitHub { source, .. } = err {
            if source.status_code.as_u16() == 422 {
                if let Some(errors) = &source.errors {
                    return errors.iter().any(|e| {
                        e.get("message")
                            .and_then(|m| m.as_str())
                            .map_or(false, |m| m.contains("name already exists on this account"))
                    });
                }
            }
        }
        false
    }

    pub async fn get_repository_state(&self) -> Result<Option<RemoteRepoState>> {
        let route = format!("/repos/{}/{}", self.owner, self.repo);
        match self
            .get_octocrab()?
            .get::<RemoteRepoState, _, _>(&route, None::<&()>)
            .await
        {
            std::result::Result::Ok(state) => Ok(Some(state)),
            Err(err) if Self::is_not_found(&err) => Ok(None),
            Err(err) => Err(anyhow!(err)).context(format!(
                "Failed to fetch repository state for {}/{}",
                self.owner, self.repo
            )),
        }
    }

    pub async fn repo_exists(&self) -> Result<bool> {
        Ok(self.get_repository_state().await?.is_some())
    }

    pub fn settings_match(remote: &RemoteRepoState, desired: &GithubRepoSettings) -> bool {
        if remote.private != desired.private {
            return false;
        }
        if let Some(desired_desc) = &desired.description {
            let remote_desc = remote.description.as_deref().unwrap_or("");
            if remote_desc != desired_desc.as_str() {
                return false;
            }
        }
        true
    }

    pub async fn update_repository_settings(&self, settings: &GithubRepoSettings) -> Result<()> {
        let route = format!("/repos/{}/{}", self.owner, self.repo);
        let body = UpdateRepoRequest {
            private: settings.private,
            description: settings.description.as_deref(),
        };
        let response = self.get_octocrab()?._patch(route, Some(&body)).await?;
        octocrab::map_github_error(response).await.context(format!(
            "Failed to update repository settings for {}/{}",
            self.owner, self.repo
        ))?;
        println!(
            "Updated repository settings for {}/{}",
            self.owner, self.repo
        );
        Ok(())
    }

    async fn wait_until_repo_ready(&self) -> Result<()> {
        for attempt in 0..5 {
            if self.repo_exists().await? {
                return Ok(());
            }
            if attempt < 4 {
                sleep(Duration::from_millis(500)).await;
            }
        }
        anyhow::bail!(
            "Repository {}/{} was created, but could not be fetched yet",
            self.owner,
            self.repo
        )
    }

    pub async fn create_repository(&self, settings: &GithubRepoSettings) -> Result<()> {
        let crab = self.get_octocrab()?;
        let body = CreateRepoRequest {
            name: &self.repo,
            private: settings.private,
            description: settings.description.as_deref(),
        };
        let visibility = if settings.private {
            "private"
        } else {
            "public"
        };

        let org_route = format!("/orgs/{}/repos", self.owner);
        let org_response = crab._post(org_route, Some(&body)).await?;
        match octocrab::map_github_error(org_response).await {
            std::result::Result::Ok(_) => {
                println!(
                    "Created {} repository {}/{}: https://github.com/{}/{}",
                    visibility, self.owner, self.repo, self.owner, self.repo
                );
                self.wait_until_repo_ready().await?;
                return Ok(());
            }
            Err(err) if Self::is_already_exists_error(&err) => {
                if self.repo_exists().await? {
                    println!(
                        "Repository {}/{} already exists (created concurrently)",
                        self.owner, self.repo
                    );
                    return Ok(());
                }
                return Err(anyhow!(err)).context(format!(
                    "Repository {}/{} reported already exists, but could not be fetched",
                    self.owner, self.repo
                ));
            }
            Err(org_err) if Self::is_not_found(&org_err) => {
                // Fall back to /user/repos if owner matches the authenticated user account.
                if let std::result::Result::Ok(user) = crab.current().user().await {
                    if user.login.eq_ignore_ascii_case(&self.owner) {
                        let user_response = crab._post("/user/repos", Some(&body)).await?;
                        match octocrab::map_github_error(user_response).await {
                            std::result::Result::Ok(_) => {
                                println!(
                                    "Created {} repository {}/{}: https://github.com/{}/{}",
                                    visibility, self.owner, self.repo, self.owner, self.repo
                                );
                                self.wait_until_repo_ready().await?;
                                return Ok(());
                            }
                            Err(err) if Self::is_already_exists_error(&err) => {
                                if self.repo_exists().await? {
                                    println!(
                                        "Repository {}/{} already exists (created concurrently)",
                                        self.owner, self.repo
                                    );
                                    return Ok(());
                                }
                            }
                            Err(user_err) => {
                                return Err(anyhow!(user_err)).context(format!(
                                    "Failed to create user repository {}/{}",
                                    self.owner, self.repo
                                ));
                            }
                        }
                    }
                }
                return Err(anyhow!(org_err)).context(format!(
                    "Failed to create repository {}/{} (verify organization '{}' exists and GITHUB_TOKEN has Administration: Read and write permission)",
                    self.owner, self.repo, self.owner
                ));
            }
            Err(err) => {
                return Err(anyhow!(err)).context(format!(
                    "Failed to create repository {}/{}",
                    self.owner, self.repo
                ));
            }
        }
    }

    pub async fn add_collaborator(&self, collaborator: &GithubCollaborator) -> Result<()> {
        let encoded_username =
            percent_encoding::utf8_percent_encode(&collaborator.username, USERNAME_ENCODE_SET);
        let route = format!(
            "/repos/{}/{}/collaborators/{}",
            self.owner, self.repo, encoded_username
        );
        let body = AddCollaboratorRequest {
            permission: collaborator.permission.as_api_str(),
        };
        let response = self.get_octocrab()?._put(route, Some(&body)).await?;
        let response = octocrab::map_github_error(response).await.context(format!(
            "Failed to grant '{}' permission to '{}' on {}/{}",
            collaborator.permission, collaborator.username, self.owner, self.repo
        ))?;

        if response.status().as_u16() == 201 {
            println!(
                "Invited '{}' with '{}' permission to {}/{}",
                collaborator.username, collaborator.permission, self.owner, self.repo
            );
        } else {
            println!(
                "Ensured '{}' has '{}' permission on {}/{}",
                collaborator.username, collaborator.permission, self.owner, self.repo
            );
        }
        Ok(())
    }

    pub async fn list_direct_collaborators(&self) -> Result<Vec<DirectCollaborator>> {
        let mut page: u32 = 1;
        let mut collaborators = Vec::new();
        loop {
            let route = format!(
                "/repos/{}/{}/collaborators?affiliation=direct&per_page=100&page={}",
                self.owner, self.repo, page
            );
            let page_items: Vec<DirectCollaborator> = self
                .get_octocrab()?
                .get(&route, None::<&()>)
                .await
                .context(format!(
                    "Failed to list direct collaborators for {}/{}",
                    self.owner, self.repo
                ))?;
            if page_items.is_empty() {
                break;
            }
            let count = page_items.len();
            collaborators.extend(page_items);
            if count < 100 {
                break;
            }
            page += 1;
        }
        Ok(collaborators)
    }

    pub async fn remove_collaborator(&self, username: &str) -> Result<()> {
        let encoded_username = percent_encoding::utf8_percent_encode(username, USERNAME_ENCODE_SET);
        let route = format!(
            "/repos/{}/{}/collaborators/{}",
            self.owner, self.repo, encoded_username
        );
        let response = self.get_octocrab()?._delete(route, None::<&()>).await?;
        octocrab::map_github_error(response).await.context(format!(
            "Failed to remove collaborator '{}' from {}/{}",
            username, self.owner, self.repo
        ))?;
        println!(
            "Removed unlisted collaborator '{}' from {}/{}",
            username, self.owner, self.repo
        );
        Ok(())
    }

    pub async fn list_pending_invitations(&self) -> Result<Vec<RepoInvitation>> {
        let mut page: u32 = 1;
        let mut invitations = Vec::new();
        loop {
            let route = format!(
                "/repos/{}/{}/invitations?per_page=100&page={}",
                self.owner, self.repo, page
            );
            let page_items: Vec<RepoInvitation> = self
                .get_octocrab()?
                .get(&route, None::<&()>)
                .await
                .context(format!(
                    "Failed to list pending repository invitations for {}/{}",
                    self.owner, self.repo
                ))?;
            if page_items.is_empty() {
                break;
            }
            let count = page_items.len();
            invitations.extend(page_items);
            if count < 100 {
                break;
            }
            page += 1;
        }
        Ok(invitations)
    }

    pub async fn delete_invitation(&self, invitation_id: u64, username: &str) -> Result<()> {
        let route = format!(
            "/repos/{}/{}/invitations/{}",
            self.owner, self.repo, invitation_id
        );
        let response = self.get_octocrab()?._delete(route, None::<&()>).await?;
        octocrab::map_github_error(response).await.context(format!(
            "Failed to revoke pending invitation {} for '{}' on {}/{}",
            invitation_id, username, self.owner, self.repo
        ))?;
        println!(
            "Revoked pending invitation for '{}' on {}/{}",
            username, self.owner, self.repo
        );
        Ok(())
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
            .get_octocrab()?
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
            self.get_octocrab()?
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
            self.get_octocrab()?
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
            .get_octocrab()?
            .issues(&self.owner, &self.repo)
            .list()
            .per_page(100)
            .state(state.unwrap_or(State::All))
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
            .get_octocrab()?
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

    #[async_recursion]
    pub async fn update_issue(
        &self,
        issue_id: u64,
        title: String,
        body: String,
        // labels: Vec<String>,  //TODO(tfoote) Restore label setting
    ) -> Result<()> {
        let result = self
            .get_octocrab()?
            .issues(&self.owner, &self.repo)
            .update(issue_id)
            .title(&title)
            .body(&body)
            // .labels(labels.clone())
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
                return self.update_issue(issue_id, title, body).await; //, labels
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
        self.get_octocrab()?
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
        self.prepend_github_info_with_settings(content, None)
    }

    pub fn prepend_github_info_with_settings(
        &self,
        content: &String,
        repo_settings: Option<&GithubRepoSettings>,
    ) -> String {
        let mut header = format!(
            "# Github Target\n\nrepository: [{}/{}](https://github.com/{}/{})",
            self.owner, self.repo, self.owner, self.repo
        );
        if let Some(settings) = repo_settings {
            let visibility = if settings.private {
                "private"
            } else {
                "public"
            };
            header.push_str(&format!(
                "\n\nrepo_settings: create_if_missing={}, reapply_settings_after_creation={}, visibility={}",
                settings.create_if_missing, settings.reapply_settings_after_creation, visibility
            ));
            if let Some(description) = &settings.description {
                header.push_str(&format!("\n\ndescription: {}", description));
            }
            if settings.strict_collaborators {
                header.push_str("\n\nstrict_collaborators: true");
            }
            if !settings.collaborators.is_empty() {
                header.push_str("\n\ncollaborators:");
                for collaborator in &settings.collaborators {
                    header.push_str(&format!(
                        "\n- `{}` (`{}`)",
                        collaborator.username, collaborator.permission
                    ));
                }
            }
        }
        if header.is_empty() {
            return content.clone();
        }
        format!("{}\n\n{}", header, content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::CollaboratorPermission;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    async fn read_http_request(stream: &mut tokio::net::TcpStream) -> String {
        let mut buf = Vec::new();
        let mut chunk = [0u8; 4096];
        loop {
            let n = stream.read(&mut chunk).await.unwrap();
            if n == 0 {
                break;
            }
            buf.extend_from_slice(&chunk[..n]);
            if let Some(header_end) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                let headers = String::from_utf8_lossy(&buf[..header_end]);
                let content_len = headers
                    .lines()
                    .find_map(|line| {
                        let lower = line.to_ascii_lowercase();
                        lower
                            .strip_prefix("content-length:")
                            .and_then(|v| v.trim().parse::<usize>().ok())
                    })
                    .unwrap_or(0);
                if buf.len() >= header_end + 4 + content_len {
                    break;
                }
            }
        }
        String::from_utf8_lossy(&buf).to_string()
    }

    async fn send_http_response(stream: &mut tokio::net::TcpStream, status: &str, body: &str) {
        let resp = format!(
            "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            status,
            body.len(),
            body
        );
        stream.write_all(resp.as_bytes()).await.unwrap();
    }

    #[test]
    fn test_prepend_github_info_default_and_with_settings() {
        let gh = Github::new_with_options("intrinsic-checklists", "jdoe-onboarding", None, None)
            .unwrap();
        let body = "Body".to_string();
        assert_eq!(
            gh.prepend_github_info(&body),
            "# Github Target\n\nrepository: [intrinsic-checklists/jdoe-onboarding](https://github.com/intrinsic-checklists/jdoe-onboarding)\n\nBody"
        );

        let settings = GithubRepoSettings {
            create_if_missing: true,
            reapply_settings_after_creation: true,
            private: true,
            description: Some("Checklist repo".to_string()),
            collaborators: vec![GithubCollaborator {
                username: "mgr-handle".to_string(),
                permission: CollaboratorPermission::Admin,
            }],
            strict_collaborators: true,
        };
        let rendered = gh.prepend_github_info_with_settings(&body, Some(&settings));
        assert!(rendered.contains(
            "repo_settings: create_if_missing=true, reapply_settings_after_creation=true, visibility=private"
        ));
        assert!(rendered.contains("description: Checklist repo"));
        assert!(rendered.contains("strict_collaborators: true"));
        assert!(rendered.contains("- `mgr-handle` (`admin`)"));
    }

    #[tokio::test]
    async fn test_repo_exists_404_and_200() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            // 1st request: 404 Not Found
            let (mut stream, _) = listener.accept().await.unwrap();
            let req1 = read_http_request(&mut stream).await;
            assert!(req1.starts_with("GET /repos/intrinsic-checklists/missing-repo "));
            send_http_response(&mut stream, "404 Not Found", r#"{"message":"Not Found"}"#).await;

            // 2nd request: 200 OK
            let (mut stream, _) = listener.accept().await.unwrap();
            let req2 = read_http_request(&mut stream).await;
            assert!(req2.starts_with("GET /repos/intrinsic-checklists/missing-repo "));
            send_http_response(&mut stream, "200 OK", r#"{"id":1}"#).await;
        });

        let gh = Github::new_with_options(
            "intrinsic-checklists",
            "missing-repo",
            Some("test-token".to_string()),
            Some(format!("http://{}", addr)),
        )
        .unwrap();

        assert!(!gh.repo_exists().await.unwrap());
        assert!(gh.repo_exists().await.unwrap());
        server.await.unwrap();
    }

    #[tokio::test]
    async fn test_create_repository_and_add_collaborator() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            // 1st request: POST /orgs/intrinsic-checklists/repos
            let (mut stream, _) = listener.accept().await.unwrap();
            let req1 = read_http_request(&mut stream).await;
            assert!(req1.starts_with("POST /orgs/intrinsic-checklists/repos "));
            assert!(req1.contains(r#""name":"jdoe-onboarding""#));
            assert!(req1.contains(r#""private":true"#));
            send_http_response(&mut stream, "201 Created", r#"{"id":1}"#).await;

            // 2nd request: GET /repos/intrinsic-checklists/jdoe-onboarding (wait_until_repo_ready)
            let (mut stream, _) = listener.accept().await.unwrap();
            let req2 = read_http_request(&mut stream).await;
            assert!(req2.starts_with("GET /repos/intrinsic-checklists/jdoe-onboarding "));
            send_http_response(&mut stream, "200 OK", r#"{"id":1}"#).await;

            // 3rd request: PUT collaborator returning 201 Created (invitation)
            let (mut stream, _) = listener.accept().await.unwrap();
            let req3 = read_http_request(&mut stream).await;
            assert!(req3.starts_with(
                "PUT /repos/intrinsic-checklists/jdoe-onboarding/collaborators/mgr-user "
            ));
            assert!(req3.contains(r#""permission":"admin""#));
            send_http_response(&mut stream, "201 Created", r#"{"id":99}"#).await;

            // 4th request: PUT collaborator returning 204 No Content (existing member)
            let (mut stream, _) = listener.accept().await.unwrap();
            let req4 = read_http_request(&mut stream).await;
            assert!(req4.starts_with(
                "PUT /repos/intrinsic-checklists/jdoe-onboarding/collaborators/mgr-user "
            ));
            send_http_response(&mut stream, "204 No Content", "").await;
        });

        let gh = Github::new_with_options(
            "intrinsic-checklists",
            "jdoe-onboarding",
            Some("test-token".to_string()),
            Some(format!("http://{}", addr)),
        )
        .unwrap();

        gh.create_repository(&GithubRepoSettings {
            create_if_missing: true,
            private: true,
            description: Some("Onboarding checklist".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();

        let collab = GithubCollaborator {
            username: "mgr-user".to_string(),
            permission: CollaboratorPermission::Admin,
        };
        gh.add_collaborator(&collab).await.unwrap();
        gh.add_collaborator(&collab).await.unwrap();

        server.await.unwrap();
    }

    #[tokio::test]
    async fn test_settings_match_and_update_repository_settings() {
        let remote = RemoteRepoState {
            private: true,
            description: None,
        };
        assert!(Github::settings_match(
            &remote,
            &GithubRepoSettings {
                create_if_missing: true,
                private: true,
                description: Some("".to_string()),
                ..Default::default()
            }
        ));
        assert!(Github::settings_match(
            &RemoteRepoState {
                private: true,
                description: Some("Existing desc".to_string()),
            },
            &GithubRepoSettings {
                create_if_missing: false,
                private: true,
                description: None,
                ..Default::default()
            }
        ));
        assert!(!Github::settings_match(
            &remote,
            &GithubRepoSettings {
                create_if_missing: true,
                private: false,
                description: None,
                ..Default::default()
            }
        ));
        assert!(!Github::settings_match(
            &remote,
            &GithubRepoSettings {
                create_if_missing: true,
                private: true,
                description: Some("New desc".to_string()),
                ..Default::default()
            }
        ));

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let req = read_http_request(&mut stream).await;
            assert!(req.starts_with("PATCH /repos/intrinsic-checklists/jdoe-onboarding "));
            assert!(req.contains(r#""private":true"#));
            assert!(req.contains(r#""description":"Updated desc""#));
            send_http_response(&mut stream, "200 OK", r#"{"id":1}"#).await;
        });

        let gh = Github::new_with_options(
            "intrinsic-checklists",
            "jdoe-onboarding",
            Some("test-token".to_string()),
            Some(format!("http://{}", addr)),
        )
        .unwrap();

        gh.update_repository_settings(&GithubRepoSettings {
            create_if_missing: true,
            private: true,
            description: Some("Updated desc".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();

        server.await.unwrap();
    }

    #[tokio::test]
    async fn test_strict_collaborators_api_helpers() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let (mut s1, _) = listener.accept().await.unwrap();
            let req1 = read_http_request(&mut s1).await;
            assert!(req1.starts_with(
                "GET /repos/intrinsic-checklists/jdoe-onboarding/collaborators?affiliation=direct&per_page=100&page=1 "
            ));
            send_http_response(
                &mut s1,
                "200 OK",
                r#"[{"login":"mgr-user"},{"login":"extra-user"}]"#,
            )
            .await;

            let (mut s2, _) = listener.accept().await.unwrap();
            let req2 = read_http_request(&mut s2).await;
            assert!(req2.starts_with(
                "DELETE /repos/intrinsic-checklists/jdoe-onboarding/collaborators/extra-user "
            ));
            send_http_response(&mut s2, "204 No Content", "").await;

            let (mut s3, _) = listener.accept().await.unwrap();
            let req3 = read_http_request(&mut s3).await;
            assert!(req3.starts_with(
                "GET /repos/intrinsic-checklists/jdoe-onboarding/invitations?per_page=100&page=1 "
            ));
            send_http_response(
                &mut s3,
                "200 OK",
                r#"[{"id":77,"invitee":{"login":"stale-invitee"}}]"#,
            )
            .await;

            let (mut s4, _) = listener.accept().await.unwrap();
            let req4 = read_http_request(&mut s4).await;
            assert!(req4
                .starts_with("DELETE /repos/intrinsic-checklists/jdoe-onboarding/invitations/77 "));
            send_http_response(&mut s4, "204 No Content", "").await;
        });

        let gh = Github::new_with_options(
            "intrinsic-checklists",
            "jdoe-onboarding",
            Some("test-token".to_string()),
            Some(format!("http://{}", addr)),
        )
        .unwrap();

        let direct = gh.list_direct_collaborators().await.unwrap();
        assert_eq!(direct.len(), 2);
        assert_eq!(direct[1].login, "extra-user");
        gh.remove_collaborator("extra-user").await.unwrap();

        let invites = gh.list_pending_invitations().await.unwrap();
        assert_eq!(invites.len(), 1);
        assert_eq!(invites[0].id, 77);
        gh.delete_invitation(77, "stale-invitee").await.unwrap();

        server.await.unwrap();
    }
}
