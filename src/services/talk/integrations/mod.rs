//! Integrations with other servers and services.

pub mod bots;
pub mod federation;
pub mod matterbridge;

use eventful_rs::ShardRcHandle;

pub use bots::BotService;
pub use federation::FederationService;
pub use matterbridge::MatterbridgeService;

use crate::services::ncclient::{NCClient, NcError};

#[derive(Clone)]
pub struct IntegrationServices {
    pub bots: ShardRcHandle<BotService>,
    pub federation: ShardRcHandle<FederationService>,
    pub matterbridge: ShardRcHandle<MatterbridgeService>,
}

impl IntegrationServices {
    pub async fn new(client: &ShardRcHandle<NCClient>) -> Result<Self, NcError> {
        Ok(Self {
            bots: BotService::new(client).await?,
            federation: FederationService::new(client).await?,
            matterbridge: MatterbridgeService::new(client).await?,
        })
    }
}
