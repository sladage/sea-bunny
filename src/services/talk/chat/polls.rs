//! Polls posted in a conversation, and poll drafts (moderators only).

use eventful_rs::*;

use crate::{
    dto::talk::{CreatePoll, Poll, PollDraft, PollExportFormat},
    services::{
        ncclient::{Download, NcError},
        talk::{ApiVersion, talk_path},
    },
};

service! {
    pub struct PollService;
}

#[asynchronize(pub)]
impl PollService {
    /// Post a poll. Use [`Self::save_draft`] for drafts.
    #[asynced]
    pub async fn create(&self, token: String, poll: CreatePoll) -> Result<Poll, NcError> {
        let poll = CreatePoll {
            draft: false,
            ..poll
        };
        Ok(self
            .client
            .post(&poll_path(&token, ""))
            .json(&poll)
            .send()
            .await?
            .data)
    }

    #[asynced]
    pub async fn get(&self, token: String, poll_id: i64) -> Result<Poll, NcError> {
        Ok(self
            .client
            .get(&poll_path(&token, &format!("/{poll_id}")))
            .send()
            .await?
            .data)
    }

    /// Vote for the given option indexes, replacing earlier votes. An empty list
    /// removes the vote.
    #[asynced]
    pub async fn vote(
        &self,
        token: String,
        poll_id: i64,
        option_ids: Vec<i64>,
    ) -> Result<Poll, NcError> {
        Ok(self
            .client
            .post(&poll_path(&token, &format!("/{poll_id}")))
            .json(&serde_json::json!({ "optionIds": option_ids }))
            .send()
            .await?
            .data)
    }

    /// Close a poll, or delete a draft. Returns the closed poll (`None` for drafts).
    #[asynced]
    pub async fn close(&self, token: String, poll_id: i64) -> Result<Option<Poll>, NcError> {
        Ok(self
            .client
            .delete(&poll_path(&token, &format!("/{poll_id}")))
            .send()
            .await?
            .data)
    }

    #[asynced]
    pub async fn drafts(&self, token: String) -> Result<Vec<PollDraft>, NcError> {
        Ok(self
            .client
            .get(&poll_path(&token, "/drafts"))
            .send()
            .await?
            .data)
    }

    #[asynced]
    pub async fn save_draft(&self, token: String, poll: CreatePoll) -> Result<PollDraft, NcError> {
        let poll = CreatePoll {
            draft: true,
            ..poll
        };
        Ok(self
            .client
            .post(&poll_path(&token, ""))
            .json(&poll)
            .send()
            .await?
            .data)
    }

    #[asynced]
    pub async fn update_draft(
        &self,
        token: String,
        poll_id: i64,
        poll: CreatePoll,
    ) -> Result<PollDraft, NcError> {
        let poll = CreatePoll {
            draft: false,
            thread_id: None,
            ..poll
        };
        Ok(self
            .client
            .post(&poll_path(&token, &format!("/draft/{poll_id}")))
            .json(&poll)
            .send()
            .await?
            .data)
    }

    /// The results as a spreadsheet.
    #[asynced]
    pub async fn export(
        &self,
        token: String,
        poll_id: i64,
        format: PollExportFormat,
    ) -> Result<Download, NcError> {
        self.client
            .get(&poll_path(
                &token,
                &format!("/{poll_id}/export/{}", format.as_str()),
            ))
            .send_download()
            .await
    }
}

fn poll_path(token: &str, suffix: &str) -> String {
    talk_path(ApiVersion::V1, format!("poll/{token}{suffix}"))
}
