//! Nextcloud Notifications app (`apps/notifications/api/v2`).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::dto::{rich_object::RichObjectParameter, serde_ext::empty_as_default};

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Notification {
    pub notification_id: i64,
    /// App that created it, e.g. `spreed` for Talk.
    pub app: String,
    pub user: String,
    /// ISO 8601 timestamp.
    pub datetime: String,
    /// For Talk: `chat`, `call`, `room`, `reminder`, ...
    pub object_type: String,
    /// For Talk: the conversation token, or `token/messageId` for chat notifications.
    pub object_id: String,
    pub subject: String,
    pub message: String,
    #[serde(rename = "subjectRich")]
    pub subject_rich: String,
    #[serde(
        rename = "subjectRichParameters",
        deserialize_with = "empty_as_default"
    )]
    pub subject_rich_parameters: HashMap<String, RichObjectParameter>,
    #[serde(rename = "messageRich")]
    pub message_rich: String,
    #[serde(
        rename = "messageRichParameters",
        deserialize_with = "empty_as_default"
    )]
    pub message_rich_parameters: HashMap<String, RichObjectParameter>,
    pub link: String,
    pub icon: String,
    pub actions: Vec<NotificationAction>,
    /// Whether a desktop notification should be shown (absent on older servers).
    #[serde(rename = "shouldNotify")]
    pub should_notify: Option<bool>,
}

impl Notification {
    /// The conversation token of a Talk notification.
    pub fn talk_token(&self) -> Option<&str> {
        (self.app == "spreed").then(|| self.object_id.split('/').next().unwrap_or(""))
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct NotificationAction {
    pub label: String,
    pub link: String,
    /// HTTP method to call `link` with (`GET`, `POST`, `DELETE`, `WEB`).
    #[serde(rename = "type")]
    pub method: String,
    pub primary: bool,
}

/// Result of listing notifications.
#[derive(Debug, Clone, Default)]
pub struct NotificationList {
    pub notifications: Vec<Notification>,
    /// Pass back as `if_none_match` to receive `None` when nothing changed.
    pub etag: Option<String>,
}

/// How often notifications are batched into emails.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(into = "u8")]
pub enum EmailBatching {
    Never = 0,
    Hourly = 1,
    ThreeHourly = 2,
    Daily = 3,
    Weekly = 4,
}

impl From<EmailBatching> for u8 {
    fn from(value: EmailBatching) -> Self {
        value as u8
    }
}

/// Body for the personal notification settings.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationSettings {
    pub batch_setting: EmailBatching,
    #[serde(serialize_with = "yes_no")]
    pub sound_notification: bool,
    #[serde(serialize_with = "yes_no")]
    pub sound_talk: bool,
}

fn yes_no<S: serde::Serializer>(value: &bool, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(if *value { "yes" } else { "no" })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_talk_notification() {
        let json = r#"{
            "notification_id": 5, "app": "spreed", "user": "alice",
            "datetime": "2026-10-04T12:00:00+00:00", "object_type": "chat",
            "object_id": "abc123/42", "subject": "Bob mentioned you", "message": "hi",
            "subjectRich": "{user} mentioned you", "subjectRichParameters": {
                "user": {"type": "user", "id": "bob", "name": "Bob"}
            },
            "messageRich": "", "messageRichParameters": [], "link": "", "icon": "",
            "actions": [{"label": "View", "link": "/x", "type": "WEB", "primary": true}]
        }"#;
        let n: Notification = serde_json::from_str(json).unwrap();
        assert_eq!(n.talk_token(), Some("abc123"));
        assert_eq!(n.subject_rich_parameters["user"].id, "bob");
        assert!(n.message_rich_parameters.is_empty());
        assert_eq!(n.actions[0].method, "WEB");
    }

    #[test]
    fn serializes_settings() {
        let body = NotificationSettings {
            batch_setting: EmailBatching::Daily,
            sound_notification: true,
            sound_talk: false,
        };
        assert_eq!(
            serde_json::to_value(body).unwrap(),
            serde_json::json!({"batchSetting": 3, "soundNotification": "yes", "soundTalk": "no"})
        );
    }
}
