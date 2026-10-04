use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::ConversationType;
use crate::dto::serde_ext::empty_as_default;

/// A calendar event linked to a conversation.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct DashboardEvent {
    pub event_name: String,
    pub event_description: Option<String>,
    pub event_link: String,
    pub start: i64,
    pub end: i64,
    pub calendars: Vec<DashboardEventCalendar>,
    #[serde(deserialize_with = "empty_as_default")]
    pub event_attachments: HashMap<String, DashboardEventAttachment>,
    pub room_token: String,
    pub room_name: String,
    pub room_display_name: String,
    pub room_type: ConversationType,
    pub room_avatar_version: String,
    /// Start of the ongoing call, if any.
    pub room_active_since: Option<i64>,
    pub invited: Option<i64>,
    pub accepted: Option<i64>,
    pub tentative: Option<i64>,
    pub declined: Option<i64>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct DashboardEventCalendar {
    pub principal_uri: String,
    pub calendar_name: String,
    pub calendar_color: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct DashboardEventAttachment {
    pub calendars: Vec<String>,
    pub fmttype: String,
    pub filename: String,
    pub fileid: i64,
    pub preview: bool,
    pub preview_link: Option<String>,
}

/// Body for scheduling a meeting in the user's calendar for a conversation.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleMeeting {
    /// Last part of the calendar URI, e.g. `personal`.
    pub calendar_uri: String,
    pub start: i64,
    /// Defaults to one hour after `start`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<i64>,
    /// `None` invites everyone; an empty list only the current user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attendee_ids: Option<Vec<i64>>,
    /// Defaults to the conversation name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Defaults to the conversation description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}
