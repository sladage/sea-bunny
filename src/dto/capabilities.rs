//! Response of `GET ocs/v2.php/cloud/capabilities`.
//!
//! Only the parts the client acts on are typed; everything else is kept as raw JSON
//! so new server capabilities never break deserialization.

use serde::Deserialize;
use serde_json::{Map, Value};

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ServerCapabilities {
    pub version: ServerVersion,
    pub capabilities: CapabilitySet,
}

impl ServerCapabilities {
    /// Talk capabilities, or `None` when the Talk app is not enabled for this user.
    pub fn talk(&self) -> Option<&TalkCapabilities> {
        self.capabilities.spreed.as_ref()
    }

    pub fn has_talk_feature(&self, feature: &str) -> bool {
        self.talk().is_some_and(|talk| talk.has_feature(feature))
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ServerVersion {
    pub major: i64,
    pub minor: i64,
    pub micro: i64,
    pub string: String,
    pub edition: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct CapabilitySet {
    pub spreed: Option<TalkCapabilities>,
    /// Capabilities of all other apps, keyed by app id.
    #[serde(flatten)]
    pub other: Map<String, Value>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct TalkCapabilities {
    pub features: Vec<String>,
    /// Features that only apply to this server, not to federated conversations.
    #[serde(rename = "features-local")]
    pub features_local: Vec<String>,
    pub config: TalkConfig,
    pub version: String,
}

impl TalkCapabilities {
    pub fn has_feature(&self, feature: &str) -> bool {
        self.features.iter().any(|f| f == feature)
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct TalkConfig {
    pub attachments: AttachmentsConfig,
    pub call: CallConfig,
    pub chat: ChatConfig,
    pub conversations: ConversationsConfig,
    pub federation: FederationConfig,
    pub signaling: SignalingConfig,
    /// Remaining sections (`previews`, `experiments`, `permissions`, ...).
    #[serde(flatten)]
    pub other: Map<String, Value>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct AttachmentsConfig {
    pub allowed: bool,
    pub folder: Option<String>,
    pub conversation_subfolders: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct CallConfig {
    pub enabled: bool,
    pub breakout_rooms: bool,
    pub recording: bool,
    pub recording_consent: i64,
    pub sip_enabled: bool,
    pub sip_dialout_enabled: bool,
    pub can_enable_sip: bool,
    pub max_duration: i64,
    pub supported_reactions: Vec<String>,
    #[serde(flatten)]
    pub other: Map<String, Value>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct ChatConfig {
    pub max_length: i64,
    pub read_privacy: i64,
    pub typing_privacy: i64,
    pub has_translation_providers: bool,
    pub summary_threshold: i64,
    pub style: Option<String>,
    #[serde(flatten)]
    pub other: Map<String, Value>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct ConversationsConfig {
    pub can_create: bool,
    pub description_length: i64,
    pub force_passwords: bool,
    pub list_style: Option<String>,
    pub sort_order: Option<String>,
    pub group_mode: Option<String>,
    #[serde(flatten)]
    pub other: Map<String, Value>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct FederationConfig {
    pub enabled: bool,
    pub incoming_enabled: bool,
    pub outgoing_enabled: bool,
    pub only_trusted_servers: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct SignalingConfig {
    pub mode: Option<String>,
    pub session_ping_limit: i64,
    pub hello_v2_token_key: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_talk_capabilities() {
        let json = r#"{
            "version": {"major": 32, "minor": 0, "micro": 1, "string": "32.0.1", "edition": ""},
            "capabilities": {
                "core": {"pollinterval": 60},
                "spreed": {
                    "features": ["audio", "chat-v2", "archived-conversations-v2"],
                    "features-local": ["favorites"],
                    "config": {
                        "chat": {"max-length": 32000, "read-privacy": 0, "typing-privacy": 0},
                        "conversations": {"can-create": true},
                        "previews": {"max-gif-size": 3145728}
                    },
                    "version": "22.0.0"
                }
            }
        }"#;
        let caps: ServerCapabilities = serde_json::from_str(json).unwrap();
        assert_eq!(caps.version.major, 32);
        assert!(caps.has_talk_feature("chat-v2"));
        assert!(!caps.has_talk_feature("nope"));
        let talk = caps.talk().unwrap();
        assert_eq!(talk.config.chat.max_length, 32000);
        assert!(talk.config.conversations.can_create);
        assert!(talk.config.other.contains_key("previews"));
        assert!(caps.capabilities.other.contains_key("core"));
    }
}
