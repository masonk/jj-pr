use anyhow::{Context, Result};
use octocrab::Octocrab;

#[derive(Debug, Clone)]
pub struct PullRequest {
    pub number: u64,
    pub title: String,
    pub body: String,
    pub base: String,
    pub head: String,
    pub state: String,
}

pub struct GitHub {
    client: Octocrab,
    owner: String,
    repo: String,
}

impl GitHub {
    pub fn new(token: String, owner: String, repo: String) -> Result<Self> {
        let client = Octocrab::builder()
            .personal_token(token)
            .build()
            .context("Failed to create GitHub client")?;

        Ok(Self {
            client,
            owner,
            repo,
        })
    }

    /// Create a new pull request
    pub async fn create_pull_request(
        &self,
        title: String,
        body: String,
        head: String,
        base: String,
    ) -> Result<PullRequest> {
        let pr = self
            .client
            .pulls(&self.owner, &self.repo)
            .create(title.clone(), head.clone(), base.clone())
            .body(body.clone())
            .send()
            .await
            .context("Failed to create pull request")?;

        Ok(PullRequest {
            number: pr.number,
            title,
            body,
            base,
            head,
            state: pr.state.map(|s| format!("{:?}", s)).unwrap_or_default(),
        })
    }

    /// Update an existing pull request
    pub async fn update_pull_request(
        &self,
        number: u64,
        title: Option<String>,
        body: Option<String>,
        base: Option<String>,
    ) -> Result<()> {
        let pulls = self.client.pulls(&self.owner, &self.repo);
        let mut pr_update = pulls.update(number);

        if let Some(ref t) = title {
            pr_update = pr_update.title(t);
        }
        if let Some(ref b) = body {
            pr_update = pr_update.body(b);
        }
        if let Some(ref base_ref) = base {
            pr_update = pr_update.base(base_ref);
        }

        pr_update.send().await.context("Failed to update pull request")?;
        Ok(())
    }

    /// Get a pull request by number
    pub async fn get_pull_request(&self, number: u64) -> Result<PullRequest> {
        let pr = self
            .client
            .pulls(&self.owner, &self.repo)
            .get(number)
            .await
            .context("Failed to get pull request")?;

        Ok(PullRequest {
            number: pr.number,
            title: pr.title.unwrap_or_default(),
            body: pr.body.unwrap_or_default(),
            base: pr.base.ref_field,
            head: pr.head.ref_field,
            state: pr.state.map(|s| format!("{:?}", s)).unwrap_or_default(),
        })
    }

    /// Find pull request by head branch
    pub async fn find_pull_request_by_branch(&self, branch: &str) -> Result<Option<PullRequest>> {
        let head = format!("{}:{}", self.owner, branch);

        let prs = self
            .client
            .pulls(&self.owner, &self.repo)
            .list()
            .state(octocrab::params::State::Open)
            .head(&head)
            .send()
            .await
            .context("Failed to list pull requests")?;

        if let Some(pr) = prs.items.first() {
            Ok(Some(PullRequest {
                number: pr.number,
                title: pr.title.clone().unwrap_or_default(),
                body: pr.body.clone().unwrap_or_default(),
                base: pr.base.ref_field.clone(),
                head: pr.head.ref_field.clone(),
                state: pr.state.as_ref().map(|s| format!("{:?}", s)).unwrap_or_default(),
            }))
        } else {
            Ok(None)
        }
    }
}
