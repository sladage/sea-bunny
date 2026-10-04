use std::collections::HashMap;

use serde::Deserialize;

use crate::dto::serde_ext::empty_as_default;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct TranscriptionLanguage {
    pub name: String,
    pub metadata: TranscriptionLanguageMetadata,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct TranscriptionLanguageMetadata {
    /// Separator between words (empty for languages without spaces).
    pub separator: String,
    pub rtl: bool,
}

/// Languages keyed by language id.
pub type TranscriptionLanguages = HashMap<String, TranscriptionLanguage>;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct TranslationLanguages {
    #[serde(deserialize_with = "empty_as_default")]
    pub origin_languages: TranscriptionLanguages,
    #[serde(deserialize_with = "empty_as_default")]
    pub target_languages: TranscriptionLanguages,
    pub default_target_language_id: String,
}
