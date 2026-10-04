//! Guest-only endpoints: display name and avatar of a guest session. They do
//! nothing for logged-in users.

use eventful_rs::*;
use reqwest::multipart::{Form, Part};

use crate::services::{
    ncclient::NcError,
    talk::{ApiVersion, talk_path},
};

const TEMP_AVATAR_PATH: &str = "ocs/v2.php/apps/spreed/temp-user-avatar";

service! {
    pub struct GuestService;
}

#[asynchronize(pub)]
impl GuestService {
    #[asynced]
    pub async fn set_display_name(
        &self,
        token: String,
        display_name: String,
    ) -> Result<(), NcError> {
        self.client
            .post(&talk_path(ApiVersion::V1, format!("guest/{token}/name")))
            .json(&serde_json::json!({ "displayName": display_name }))
            .send_discarding_data()
            .await?;
        Ok(())
    }

    /// Upload a square PNG or JPEG avatar for the guest.
    #[asynced]
    pub async fn set_avatar(&self, image: Vec<u8>, mime_type: String) -> Result<(), NcError> {
        let part = Part::bytes(image)
            .file_name("avatar")
            .mime_str(&mime_type)
            .map_err(NcError::from)?;
        self.client
            .post(TEMP_AVATAR_PATH)
            .multipart(Form::new().part("files[]", part))
            .send_discarding_data()
            .await?;
        Ok(())
    }

    #[asynced]
    pub async fn delete_avatar(&self) -> Result<(), NcError> {
        self.client
            .delete(TEMP_AVATAR_PATH)
            .send_discarding_data()
            .await?;
        Ok(())
    }
}
