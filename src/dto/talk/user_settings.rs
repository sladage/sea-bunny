use serde::{Serialize, Serializer, ser::SerializeStruct};

use super::Privacy;

/// A Talk user setting that can be changed through the API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserSetting {
    /// Folder in the user's files for uploads and received shares.
    AttachmentFolder(String),
    ReadStatusPrivacy(Privacy),
    TypingPrivacy(Privacy),
    PlaySounds(bool),
}

impl Serialize for UserSetting {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut body = serializer.serialize_struct("UserSetting", 2)?;
        match self {
            Self::AttachmentFolder(folder) => {
                body.serialize_field("key", "attachment_folder")?;
                body.serialize_field("value", folder)?;
            }
            Self::ReadStatusPrivacy(privacy) => {
                body.serialize_field("key", "read_status_privacy")?;
                body.serialize_field("value", privacy)?;
            }
            Self::TypingPrivacy(privacy) => {
                body.serialize_field("key", "typing_privacy")?;
                body.serialize_field("value", privacy)?;
            }
            Self::PlaySounds(play) => {
                body.serialize_field("key", "play_sounds")?;
                body.serialize_field("value", if *play { "yes" } else { "no" })?;
            }
        }
        body.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_key_and_value() {
        let json = serde_json::to_value(UserSetting::TypingPrivacy(Privacy::Private)).unwrap();
        assert_eq!(
            json,
            serde_json::json!({"key": "typing_privacy", "value": 1})
        );
        let json = serde_json::to_value(UserSetting::PlaySounds(false)).unwrap();
        assert_eq!(
            json,
            serde_json::json!({"key": "play_sounds", "value": "no"})
        );
    }
}
