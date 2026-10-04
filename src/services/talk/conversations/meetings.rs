//! Calendar integration: scheduled meetings and events linked to conversations.

use eventful_rs::*;

use crate::{
    dto::talk::{DashboardEvent, ScheduleMeeting},
    services::{
        ncclient::NcError,
        talk::{ApiVersion, room_path, talk_path},
    },
};

service! {
    pub struct MeetingService;
}

#[asynchronize(pub)]
impl MeetingService {
    /// Create a calendar event for the conversation and invite its attendees.
    #[asynced]
    pub async fn schedule(&self, token: String, meeting: ScheduleMeeting) -> Result<(), NcError> {
        self.client
            .post(&room_path(&token, "/meeting"))
            .json(&meeting)
            .send_discarding_data()
            .await?;
        Ok(())
    }

    /// Upcoming events of the current user that have a conversation.
    #[asynced]
    pub async fn upcoming(&self) -> Result<Vec<DashboardEvent>, NcError> {
        Ok(self
            .client
            .get(&talk_path(ApiVersion::V4, "dashboard/events"))
            .send()
            .await?
            .data)
    }

    /// Upcoming events shared with the other participant of a one-to-one conversation.
    #[asynced]
    pub async fn mutual(&self, token: String) -> Result<Vec<DashboardEvent>, NcError> {
        Ok(self
            .client
            .get(&room_path(&token, "/mutual-events"))
            .send()
            .await?
            .data)
    }
}
