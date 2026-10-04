//! Reading and dismissing notifications of all apps (Talk mentions, calls,
//! reminders, ...).

use eventful_rs::*;

use crate::{
    dto::notifications::{Notification, NotificationList, NotificationSettings},
    services::ncclient::NcError,
};

service! {
    pub struct NotificationService;
}

#[asynchronize(pub)]
impl NotificationService {
    /// All notifications of the current user, newest first. With the ETag of a
    /// previous list, returns `None` when nothing changed.
    #[asynced]
    pub async fn list(
        &self,
        if_none_match: Option<String>,
    ) -> Result<Option<NotificationList>, NcError> {
        fetch_notifications(&self.client, if_none_match.as_deref()).await
    }

    #[asynced]
    pub async fn get(&self, id: i64) -> Result<Notification, NcError> {
        Ok(self
            .client
            .get(&notifications_path(&format!("/{id}")))
            .send()
            .await?
            .data)
    }

    #[asynced]
    pub async fn dismiss(&self, id: i64) -> Result<(), NcError> {
        self.client
            .delete(&notifications_path(&format!("/{id}")))
            .send_discarding_data()
            .await?;
        Ok(())
    }

    #[asynced]
    pub async fn dismiss_all(&self) -> Result<(), NcError> {
        self.client
            .delete(&notifications_path(""))
            .send_discarding_data()
            .await?;
        Ok(())
    }

    /// Which of the given notification ids still exist (e.g. to clean up shown
    /// desktop notifications).
    #[asynced]
    pub async fn existing(&self, ids: Vec<i64>) -> Result<Vec<i64>, NcError> {
        Ok(self
            .client
            .post(&notifications_path("/exists"))
            .json(&serde_json::json!({ "ids": ids }))
            .send()
            .await?
            .data)
    }

    #[asynced]
    pub async fn set_settings(&self, settings: NotificationSettings) -> Result<(), NcError> {
        self.client
            .post("ocs/v2.php/apps/notifications/api/v2/settings")
            .json(&settings)
            .send_discarding_data()
            .await?;
        Ok(())
    }
}

/// Shared with `NotificationFeedService`. `None` when unchanged since `if_none_match`.
pub(crate) async fn fetch_notifications(
    client: &crate::services::ncclient::NCClient,
    if_none_match: Option<&str>,
) -> Result<Option<NotificationList>, NcError> {
    let mut request = client.get(&notifications_path(""));
    if let Some(etag) = if_none_match {
        request = request.header("If-None-Match", etag);
    }
    let response = request.send_optional::<Vec<Notification>>().await?;
    if response.status == reqwest::StatusCode::NOT_MODIFIED {
        return Ok(None);
    }
    Ok(Some(NotificationList {
        etag: response.header("ETag").map(str::to_owned),
        // 204: no notifications.
        notifications: response.data.unwrap_or_default(),
    }))
}

fn notifications_path(suffix: &str) -> String {
    format!("ocs/v2.php/apps/notifications/api/v2/notifications{suffix}")
}
