use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{ActorType, PollResultMode, PollStatus};
use crate::dto::serde_ext::empty_as_default;

/// A poll draft, or the question part of a poll.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct PollDraft {
    pub id: i64,
    pub question: String,
    pub options: Vec<String>,
    pub result_mode: PollResultMode,
    /// 0 for unlimited.
    pub max_votes: i64,
    pub status: PollStatus,
    pub actor_type: ActorType,
    pub actor_id: String,
    pub actor_display_name: String,
}

/// A poll with its results, as far as the current user may see them.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Poll {
    pub id: i64,
    pub question: String,
    pub options: Vec<String>,
    pub result_mode: PollResultMode,
    pub max_votes: i64,
    pub status: PollStatus,
    pub actor_type: ActorType,
    pub actor_id: String,
    pub actor_display_name: String,
    /// Vote count per option, keyed `option-<index>`. Hidden polls only expose
    /// it once closed.
    #[serde(deserialize_with = "empty_as_default")]
    pub votes: HashMap<String, i64>,
    pub num_voters: i64,
    /// Option indexes the current user voted for.
    pub voted_self: Vec<i64>,
    /// Individual votes (public polls only).
    pub details: Vec<PollVote>,
}

impl Poll {
    /// Votes for the option at `index`.
    pub fn votes_for(&self, index: usize) -> i64 {
        self.votes
            .get(&format!("option-{index}"))
            .copied()
            .unwrap_or(0)
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct PollVote {
    pub actor_type: ActorType,
    pub actor_id: String,
    pub actor_display_name: String,
    pub option_id: i64,
}

/// Body for creating a poll or updating a draft.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePoll {
    pub question: String,
    pub options: Vec<String>,
    pub result_mode: PollResultMode,
    /// 0 for unlimited.
    pub max_votes: i64,
    /// Save as a draft instead of posting it.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub draft: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PollExportFormat {
    Csv,
    Ods,
}

impl PollExportFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::Ods => "ods",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_poll_with_votes() {
        let json = r#"{
            "id": 3, "question": "Lunch?", "options": ["Pizza", "Sushi"], "resultMode": 0,
            "maxVotes": 1, "status": 0, "actorType": "users", "actorId": "alice",
            "actorDisplayName": "Alice", "votes": {"option-1": 2}, "numVoters": 2,
            "votedSelf": [1], "details": [{"actorType": "users", "actorId": "bob",
            "actorDisplayName": "Bob", "optionId": 1}]
        }"#;
        let poll: Poll = serde_json::from_str(json).unwrap();
        assert_eq!(poll.votes_for(1), 2);
        assert_eq!(poll.votes_for(0), 0);
        assert_eq!(poll.details[0].option_id, 1);
    }

    #[test]
    fn parses_poll_with_empty_votes() {
        let poll: Poll = serde_json::from_str(r#"{"id": 1, "votes": [], "status": 1}"#).unwrap();
        assert!(poll.votes.is_empty());
        assert_eq!(poll.status, PollStatus::Closed);
    }
}
