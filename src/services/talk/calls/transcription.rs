//! Live transcription and translation of calls. Transcripts are delivered through
//! the external signaling server.

use eventful_rs::*;

use crate::{
    dto::{
        serde_ext::PhpMap,
        talk::{TranscriptionLanguages, TranslationLanguages},
    },
    services::{
        ncclient::NcError,
        talk::{ApiVersion, talk_path},
    },
};

service! {
    pub struct LiveTranscriptionService;
}

#[asynchronize(pub)]
impl LiveTranscriptionService {
    /// Receive transcriptions in the current call.
    #[asynced]
    pub async fn enable(&self, token: String) -> Result<(), NcError> {
        self.client
            .post(&transcription_path(&token))
            .send_discarding_data()
            .await?;
        Ok(())
    }

    #[asynced]
    pub async fn disable(&self, token: String) -> Result<(), NcError> {
        self.client
            .delete(&transcription_path(&token))
            .send_discarding_data()
            .await?;
        Ok(())
    }

    /// Spoken language of the conversation (moderators only).
    #[asynced]
    pub async fn set_language(&self, token: String, language_id: String) -> Result<(), NcError> {
        self.client
            .post(&format!("{}/language", transcription_path(&token)))
            .json(&serde_json::json!({ "languageId": language_id }))
            .send_discarding_data()
            .await?;
        Ok(())
    }

    /// Language to translate transcriptions into for the current user.
    #[asynced]
    pub async fn set_target_language(
        &self,
        token: String,
        language_id: String,
    ) -> Result<(), NcError> {
        self.client
            .post(&format!("{}/target-language", transcription_path(&token)))
            .json(&serde_json::json!({ "targetLanguageId": language_id }))
            .send_discarding_data()
            .await?;
        Ok(())
    }

    #[asynced]
    pub async fn languages(&self) -> Result<TranscriptionLanguages, NcError> {
        Ok(self
            .client
            .get(&talk_path(ApiVersion::V1, "live-transcription/languages"))
            .send::<PhpMap<_>>()
            .await?
            .data
            .0)
    }

    #[asynced]
    pub async fn translation_languages(&self) -> Result<TranslationLanguages, NcError> {
        Ok(self
            .client
            .get(&talk_path(
                ApiVersion::V1,
                "live-transcription/translation-languages",
            ))
            .send()
            .await?
            .data)
    }
}

fn transcription_path(token: &str) -> String {
    talk_path(ApiVersion::V1, format!("live-transcription/{token}"))
}
