//! The current user's Talk settings. Current values are reported in the Talk
//! capabilities (`config => chat => read-privacy`, `config => attachments => folder`, ...).

use eventful_rs::*;

use crate::{
    dto::talk::UserSetting,
    services::{
        ncclient::NcError,
        talk::{ApiVersion, talk_path},
    },
};

service! {
    pub struct UserSettingsService;
}

#[asynchronize(pub)]
impl UserSettingsService {
    #[asynced]
    pub async fn set(&self, setting: UserSetting) -> Result<(), NcError> {
        self.client
            .post(&talk_path(ApiVersion::V1, "settings/user"))
            .json(&setting)
            .send_discarding_data()
            .await?;
        Ok(())
    }
}
