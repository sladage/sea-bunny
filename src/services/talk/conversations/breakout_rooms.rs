//! Breakout rooms of a conversation. Configuring, starting and broadcasting
//! require moderator permissions; switching and asking for help are for
//! participants.

use eventful_rs::*;

use crate::{
    dto::talk::{
        AttendeeMap, ConfigureBreakoutRooms, Conversation, Participant,
        breakout::attendee_map_as_json,
    },
    services::{
        ncclient::NcError,
        talk::{ApiVersion, room_path, talk_path},
    },
};

service! {
    pub struct BreakoutRoomService;
}

#[asynchronize(pub)]
impl BreakoutRoomService {
    /// Breakout rooms of a parent conversation.
    #[asynced]
    pub async fn list(&self, token: String) -> Result<Vec<Conversation>, NcError> {
        Ok(self
            .client
            .get(&room_path(&token, "/breakout-rooms"))
            .send()
            .await?
            .data)
    }

    /// Participants of all breakout rooms of a parent conversation.
    #[asynced]
    pub async fn participants(
        &self,
        token: String,
        include_status: bool,
    ) -> Result<Vec<Participant>, NcError> {
        Ok(self
            .client
            .get(&room_path(&token, "/breakout-rooms/participants"))
            .query(&[("includeStatus", include_status)])
            .send()
            .await?
            .data)
    }

    /// Create the breakout rooms. Returns them.
    #[asynced]
    pub async fn configure(
        &self,
        token: String,
        config: ConfigureBreakoutRooms,
    ) -> Result<Vec<Conversation>, NcError> {
        Ok(self
            .client
            .post(&breakout_path(&token, ""))
            .json(&config)
            .send()
            .await?
            .data)
    }

    /// Delete all breakout rooms. Returns the parent conversation.
    #[asynced]
    pub async fn remove(&self, token: String) -> Result<Conversation, NcError> {
        Ok(self
            .client
            .delete(&breakout_path(&token, ""))
            .send()
            .await?
            .data)
    }

    /// Reassign attendees to rooms (manual mode).
    #[asynced]
    pub async fn assign(
        &self,
        token: String,
        attendee_map: AttendeeMap,
    ) -> Result<Vec<Conversation>, NcError> {
        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Body<'a> {
            #[serde(serialize_with = "attendee_map_as_json")]
            attendee_map: &'a AttendeeMap,
        }

        Ok(self
            .client
            .post(&breakout_path(&token, "/attendees"))
            .json(&Body {
                attendee_map: &attendee_map,
            })
            .send()
            .await?
            .data)
    }

    /// Open the breakout rooms (their lobbies are disabled).
    #[asynced]
    pub async fn start(&self, token: String) -> Result<Vec<Conversation>, NcError> {
        Ok(self
            .client
            .post(&breakout_path(&token, "/rooms"))
            .send()
            .await?
            .data)
    }

    /// Close the breakout rooms, sending everyone back to the parent conversation.
    #[asynced]
    pub async fn stop(&self, token: String) -> Result<Vec<Conversation>, NcError> {
        Ok(self
            .client
            .delete(&breakout_path(&token, "/rooms"))
            .send()
            .await?
            .data)
    }

    /// Post a message to every breakout room.
    #[asynced]
    pub async fn broadcast(
        &self,
        token: String,
        message: String,
    ) -> Result<Vec<Conversation>, NcError> {
        Ok(self
            .client
            .post(&breakout_path(&token, "/broadcast"))
            .json(&serde_json::json!({ "message": message }))
            .send()
            .await?
            .data)
    }

    /// Ask the moderators of the parent conversation for help, from a breakout room.
    #[asynced]
    pub async fn request_assistance(&self, token: String) -> Result<Conversation, NcError> {
        Ok(self
            .client
            .post(&breakout_path(&token, "/request-assistance"))
            .send()
            .await?
            .data)
    }

    #[asynced]
    pub async fn cancel_assistance_request(&self, token: String) -> Result<Conversation, NcError> {
        Ok(self
            .client
            .delete(&breakout_path(&token, "/request-assistance"))
            .send()
            .await?
            .data)
    }

    /// Move to another breakout room (free mode). `token` is the parent
    /// conversation; returns the target room.
    #[asynced]
    pub async fn switch(
        &self,
        token: String,
        target_token: String,
    ) -> Result<Conversation, NcError> {
        Ok(self
            .client
            .post(&breakout_path(&token, "/switch"))
            .json(&serde_json::json!({ "target": target_token }))
            .send()
            .await?
            .data)
    }
}

fn breakout_path(token: &str, suffix: &str) -> String {
    talk_path(ApiVersion::V1, format!("breakout-rooms/{token}{suffix}"))
}
